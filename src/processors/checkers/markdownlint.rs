use anyhow::Result;
use std::process::Command;

use crate::config::MarkdownlintConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProcessorBase, Processor, check_command_output, run_command};

pub struct MarkdownlintProcessor {
    base: ProcessorBase,
    config: MarkdownlintConfig,
}

impl MarkdownlintProcessor {
    pub fn new(config: MarkdownlintConfig) -> Self {
        Self {
            base: ProcessorBase::checker(crate::processors::names::MARKDOWNLINT, "Lint Markdown files using markdownlint"),
            config,
        }
    }
}

impl Processor for MarkdownlintProcessor {
    fn scan_config(&self) -> &crate::config::StandardConfig {
        &self.config.standard
    }

    fn standard_config(&self) -> Option<&crate::config::StandardConfig> {
        Some(&self.config.standard)
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn processor_type(&self) -> crate::processors::ProcessorType {
        self.base.processor_type()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.markdownlint_bin.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex, instance_name: &str) -> Result<()> {
        crate::processors::discover_checker_products(
            graph, &self.config.standard, file_index,
            &self.config.standard.dep_inputs, &self.config.standard.dep_auto,
            &self.config, instance_name,
        )
    }

    fn supports_batch(&self) -> bool { false }

    fn execute(&self, ctx: &crate::build_context::BuildContext, product: &Product) -> Result<()> {
        let file = product.primary_input();
        let mut cmd = Command::new(&self.config.markdownlint_bin);
        for arg in &self.config.standard.args {
            cmd.arg(arg);
        }
        cmd.arg(file);
        let output = run_command(ctx, &mut cmd)?;
        check_command_output(&output, format_args!("markdownlint {}", file.display()))
    }
}

fn plugin_create(toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registries::deserialize_and_create(toml, |cfg| Box::new(MarkdownlintProcessor::new(cfg)))
}
inventory::submit! {
    crate::registries::ProcessorPlugin {
        version: 1,
        name: "markdownlint",
        processor_type: crate::processors::ProcessorType::Checker,
        create: plugin_create,
        defconfig_json: crate::registries::default_config_json::<crate::config::MarkdownlintConfig>,
        known_fields: crate::registries::typed_known_fields::<crate::config::MarkdownlintConfig>,
        output_fields: crate::registries::typed_output_fields::<crate::config::MarkdownlintConfig>,
        must_fields: crate::registries::typed_must_fields::<crate::config::MarkdownlintConfig>,
        field_descriptions: crate::registries::typed_field_descriptions::<crate::config::MarkdownlintConfig>,
        keywords: &["markdown", "md", "linter", "node", "npm"],
    }
}
