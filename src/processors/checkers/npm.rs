use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::NpmConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, SiblingFilter, discover_directory_products, scan_root_valid, run_in_anchor_dir, anchor_display_dir, check_command_output};

pub struct NpmProcessor {
    config: NpmConfig,
}

impl NpmProcessor {
    pub fn new(config: NpmConfig) -> Self {
        Self { config }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    /// Run npm install in the package.json's directory
    fn execute_npm(&self, package_json: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.config.npm);
        cmd.arg(&self.config.command);
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        let output = run_in_anchor_dir(&mut cmd, package_json)?;
        check_command_output(&output, format_args!("npm {} in {}", self.config.command, anchor_display_dir(package_json)))
    }
}

impl ProductDiscovery for NpmProcessor {
    fn description(&self) -> &str {
        "Install Node.js dependencies using npm"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_process() && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.npm.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        if !self.should_process() {
            return Ok(());
        }

        discover_directory_products(
            graph,
            &self.config.scan,
            file_index,
            &self.config.extra_inputs,
            &self.config,
            &SiblingFilter {
                extensions: &[".json", ".js", ".ts"],
                excludes: &["/.git/", "/out/", "/.rsb/", "/node_modules/"],
            },
            crate::processors::names::NPM,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.execute_npm(product.primary_input())
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
