use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{PipConfig, config_hash, resolve_extra_inputs};
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, scan_root_valid, run_in_anchor_dir, anchor_display_dir, check_command_output};

pub struct PipProcessor {
    config: PipConfig,
    output_dir: PathBuf,
}

impl PipProcessor {
    pub fn new(config: PipConfig) -> Self {
        Self {
            config,
            output_dir: PathBuf::from("out/pip"),
        }
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

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::MassGenerator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        scan_root_valid(&self.config.scan) && !file_index.scan(&self.config.scan, false).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.pip.clone(), "python3".to_string()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        if !scan_root_valid(&self.config.scan) {
            return Ok(());
        }

        let files = file_index.scan(&self.config.scan, false);
        if files.is_empty() {
            return Ok(());
        }

        let hash = Some(config_hash(&self.config));
        let extra = resolve_extra_inputs(&self.config.extra_inputs)?;

        for anchor in files {
            let stamp = super::stamp_path(&self.output_dir, &anchor);

            let mut inputs: Vec<PathBuf> = Vec::with_capacity(1 + extra.len());
            inputs.push(anchor.clone());
            inputs.extend_from_slice(&extra);

            if self.config.cache_output_dir {
                let anchor_dir = anchor.parent().unwrap_or(Path::new(""));
                let output_dir = if anchor_dir.as_os_str().is_empty() {
                    self.output_dir.clone()
                } else {
                    anchor_dir.join(&self.output_dir)
                };
                graph.add_product_with_output_dir(inputs, vec![stamp], crate::processors::names::PIP, hash.clone(), output_dir)?;
            } else {
                graph.add_product(inputs, vec![stamp], crate::processors::names::PIP, hash.clone())?;
            }
        }

        Ok(())
    }

    fn execute(&self, product: &Product) -> Result<()> {
        self.execute_pip(product.primary_input())?;
        super::write_stamp(product, "pip")
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        super::clean_stamp_and_output_dir(product, "pip", verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
