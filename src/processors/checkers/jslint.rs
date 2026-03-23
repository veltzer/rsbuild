use anyhow::Result;
use std::path::Path;

use crate::config::JslintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct JslintProcessor {
    config: JslintConfig,
}

impl JslintProcessor {
    pub fn new(config: JslintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("jslint", None, &self.config.args, files)
    }
}

impl_checker!(JslintProcessor,
    config: config,
    description: "Lint JavaScript files with jslint",
    name: crate::processors::names::JSLINT,
    execute: execute_product,
    tools: ["jslint".to_string(), "node".to_string()],
    config_json: true,
    batch: lint_files,
);
