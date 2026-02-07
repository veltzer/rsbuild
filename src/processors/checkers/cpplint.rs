use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::config::CpplintConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, discover_checker_products, scan_root, run_command, check_command_output};

pub struct CpplintProcessor {
    project_root: PathBuf,
    cpplint_config: CpplintConfig,
}

impl CpplintProcessor {
    pub fn new(project_root: PathBuf, cpplint_config: CpplintConfig) -> Self {
        Self {
            project_root,
            cpplint_config,
        }
    }

    /// Check if C/C++ linting should be enabled
    fn should_lint(&self) -> bool {
        scan_root(&self.cpplint_config.scan).as_os_str().is_empty() || scan_root(&self.cpplint_config.scan).exists()
    }
}

impl ProductDiscovery for CpplintProcessor {
    fn description(&self) -> &str {
        "Run static analysis on C/C++ source files"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_lint() && !file_index.scan(&self.cpplint_config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.cpplint_config.checker.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        if !self.should_lint() {
            return Ok(());
        }
        discover_checker_products(
            graph,
            &self.cpplint_config.scan,
            file_index,
            &self.cpplint_config.extra_inputs,
            &self.cpplint_config,
            "cpplint",
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let mut cmd = Command::new(&self.cpplint_config.checker);
        for arg in &self.cpplint_config.args {
            cmd.arg(arg);
        }
        cmd.arg(&product.inputs[0]);
        cmd.current_dir(&self.project_root);

        let output = run_command(&mut cmd)?;
        check_command_output(&output, "cpplint")
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.cpplint_config).ok()
    }
}
