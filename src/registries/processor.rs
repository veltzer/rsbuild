//! Processor plugin registry.
//!
//! Every built-in processor (checker, generator, creator, mass-generator) submits
//! a [`ProcessorPlugin`] entry via `inventory::submit!`. The inventory is collected
//! at link time.

use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::KnownFields;
use crate::processors::{Processor, ProcessorType};

/// A processor plugin. One struct for all processor types.
/// Each processor file submits one of these via `inventory::submit!`.
///
/// The plugin is a factory: it knows its name, type, how to create a processor
/// from TOML config, and metadata about its config fields.
///
/// The framework applies defaults to the TOML before calling `create`.
/// The `create` function deserializes the TOML and returns a fully configured,
/// immutable processor.
pub struct ProcessorPlugin {
    pub name: &'static str,
    /// Processor type. Declared by every plugin but not yet queried by any
    /// runtime code path — kept as plugin metadata so future features
    /// (e.g. `processors list --type=checker`) can filter without touching
    /// every registration.
    #[allow(dead_code)]
    pub processor_type: ProcessorType,
    /// Implementation version. **Bump this when changes would make the processor
    /// produce different output for the same inputs**, or change which inputs are
    /// discovered, which outputs are declared, or how config fields are interpreted.
    /// Do NOT bump for refactors, comments, reformats, or behavior-preserving
    /// bug fixes. See `docs/src/processor-versioning.md` for the full bump rule.
    ///
    /// The version is mixed into every product's cache key, so bumping here
    /// invalidates caches only for this processor (leaves others untouched).
    pub version: u32,
    /// Create a processor from resolved TOML config (defaults already applied).
    pub create: fn(&toml::Value) -> Result<Box<dyn Processor>>,
    /// Config metadata
    pub known_fields: fn() -> &'static [&'static str],
    pub output_fields: fn() -> &'static [&'static str],
    pub must_fields: fn() -> &'static [&'static str],
    pub field_descriptions: fn() -> &'static [(&'static str, &'static str)],
    /// Return the default config as pretty JSON. Receives the processor name
    /// so it can apply the correct defaults.
    pub defconfig_json: fn(&str) -> Option<String>,
}

unsafe impl Sync for ProcessorPlugin {}

inventory::collect!(ProcessorPlugin);

pub(crate) fn all_plugins() -> impl Iterator<Item = &'static ProcessorPlugin> {
    inventory::iter::<ProcessorPlugin>.into_iter()
}

/// Look up a processor's implementation version by name.
/// Returns `None` for processor names not in the builtin registry (e.g. Lua plugins).
/// Used by `Product::descriptor_key` to mix the processor's version into every
/// cache key, so bumping a processor's `version` invalidates exactly that
/// processor's cached outputs.
pub(crate) fn processor_version(name: &str) -> Option<u32> {
    all_plugins().find(|p| p.name == name).map(|p| p.version)
}

/// Build a clap value parser that accepts any registered processor type name (pname).
pub(crate) fn processor_name_parser() -> clap::builder::PossibleValuesParser {
    let mut names: Vec<&'static str> = all_plugins().map(|p| p.name).collect();
    names.sort();
    clap::builder::PossibleValuesParser::new(names)
}

/// Apply both processor defaults and scan defaults to a TOML value.
/// Every field that's injected is recorded in `provenance`.
pub fn apply_all_defaults(
    name: &str,
    value: &mut toml::Value,
    provenance: &mut crate::config::ProvenanceMap,
) {
    crate::config::apply_processor_defaults(name, value, provenance);
    crate::config::apply_scan_defaults(name, value, provenance);
}

// --- Helpers that processor files call from their create/defconfig functions ---

/// Deserialize TOML into config type C and call the constructor.
/// The TOML should already have defaults applied by the framework.
pub fn deserialize_and_create<C: Default + DeserializeOwned>(
    config_toml: &toml::Value, ctor: fn(C) -> Box<dyn Processor>,
) -> Result<Box<dyn Processor>> {
    let cfg: C = toml::from_str(&toml::to_string(config_toml)?)?;
    Ok(ctor(cfg))
}

/// Build default config JSON for a config type, applying defaults for the given processor name.
pub fn default_config_json<C: Default + DeserializeOwned + Serialize>(name: &str) -> Option<String> {
    let mut val = toml::Value::Table(toml::map::Map::new());
    let mut prov = crate::config::ProvenanceMap::new();
    apply_all_defaults(name, &mut val, &mut prov);
    let cfg: C = toml::from_str(&toml::to_string(&val).ok()?).ok()?;
    serde_json::to_string_pretty(&serde_json::to_value(cfg).ok()?).ok()
}

pub fn typed_known_fields<C: KnownFields>() -> &'static [&'static str] { C::known_fields() }
pub fn typed_output_fields<C: KnownFields>() -> &'static [&'static str] { C::output_fields() }
pub fn typed_must_fields<C: KnownFields>() -> &'static [&'static str] { C::must_fields() }
pub fn typed_field_descriptions<C: KnownFields>() -> &'static [(&'static str, &'static str)] { C::field_descriptions() }
