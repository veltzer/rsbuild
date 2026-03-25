use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::config::ChromiumConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command, check_command_output};

use super::DiscoverParams;

pub struct ChromiumProcessor {
    config: ChromiumConfig,
}

impl ChromiumProcessor {
    pub fn new(config: ChromiumConfig) -> Self {
        Self { config }
    }
}

impl ProductDiscovery for ChromiumProcessor {
    fn description(&self) -> &str {
        "Convert HTML to PDF using headless Chromium"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.chromium_bin.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config.scan,
            extra_inputs: &self.config.extra_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: crate::processors::names::CHROMIUM,
        };
        super::discover_single_format(graph, file_index, &params, "pdf")
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.primary_output();

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create chromium output directory: {}", parent.display()))?;
        }

        // Convert the input path to an absolute file:// URL for Chromium
        let abs_input = fs::canonicalize(input)
            .with_context(|| format!("Failed to resolve absolute path for: {}", input.display()))?;
        let input_url = format!("file://{}", abs_input.display());

        let mut cmd = Command::new(&self.config.chromium_bin);
        cmd.arg("--headless");
        cmd.arg("--disable-gpu");
        cmd.arg("--no-sandbox");
        cmd.arg(format!("--print-to-pdf={}", output.display()));
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        cmd.arg(&input_url);

        let out = run_command(&mut cmd)?;
        check_command_output(&out, format_args!("chromium {}", input.display()))
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::CHROMIUM, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
