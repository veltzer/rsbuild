use anyhow::Result;
use std::path::Path;

use crate::config::JqConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct JqProcessor {
    config: JqConfig,
}

impl JqProcessor {
    pub fn new(config: JqConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    /// Run jq on one or more files.
    /// Uses `jq empty` which validates JSON syntax without producing output.
    fn check_files(&self, files: &[&Path]) -> Result<()> {
        let mut args = vec!["empty".to_string()];
        args.extend_from_slice(&self.config.args);
        run_checker(&self.config.checker, None, &args, files)
    }
}

impl_checker!(JqProcessor,
    config: config,
    description: "Validate JSON files with jq",
    name: crate::processors::names::JQ,
    execute: execute_product,
    tool_field: checker,
    config_json: true,
    batch: check_files,
);
