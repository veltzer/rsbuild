use std::path::Path;
use anyhow::Result;

use crate::config::{CheckerConfigWithCommand, SimpleCheckerParams};
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{Processor, run_checker, execute_checker_batch,
    discover_checker_products, config_file_inputs};

/// A simple checker processor driven entirely by data.
/// Replaces the `simple_checker!` macro — all 29 trivial checkers use this struct
/// with different `SimpleCheckerParams`.
pub struct SimpleChecker {
    config: CheckerConfigWithCommand,
    params: SimpleCheckerParams,
}

impl SimpleChecker {
    pub fn new(config: CheckerConfigWithCommand, params: SimpleCheckerParams) -> Self {
        Self { config, params }
    }

    fn check_files(&self, files: &[&Path]) -> Result<()> {
        let tool = &self.config.command;
        if self.params.prepend_args.is_empty() {
            run_checker(tool, self.params.subcommand, &self.config.args, files)
        } else {
            let mut combined_args: Vec<String> = self.params.prepend_args.iter().map(|s| s.to_string()).collect();
            combined_args.extend_from_slice(&self.config.args);
            run_checker(tool, self.params.subcommand, &combined_args, files)
        }
    }
}

impl Processor for SimpleChecker {
    fn scan_config(&self) -> &crate::config::ScanConfig {
        &self.config.scan
    }

    fn standard_config(&self) -> Option<&crate::config::StandardConfig> {
        Some(&self.config)
    }

    fn description(&self) -> &str {
        self.params.description
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        let mut tools = vec![self.config.command.clone()];
        for t in self.params.extra_tools {
            tools.push(t.to_string());
        }
        tools
    }

    fn discover(
        &self,
        graph: &mut BuildGraph,
        file_index: &FileIndex,
        instance_name: &str,
    ) -> Result<()> {
        let mut dep_inputs = self.config.dep_inputs.clone();
        for ai in &self.config.dep_auto {
            dep_inputs.extend(config_file_inputs(ai));
        }
        discover_checker_products(
            graph, &self.config.scan, file_index, &dep_inputs, &self.config, instance_name,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }


    fn supports_batch(&self) -> bool {
        self.config.batch
    }

    fn execute_batch(&self, products: &[&Product]) -> Vec<Result<()>> {
        execute_checker_batch(products, |files| self.check_files(files))
    }
}


// --- Plugin registrations ---

fn create_ruff(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("check"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "ruff", processor_type: crate::processors::ProcessorType::Checker, create: create_ruff,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_pylint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "pylint", processor_type: crate::processors::ProcessorType::Checker, create: create_pylint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_pytest(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "pytest", processor_type: crate::processors::ProcessorType::Checker, create: create_pytest,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_black(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &["--check"], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "black", processor_type: crate::processors::ProcessorType::Checker, create: create_black,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_doctest(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &["-m", "doctest"], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "doctest", processor_type: crate::processors::ProcessorType::Checker, create: create_doctest,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_mypy(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "mypy", processor_type: crate::processors::ProcessorType::Checker, create: create_mypy,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_pyrefly(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("check"), prepend_args: &["--disable-project-excludes-heuristics"], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "pyrefly", processor_type: crate::processors::ProcessorType::Checker, create: create_pyrefly,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_rumdl(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("check"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "rumdl", processor_type: crate::processors::ProcessorType::Checker, create: create_rumdl,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_yamllint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "yamllint", processor_type: crate::processors::ProcessorType::Checker, create: create_yamllint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_jq(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &["empty"], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "jq", processor_type: crate::processors::ProcessorType::Checker, create: create_jq,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_jsonlint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["python3"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "jsonlint", processor_type: crate::processors::ProcessorType::Checker, create: create_jsonlint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_taplo(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("check"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "taplo", processor_type: crate::processors::ProcessorType::Checker, create: create_taplo,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_eslint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "eslint", processor_type: crate::processors::ProcessorType::Checker, create: create_eslint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_jshint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "jshint", processor_type: crate::processors::ProcessorType::Checker, create: create_jshint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_htmlhint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "htmlhint", processor_type: crate::processors::ProcessorType::Checker, create: create_htmlhint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_stylelint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "stylelint", processor_type: crate::processors::ProcessorType::Checker, create: create_stylelint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_checkstyle(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "checkstyle", processor_type: crate::processors::ProcessorType::Checker, create: create_checkstyle,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_cmake(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("--lint"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "cmake", processor_type: crate::processors::ProcessorType::Checker, create: create_cmake,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_hadolint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "hadolint", processor_type: crate::processors::ProcessorType::Checker, create: create_hadolint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_htmllint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "htmllint", processor_type: crate::processors::ProcessorType::Checker, create: create_htmllint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_jslint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "jslint", processor_type: crate::processors::ProcessorType::Checker, create: create_jslint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_perlcritic(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["perl"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "perlcritic", processor_type: crate::processors::ProcessorType::Checker, create: create_perlcritic,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_php_lint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("-l"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "php_lint", processor_type: crate::processors::ProcessorType::Checker, create: create_php_lint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_slidev(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("build"), prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "slidev", processor_type: crate::processors::ProcessorType::Checker, create: create_slidev,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_standard(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &["node"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "standard", processor_type: crate::processors::ProcessorType::Checker, create: create_standard,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_svglint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "svglint", processor_type: crate::processors::ProcessorType::Checker, create: create_svglint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_tidy(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("-errors"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "tidy", processor_type: crate::processors::ProcessorType::Checker, create: create_tidy,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_xmllint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("--noout"), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "xmllint", processor_type: crate::processors::ProcessorType::Checker, create: create_xmllint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_yq(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: Some("."), prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "yq", processor_type: crate::processors::ProcessorType::Checker, create: create_yq,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_cppcheck(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "cppcheck", processor_type: crate::processors::ProcessorType::Checker, create: create_cppcheck,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_cpplint(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "cpplint", processor_type: crate::processors::ProcessorType::Checker, create: create_cpplint,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_checkpatch(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &["--no-tree", "-f"], extra_tools: &["perl"] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "checkpatch", processor_type: crate::processors::ProcessorType::Checker, create: create_checkpatch,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_shellcheck(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "shellcheck", processor_type: crate::processors::ProcessorType::Checker, create: create_shellcheck,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

fn create_luacheck(name: &str, toml: &toml::Value) -> anyhow::Result<Box<dyn crate::processors::Processor>> {
    crate::registry::typed_create(name, toml, |cfg| Box::new(SimpleChecker::new(cfg, SimpleCheckerParams { description: "", subcommand: None, prepend_args: &[], extra_tools: &[] })))
}
inventory::submit! { crate::registry::ProcessorPlugin {
    name: "luacheck", processor_type: crate::processors::ProcessorType::Checker, create: create_luacheck,
    resolve_defaults: crate::registry::typed_resolve_defaults::<crate::config::StandardConfig>,
    defconfig_json: crate::registry::typed_defconfig_json::<crate::config::StandardConfig>,
    known_fields: crate::registry::typed_known_fields::<crate::config::StandardConfig>,
    output_fields: crate::registry::typed_output_fields::<crate::config::StandardConfig>,
    must_fields: crate::registry::typed_must_fields::<crate::config::StandardConfig>,
    field_descriptions: crate::registry::typed_field_descriptions::<crate::config::StandardConfig>,
} }

