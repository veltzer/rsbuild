use anyhow::Result;

use crate::config::CpplintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct CpplintProcessor {
    config: CpplintConfig,
}

impl CpplintProcessor {
    pub fn new(config: CpplintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        run_checker("cpplint", None, &self.config.args, &[product.primary_input()])
    }
}

impl_checker!(CpplintProcessor,
    config: config,
    description: "Run cpplint (Google C++ style checker) on C/C++ source files",
    name: crate::processors::names::CPPLINT,
    execute: execute_product,
    guard: scan_root,
    tools: ["cpplint".to_string()],
    config_json: true,
);
