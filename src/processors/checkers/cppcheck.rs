use anyhow::Result;
use std::path::PathBuf;

use crate::config::CppcheckConfig;
use crate::graph::Product;
use crate::processors::{scan_root_valid, run_checker};

pub struct CppcheckProcessor {
    project_root: PathBuf,
    config: CppcheckConfig,
}

impl CppcheckProcessor {
    pub fn new(project_root: PathBuf, config: CppcheckConfig) -> Self {
        Self {
            project_root,
            config,
        }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        run_checker("cppcheck", None, &self.config.args, &[product.primary_input()], &self.project_root)
    }
}

impl_checker!(CppcheckProcessor,
    config: config,
    description: "Run cppcheck static analysis on C/C++ source files",
    name: "cppcheck",
    execute: execute_product,
    guard: should_process,
    tools: ["cppcheck".to_string()],
    config_json: true,
);
