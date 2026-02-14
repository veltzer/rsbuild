use anyhow::Result;
use std::path::Path;

use crate::config::YamllintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct YamllintProcessor {
    config: YamllintConfig,
}

impl YamllintProcessor {
    pub fn new(config: YamllintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, None, &self.config.args, files)
    }
}

impl_checker!(YamllintProcessor,
    config: config,
    description: "Lint YAML files with yamllint",
    name: crate::processors::names::YAMLLINT,
    execute: execute_product,
    tool_field: linter,
    config_json: true,
    batch: lint_files,
);
