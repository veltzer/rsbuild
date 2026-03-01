use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::config::DrawioConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command, check_command_output};

use super::DiscoverParams;

pub struct DrawioProcessor {
    config: DrawioConfig,
}

impl DrawioProcessor {
    pub fn new(config: DrawioConfig) -> Self {
        Self { config }
    }
}

impl ProductDiscovery for DrawioProcessor {
    fn description(&self) -> &str {
        "Convert Draw.io diagrams to PNG/SVG/PDF"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.drawio_bin.clone()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config.scan,
            extra_inputs: &self.config.extra_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: crate::processors::names::DRAWIO,
        };
        super::discover_multi_format(graph, file_index, &params, &self.config.formats)
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.outputs.first()
            .context("drawio product has no output")?;

        let format = output.extension()
            .context("drawio output has no extension")?
            .to_string_lossy();

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create drawio output directory: {}", parent.display()))?;
        }

        let mut cmd = Command::new(&self.config.drawio_bin);
        cmd.arg("--export");
        cmd.arg("--format").arg(format.as_ref());
        cmd.arg("--output").arg(output);
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        cmd.arg(input);

        let out = run_command(&mut cmd)?;
        check_command_output(&out, format_args!("drawio {}", input.display()))
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::DRAWIO, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
