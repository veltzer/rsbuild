use anyhow::Result;
use std::path::Path;

use crate::config::HadolintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct HadolintProcessor {
    config: HadolintConfig,
}

impl HadolintProcessor {
    pub fn new(config: HadolintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("hadolint", None, &self.config.args, files)
    }
}

impl_checker!(HadolintProcessor,
    config: config,
    description: "Lint Dockerfiles with hadolint",
    name: crate::processors::names::HADOLINT,
    execute: execute_product,
    tools: ["hadolint".to_string()],
    config_json: true,
    batch: lint_files,
);
