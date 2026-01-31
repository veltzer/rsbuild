use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::ShellcheckConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use super::{ProductDiscovery, discover_stub_products, scan_root, validate_stub_product, ensure_stub_dir, write_stub, clean_outputs, run_command, check_command_output};

const SHELLCHECK_STUB_DIR: &str = "out/shellcheck";

pub struct ShellcheckProcessor {
    project_root: PathBuf,
    config: ShellcheckConfig,
    stub_dir: PathBuf,
}

impl ShellcheckProcessor {
    pub fn new(project_root: PathBuf, config: ShellcheckConfig) -> Self {
        let stub_dir = project_root.join(SHELLCHECK_STUB_DIR);
        Self {
            project_root,
            config,
            stub_dir,
        }
    }

    /// Check if shell linting should be enabled
    fn should_lint(&self) -> bool {
        scan_root(&self.project_root, &self.config.scan).exists()
    }

    /// Run shellcheck on a single file and create stub
    fn check_file(&self, source_file: &Path, stub_path: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.config.checker);

        for arg in &self.config.args {
            cmd.arg(arg);
        }

        cmd.arg(source_file);
        cmd.current_dir(&self.project_root);

        let output = run_command(&mut cmd)?;
        check_command_output(&output, "shellcheck")?;
        write_stub(stub_path, "checked")
    }
}

impl ProductDiscovery for ShellcheckProcessor {
    fn description(&self) -> &str {
        "Lint shell scripts using shellcheck"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_lint() && !file_index.scan(&self.project_root, &self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.checker.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        if !self.should_lint() {
            return Ok(());
        }
        discover_stub_products(
            graph,
            &self.project_root,
            &self.stub_dir,
            &self.config.scan,
            file_index,
            &self.config.extra_inputs,
            &self.config,
            "shellcheck",
            "shellcheck",
            true,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        validate_stub_product(product, "Shellcheck")?;
        ensure_stub_dir(&self.stub_dir, "shellcheck")?;
        self.check_file(&product.inputs[0], &product.outputs[0])
    }

    fn clean(&self, product: &Product) -> Result<()> {
        clean_outputs(product, "shellcheck")
    }
}
