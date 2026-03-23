use anyhow::Result;
use std::path::Path;

use crate::config::CmakeConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct CmakeProcessor {
    config: CmakeConfig,
}

impl CmakeProcessor {
    pub fn new(config: CmakeConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("cmake", Some("--lint"), &self.config.args, files)
    }
}

impl_checker!(CmakeProcessor,
    config: config,
    description: "Lint CMakeLists.txt files with cmake --lint",
    name: crate::processors::names::CMAKE,
    execute: execute_product,
    tools: ["cmake".to_string()],
    config_json: true,
    batch: lint_files,
);
