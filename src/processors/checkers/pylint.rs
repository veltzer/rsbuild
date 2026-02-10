use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::PylintConfig;
use crate::graph::Product;
use crate::processors::{run_command, check_command_output};

pub struct PylintProcessor {
    project_root: PathBuf,
    pylint_config: PylintConfig,
}

impl PylintProcessor {
    pub fn new(project_root: PathBuf, pylint_config: PylintConfig) -> Self {
        Self {
            project_root,
            pylint_config,
        }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.inputs[0].as_path()])
    }

    /// Return extra inputs for discover: .pylintrc if it exists
    fn pylintrc_inputs(&self) -> Vec<String> {
        if Path::new(".pylintrc").exists() {
            vec![".pylintrc".to_string()]
        } else {
            Vec::new()
        }
    }

    /// Run pylint on one or more files
    fn lint_files(&self, py_files: &[&Path]) -> Result<()> {
        let mut cmd = Command::new("pylint");

        for arg in &self.pylint_config.args {
            cmd.arg(arg);
        }

        for file in py_files {
            cmd.arg(file);
        }
        cmd.current_dir(&self.project_root);

        let output = run_command(&mut cmd)?;
        check_command_output(&output, "pylint")
    }
}

impl_checker!(PylintProcessor,
    config: pylint_config,
    description: "Lint Python files with pylint",
    name: "pylint",
    execute: execute_product,
    tools: ["pylint".to_string()],
    config_json: true,
    batch: lint_files,
    extra_discover_inputs: pylintrc_inputs,
);
