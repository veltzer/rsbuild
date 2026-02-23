use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::PipConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, SiblingFilter, discover_directory_products, scan_root_valid, run_in_anchor_dir, anchor_display_dir, check_command_output};

pub struct PipProcessor {
    config: PipConfig,
}

impl PipProcessor {
    pub fn new(config: PipConfig) -> Self {
        Self { config }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    /// Run pip install -r requirements.txt in the file's directory
    fn execute_pip(&self, requirements_txt: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.config.pip);
        cmd.arg("install");
        cmd.arg("-r").arg(requirements_txt.file_name().unwrap_or_default());
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        let output = run_in_anchor_dir(&mut cmd, requirements_txt)?;
        check_command_output(&output, format_args!("pip install in {}", anchor_display_dir(requirements_txt)))
    }
}

impl ProductDiscovery for PipProcessor {
    fn description(&self) -> &str {
        "Install Python dependencies using pip"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_process() && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.pip.clone()]
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
                extensions: &[".txt", ".py"],
                excludes: &["/.git/", "/out/", "/.rsb/", "/node_modules/"],
            },
            crate::processors::names::PIP,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.execute_pip(product.primary_input())
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
