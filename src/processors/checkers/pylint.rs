use anyhow::Result;
use std::path::Path;

use crate::config::PylintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct PylintProcessor {
    config: PylintConfig,
}

impl PylintProcessor {
    pub fn new(config: PylintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    /// Run pylint on one or more files
    fn lint_files(&self, py_files: &[&Path]) -> Result<()> {
        run_checker("pylint", None, &self.config.args, py_files)
    }
}

impl_checker!(PylintProcessor,
    config: config,
    description: "Lint Python files with pylint",
    name: crate::processors::names::PYLINT,
    execute: execute_product,
    tools: ["pylint".to_string(), "python3".to_string()],
    config_json: true,
    batch: lint_files,
);
