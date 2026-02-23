use anyhow::Result;
use std::path::Path;
use std::process::Command;

use crate::config::GemConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, SiblingFilter, discover_directory_products, scan_root_valid, run_in_anchor_dir, anchor_display_dir, check_command_output};

pub struct GemProcessor {
    config: GemConfig,
}

impl GemProcessor {
    pub fn new(config: GemConfig) -> Self {
        Self { config }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    /// Run bundle install in the Gemfile's directory
    fn execute_gem(&self, gemfile: &Path) -> Result<()> {
        let mut cmd = Command::new(&self.config.bundler);
        cmd.arg(&self.config.command);
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        let output = run_in_anchor_dir(&mut cmd, gemfile)?;
        check_command_output(&output, format_args!("bundle {} in {}", self.config.command, anchor_display_dir(gemfile)))
    }
}

impl ProductDiscovery for GemProcessor {
    fn description(&self) -> &str {
        "Install Ruby dependencies using Bundler"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_process() && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.bundler.clone()]
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
                extensions: &[".rb", ".gemspec"],
                excludes: &["/.git/", "/out/", "/.rsb/", "/vendor/"],
            },
            crate::processors::names::GEM,
        )
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.execute_gem(product.primary_input())
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
