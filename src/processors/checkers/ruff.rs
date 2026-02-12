use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::RuffConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct RuffProcessor {
    project_root: PathBuf,
    ruff_config: RuffConfig,
}

impl RuffProcessor {
    pub fn new(project_root: PathBuf, ruff_config: RuffConfig) -> Self {
        Self {
            project_root,
            ruff_config,
        }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    /// Run the configured linter on one or more files
    fn lint_files(&self, py_files: &[&Path]) -> Result<()> {
        run_checker(&self.ruff_config.linter, Some("check"), &self.ruff_config.args, py_files, &self.project_root)
    }
}

impl_checker!(RuffProcessor,
    config: ruff_config,
    description: "Lint Python files with ruff",
    name: "ruff",
    execute: execute_product,
    tool_field: linter,
    config_json: true,
    batch: lint_files,
);
