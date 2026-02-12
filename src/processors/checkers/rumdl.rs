use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::RumdlConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct RumdlProcessor {
    project_root: PathBuf,
    config: RumdlConfig,
}

impl RumdlProcessor {
    pub fn new(project_root: PathBuf, config: RumdlConfig) -> Self {
        Self {
            project_root,
            config,
        }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    /// Run rumdl on one or more files
    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, Some("check"), &self.config.args, files, &self.project_root)
    }
}

impl_checker!(RumdlProcessor,
    config: config,
    description: "Lint Markdown files using rumdl",
    name: "rumdl",
    execute: execute_product,
    tool_field: linter,
    config_json: true,
    batch: lint_files,
);
