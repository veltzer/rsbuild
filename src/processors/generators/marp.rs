use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::config::{MarpConfig, config_hash, resolve_extra_inputs};
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProductDiscovery, ProcessorType, clean_outputs, scan_root_valid, run_command, check_command_output};

/// Remove all marp-cli-* temp directories from /tmp.
/// Chromium/Puppeteer creates these as user-data-dirs during PDF/image conversion
/// and marp-cli does not clean them up.
fn cleanup_marp_tmp_dirs() {
    let Ok(entries) = fs::read_dir("/tmp") else { return };
    for entry in entries.filter_map(|e| e.ok()) {
        if entry.file_name().to_string_lossy().starts_with("marp-cli-") {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}

pub struct MarpProcessor {
    config: MarpConfig,
}

impl MarpProcessor {
    pub fn new(config: MarpConfig) -> Self {
        Self { config }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    fn output_path(&self, source: &Path, format: &str) -> PathBuf {
        let stem = source.file_stem().unwrap_or_default();
        let parent = source.parent().unwrap_or(Path::new(""));
        let output_name = format!("{}.{}", stem.to_string_lossy(), format);
        Path::new(&self.config.output_dir).join(format).join(parent).join(output_name)
    }
}

impl ProductDiscovery for MarpProcessor {
    fn description(&self) -> &str {
        "Convert Marp slides to PDF/PPTX/HTML"
    }

    fn processor_type(&self) -> ProcessorType {
        ProcessorType::Generator
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        self.should_process() && !file_index.scan(&self.config.scan, true).is_empty()
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.marp_bin.clone(), "node".to_string()]
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex) -> Result<()> {
        if !self.should_process() {
            return Ok(());
        }

        let files = file_index.scan(&self.config.scan, true);
        if files.is_empty() {
            return Ok(());
        }

        let hash = Some(config_hash(&self.config));
        let extra = resolve_extra_inputs(&self.config.extra_inputs)?;

        for source in &files {
            for format in &self.config.formats {
                let output = self.output_path(source, format);

                let mut inputs = Vec::with_capacity(1 + extra.len());
                inputs.push(source.clone());
                inputs.extend_from_slice(&extra);

                graph.add_product(inputs, vec![output], crate::processors::names::MARP, hash.clone())?;
            }
        }

        Ok(())
    }

    fn execute(&self, product: &Product) -> Result<()> {
        let input = product.primary_input();
        let output = product.outputs.first()
            .context("marp product has no output")?;

        let format = output.extension()
            .context("marp output has no extension")?
            .to_string_lossy();

        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create marp output directory: {}", parent.display()))?;
        }

        let mut cmd = Command::new(&self.config.marp_bin);
        // HTML is marp's default output, so no format flag needed for it
        if format != "html" {
            cmd.arg(format!("--{}", format));
        }
        cmd.arg("--output").arg(output);
        for arg in &self.config.args {
            cmd.arg(arg);
        }
        cmd.arg(input);

        let out = run_command(&mut cmd)?;
        let result = check_command_output(&out, format_args!("marp {}", input.display()));

        cleanup_marp_tmp_dirs();

        result
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        clean_outputs(product, crate::processors::names::MARP, verbose)
    }

    fn config_json(&self) -> Option<String> {
        serde_json::to_string(&self.config).ok()
    }
}
