use anyhow::Result;
use std::path::Path;

use crate::config::LuacheckConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct LuacheckProcessor {
    config: LuacheckConfig,
}

impl LuacheckProcessor {
    pub fn new(config: LuacheckConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.check_files(&[product.primary_input()])
    }

    /// Run luacheck on one or more files
    fn check_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.command, None, &self.config.args, files)
    }
}

impl crate::processors::ProductDiscovery for LuacheckProcessor {
    fn scan_config(&self) -> &crate::config::ScanConfig {
        &self.config.scan
    }

    fn standard_config(&self) -> Option<&crate::config::StandardConfig> {
        Some(&self.config)
    }

    fn description(&self) -> &str {
        "Lint Lua scripts using luacheck"
    }

    fn auto_detect(&self, file_index: &crate::file_index::FileIndex) -> bool {
        crate::processors::checker_auto_detect_with_scan_root(&self.config.scan, file_index)
    }

    fn required_tools(&self) -> Vec<String> {
        vec![self.config.command.clone()]
    }

    fn discover(
        &self,
        graph: &mut crate::graph::BuildGraph,
        file_index: &crate::file_index::FileIndex,
        instance_name: &str,
    ) -> anyhow::Result<()> {
        if !crate::processors::scan_root_valid(&self.config.scan) {
            return Ok(());
        }
        crate::processors::checker_discover(
            graph, &self.config.scan, file_index,
            &self.config.dep_inputs, &self.config.dep_auto,
            &self.config, instance_name,
        )
    }

    fn execute(&self, product: &crate::graph::Product) -> anyhow::Result<()> {
        self.execute_product(product)
    }





    fn supports_batch(&self) -> bool { self.config.batch }

    fn execute_batch(&self, products: &[&crate::graph::Product]) -> Vec<anyhow::Result<()>> {
        crate::processors::execute_checker_batch(products, |files| self.check_files(files))
    }
}