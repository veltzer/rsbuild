use anyhow::Result;
use std::path::Path;

use crate::config::YqConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct YqProcessor {
    config: YqConfig,
}

impl YqProcessor {
    pub fn new(config: YqConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("yq", Some("."), &self.config.args, files)
    }
}

impl_checker!(YqProcessor,
    config: config,
    description: "Validate YAML files with yq",
    name: crate::processors::names::YQ,
    execute: execute_product,
    tools: ["yq".to_string()],
    config_json: true,
    batch: lint_files,
);
