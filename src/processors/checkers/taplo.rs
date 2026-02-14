use anyhow::Result;
use std::path::Path;

use crate::config::TaploConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct TaploProcessor {
    config: TaploConfig,
}

impl TaploProcessor {
    pub fn new(config: TaploConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    fn check_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, Some("check"), &self.config.args, files)
    }
}

impl_checker!(TaploProcessor,
    config: config,
    description: "Check TOML files with taplo",
    name: crate::processors::names::TAPLO,
    execute: execute_product,
    tool_field: linter,
    config_json: true,
    batch: check_files,
);
