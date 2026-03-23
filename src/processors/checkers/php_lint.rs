use anyhow::Result;
use std::path::Path;

use crate::config::PhpLintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct PhpLintProcessor {
    config: PhpLintConfig,
}

impl PhpLintProcessor {
    pub fn new(config: PhpLintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("php", Some("-l"), &self.config.args, files)
    }
}

impl_checker!(PhpLintProcessor,
    config: config,
    description: "Check PHP syntax with php -l",
    name: crate::processors::names::PHP_LINT,
    execute: execute_product,
    tools: ["php".to_string()],
    config_json: true,
    batch: lint_files,
);
