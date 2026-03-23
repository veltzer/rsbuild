use anyhow::Result;
use std::path::Path;

use crate::config::StandardConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct StandardProcessor {
    config: StandardConfig,
}

impl StandardProcessor {
    pub fn new(config: StandardConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("standard", None, &self.config.args, files)
    }
}

impl_checker!(StandardProcessor,
    config: config,
    description: "Check JavaScript style with standard",
    name: crate::processors::names::STANDARD,
    execute: execute_product,
    tools: ["standard".to_string(), "node".to_string()],
    config_json: true,
    batch: lint_files,
);
