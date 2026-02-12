use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::RuffConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct RuffProcessor {
    project_root: PathBuf,
    config: RuffConfig,
}

impl RuffProcessor {
    pub fn new(project_root: PathBuf, config: RuffConfig) -> Self {
        Self {
            project_root,
            config,
        }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    /// Run the configured linter on one or more files
    fn lint_files(&self, py_files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, Some("check"), &self.config.args, py_files, &self.project_root)
    }
}

impl_checker!(RuffProcessor,
    config: config,
    description: "Lint Python files with ruff",
    name: crate::processors::names::RUFF,
    execute: execute_product,
    tool_field: linter,
    config_json: true,
    batch: lint_files,
);
