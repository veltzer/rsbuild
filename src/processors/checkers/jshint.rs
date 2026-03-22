use anyhow::Result;
use std::path::Path;

use crate::config::JshintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct JshintProcessor {
    config: JshintConfig,
}

impl JshintProcessor {
    pub fn new(config: JshintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, None, &self.config.args, files)
    }
}

impl_checker!(JshintProcessor,
    config: config,
    description: "Lint JavaScript files with jshint",
    name: crate::processors::names::JSHINT,
    execute: execute_product,
    tool_field_extra: linter ["node".to_string()],
    config_json: true,
    batch: lint_files,
);
