use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::config::PylintConfig;
use crate::graph::Product;
use crate::processors::{run_checker, config_file_inputs};

pub struct PylintProcessor {
    project_root: PathBuf,
    config: PylintConfig,
}

impl PylintProcessor {
    pub fn new(project_root: PathBuf, config: PylintConfig) -> Self {
        Self {
            project_root,
            config,
        }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    /// Return extra inputs for discover: .pylintrc if it exists
    fn pylintrc_inputs(&self) -> Vec<String> {
        config_file_inputs(".pylintrc")
    }

    /// Run pylint on one or more files
    fn lint_files(&self, py_files: &[&Path]) -> Result<()> {
        run_checker("pylint", None, &self.config.args, py_files, &self.project_root)
    }
}

impl_checker!(PylintProcessor,
    config: config,
    description: "Lint Python files with pylint",
    name: crate::processors::names::PYLINT,
    execute: execute_product,
    tools: ["pylint".to_string()],
    config_json: true,
    batch: lint_files,
    extra_discover_inputs: pylintrc_inputs,
);
