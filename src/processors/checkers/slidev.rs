use anyhow::Result;
use std::path::Path;

use crate::config::SlidevConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct SlidevProcessor {
    config: SlidevConfig,
}

impl SlidevProcessor {
    pub fn new(config: SlidevConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("slidev", Some("build"), &self.config.args, files)
    }
}

impl_checker!(SlidevProcessor,
    config: config,
    description: "Build Slidev presentations",
    name: crate::processors::names::SLIDEV,
    execute: execute_product,
    tools: ["slidev".to_string(), "node".to_string()],
    config_json: true,
    batch: lint_files,
);
