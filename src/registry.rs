use anyhow::Result;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::config::KnownFields;
use crate::processors::{ProductDiscovery, ProcessorType};

/// A processor plugin. One struct for all processor types.
/// Each processor file submits one of these via `inventory::submit!`.
pub struct ProcessorPlugin {
    pub name: &'static str,
    pub processor_type: ProcessorType,
    pub create: fn(&str, &toml::Value) -> Result<Box<dyn ProductDiscovery>>,
    pub create_default: fn(&str) -> Box<dyn ProductDiscovery>,
    pub resolve_defaults: fn(&str, &mut toml::Value) -> Result<()>,
    pub defconfig_json: fn(&str) -> Option<String>,
    pub known_fields: fn() -> &'static [&'static str],
    pub output_fields: fn() -> &'static [&'static str],
    pub must_fields: fn() -> &'static [&'static str],
    pub field_descriptions: fn() -> &'static [(&'static str, &'static str)],
}

unsafe impl Sync for ProcessorPlugin {}

inventory::collect!(ProcessorPlugin);

pub(crate) fn all_plugins() -> impl Iterator<Item = &'static ProcessorPlugin> {
    inventory::iter::<ProcessorPlugin>.into_iter()
}

/// Apply both processor defaults and scan defaults to a TOML value.
pub fn apply_all_defaults(name: &str, value: &mut toml::Value) {
    crate::config::apply_processor_defaults(name, value);
    crate::config::apply_scan_defaults(name, value);
}

// --- Generic helpers that processors call from their plugin functions ---

pub fn typed_create<C: Default + DeserializeOwned + Serialize + Clone>(
    name: &str, config_toml: &toml::Value, ctor: fn(C) -> Box<dyn ProductDiscovery>,
) -> Result<Box<dyn ProductDiscovery>> {
    let mut val = config_toml.clone();
    apply_all_defaults(name, &mut val);
    let cfg: C = toml::from_str(&toml::to_string(&val)?)?;
    Ok(ctor(cfg))
}

pub fn typed_create_default<C: Default + DeserializeOwned + Serialize + Clone>(
    name: &str, ctor: fn(C) -> Box<dyn ProductDiscovery>,
) -> Box<dyn ProductDiscovery> {
    let val = toml::Value::Table(toml::map::Map::new());
    typed_create(name, &val, ctor).unwrap()
}

pub fn typed_resolve_defaults<C: Default + DeserializeOwned + Serialize + Clone>(
    name: &str, value: &mut toml::Value,
) -> Result<()> {
    apply_all_defaults(name, value);
    let cfg: C = toml::from_str(&toml::to_string(value)?)?;
    *value = toml::Value::try_from(&cfg)?;
    Ok(())
}

pub fn typed_defconfig_json<C: Default + DeserializeOwned + Serialize + Clone>(
    name: &str,
) -> Option<String> {
    let mut val = toml::Value::Table(toml::map::Map::new());
    apply_all_defaults(name, &mut val);
    let cfg: C = toml::from_str(&toml::to_string(&val).ok()?).ok()?;
    serde_json::to_string_pretty(&serde_json::to_value(cfg).ok()?).ok()
}

pub fn typed_known_fields<C: KnownFields>() -> &'static [&'static str] { C::known_fields() }
pub fn typed_output_fields<C: KnownFields>() -> &'static [&'static str] { C::output_fields() }
pub fn typed_must_fields<C: KnownFields>() -> &'static [&'static str] { C::must_fields() }
pub fn typed_field_descriptions<C: KnownFields>() -> &'static [(&'static str, &'static str)] { C::field_descriptions() }
