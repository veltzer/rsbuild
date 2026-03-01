use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::config::MarkdownConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command_capture, check_command_output};

use super::DiscoverParams;

pub struct MarkdownProcessor {
    config: MarkdownConfig,
}

impl MarkdownProcessor {
    pub fn new(config: MarkdownConfig) -> Self {
        Self { config }
    }
}

impl ProductDiscovery for MarkdownProcessor {
    fn description(&self) -> &str {
        "Convert Markdown to HTML using markdown"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.markdown_bin.clone(), "perl".to_string()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config.scan,
            extra_inputs: &self.config.extra_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: crate::processors::names::MARKDOWN,
        };
        super::discover_single_format(graph, file_index, &params, "html")
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.outputs.first()
            .context("markdown product has no output")?;

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create markdown output directory: {}", parent.display()))?;
        }

        let mut cmd = Command::new(&self.config.markdown_bin);
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        cmd.arg(input);

        let out = run_command_capture(&mut cmd)?;
        check_command_output(&out, format_args!("markdown {}", input.display()))?;

        fs::write(output, &out.stdout)
            .with_context(|| format!("Failed to write markdown output: {}", output.display()))?;

        Ok(())
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::MARKDOWN, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
