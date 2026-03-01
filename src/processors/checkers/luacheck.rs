use anyhow::Result;
use std::path::Path;

use crate::config::LuacheckConfig;
use crate::graph::Product;
use crate::processors::{scan_root_valid, run_checker};

pub struct LuacheckProcessor {
    config: LuacheckConfig,
}

impl LuacheckProcessor {
    pub fn new(config: LuacheckConfig) -> Self {
        Self { config }
    }

    fn should_process(&self) -> bool {
        scan_root_valid(&self.config.scan)
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    /// Run luacheck on one or more files
    fn check_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.checker, None, &self.config.args, files)
    }
}

impl_checker!(LuacheckProcessor,
    config: config,
    description: "Lint Lua scripts using luacheck",
    name: crate::processors::names::LUACHECK,
    execute: execute_product,
    guard: should_process,
    tool_field: checker,
    config_json: true,
    batch: check_files,
);
