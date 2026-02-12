use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::ShellcheckConfig;
use crate::graph::Product;
use crate::processors::{scan_root_valid, run_checker};

pub struct ShellcheckProcessor {
    project_root: PathBuf,
    config: ShellcheckConfig,
}

impl ShellcheckProcessor {
    pub fn new(project_root: PathBuf, config: ShellcheckConfig) -> Self {
        Self {
            project_root,
            config,
        }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    /// Run shellcheck on one or more files
    fn check_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.checker, None, &self.config.args, files, &self.project_root)
    }
}

impl_checker!(ShellcheckProcessor,
    config: config,
    description: "Lint shell scripts using shellcheck",
    name: crate::processors::names::SHELLCHECK,
    execute: execute_product,
    guard: should_process,
    tool_field: checker,
    config_json: true,
    batch: check_files,
);
