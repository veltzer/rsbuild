//! sass generator — registered as a SimpleGenerator with a custom execute fn.

use std::process::Command;
use anyhow::Result;

use crate::config::StandardConfig;
use crate::graph::Product;
use crate::processors::{run_command, check_command_output, ensure_output_dir};

use super::simple::{SimpleGenerator, SimpleGeneratorParams, DiscoverMode};

fn execute_sass(config: &StandardConfig, product: &Product) -> Result<()> {
    let input = product.primary_input();
    let output = product.primary_output();
    ensure_output_dir(output)?;
    let mut cmd = Command::new(&config.command);
    for arg in &config.args { cmd.arg(arg); }
    cmd.arg(input).arg(output);
    let out = run_command(&mut cmd)?;
    check_command_output(&out, format_args!("sass {}", input.display()))
}


fn create_sass(toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::deserialize_and_create(toml, |cfg| Box::new(SimpleGenerator::new(cfg, SimpleGeneratorParams { description: "Compile Sass/SCSS to CSS", extra_tools: &[], discover_mode: DiscoverMode::SingleFormat("css"), execute_fn: execute_sass, is_native: false })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    version: 1,
    name: "sass", processor_type: crate::processors::ProcessorType::Generator, create: create_sass,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::default_config_json::<crate::config::StandardConfig>,
} }
