use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::config::MermaidConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command, check_command_output};

use super::DiscoverParams;

pub struct MermaidProcessor {
    config: MermaidConfig,
}

impl MermaidProcessor {
    pub fn new(config: MermaidConfig) -> Self {
        Self { config }
    }
}

impl ProductDiscovery for MermaidProcessor {
    fn description(&self) -> &str {
        "Convert Mermaid diagrams to PNG/SVG/PDF"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.mmdc_bin.clone(), "node".to_string()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config.scan,
            extra_inputs: &self.config.extra_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: crate::processors::names::MERMAID,
        };
        super::discover_multi_format(graph, file_index, &params, &self.config.formats)
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.outputs.first()
            .context("mermaid product has no output")?;

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create mermaid output directory: {}", parent.display()))?;
        }

        let mut cmd = Command::new(&self.config.mmdc_bin);
        cmd.arg("-i").arg(input);
        cmd.arg("-o").arg(output);
        for arg in &self.config.args {
            cmd.arg(arg);
        }

        let out = run_command(&mut cmd)?;
        check_command_output(&out, format_args!("mmdc {}", input.display()))
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::MERMAID, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
