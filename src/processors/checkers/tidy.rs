use anyhow::Result;
use std::path::Path;

use crate::config::TidyConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct TidyProcessor {
    config: TidyConfig,
}

impl TidyProcessor {
    pub fn new(config: TidyConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("tidy", Some("-errors"), &self.config.args, files)
    }
}

impl_checker!(TidyProcessor,
    config: config,
    description: "Validate HTML files with tidy",
    name: crate::processors::names::TIDY,
    execute: execute_product,
    tools: ["tidy".to_string()],
    config_json: true,
    batch: lint_files,
);
