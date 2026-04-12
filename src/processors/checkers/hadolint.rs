//! hadolint checker — registered as a {SimpleChecker}.

use super::simple::SimpleChecker;
use crate::config::SimpleCheckerParams;

fn create_hadolint(toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::deserialize_and_create(toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "Lint Dockerfiles using hadolint", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    version: 1,
    name: "hadolint", processor_type: crate::processors::ProcessorType::Checker, create: create_hadolint,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::default_config_json::<crate::config::StandardConfig>,
} }
