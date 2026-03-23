use anyhow::Result;
use std::path::Path;

use crate::config::CheckstyleConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct CheckstyleProcessor {
    config: CheckstyleConfig,
}

impl CheckstyleProcessor {
    pub fn new(config: CheckstyleConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("checkstyle", None, &self.config.args, files)
    }
}

impl_checker!(CheckstyleProcessor,
    config: config,
    description: "Check Java code style with checkstyle",
    name: crate::processors::names::CHECKSTYLE,
    execute: execute_product,
    tools: ["checkstyle".to_string()],
    config_json: true,
    batch: lint_files,
);
