use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::config::ObjdumpConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command_capture, check_command_output};

use super::DiscoverParams;

pub struct ObjdumpProcessor {
    config: ObjdumpConfig,
}

impl ObjdumpProcessor {
    pub fn new(config: ObjdumpConfig) -> Self {
        Self { config }
    }
}

impl ProductDiscovery for ObjdumpProcessor {
    fn description(&self) -> &str {
        "Disassemble binaries using objdump"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec!["objdump".to_string()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config.scan,
            extra_inputs: &self.config.extra_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: crate::processors::names::OBJDUMP,
        };
        super::discover_single_format(graph, file_index, &params, "dis")
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.primary_output();

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create objdump output directory: {}", parent.display()))?;
        }

        let mut cmd = Command::new("objdump");
        cmd.arg("--disassemble").arg("--source");
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        cmd.arg(input);

        let out = run_command_capture(&mut cmd)?;
        check_command_output(&out, format_args!("objdump {}", input.display()))?;

        fs::write(output, &out.stdout)
            .with_context(|| format!("Failed to write objdump output: {}", output.display()))?;

        Ok(())
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::OBJDUMP, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
