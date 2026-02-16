use anyhow::Result;
use std::path::Path;

use crate::config::PyreflyConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct PyreflyProcessor {
    config: PyreflyConfig,
}

impl PyreflyProcessor {
    pub fn new(config: PyreflyConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    /// Run pyrefly on one or more files
    fn check_files(&self, py_files: &[&Path]) -> Result<()> {
        run_checker(&self.config.checker, Some("check"), &self.config.args, py_files)
    }
}

impl_checker!(PyreflyProcessor,
    config: config,
    description: "Type-check Python files with pyrefly",
    name: crate::processors::names::PYREFLY,
    execute: execute_product,
    tool_field_extra: checker ["python3".to_string()],
    config_json: true,
    batch: check_files,
);
