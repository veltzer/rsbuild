use anyhow::Result;

use crate::config::CheckpatchConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct CheckpatchProcessor {
    config: CheckpatchConfig,
}

impl CheckpatchProcessor {
    pub fn new(config: CheckpatchConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        let mut args = vec![
            "--no-tree".to_string(),
            "-f".to_string(),
        ];
        args.extend(self.config.args.iter().cloned());
        run_checker("checkpatch.pl", None, &args, &[product.primary_input()])
    }
}

impl_checker!(CheckpatchProcessor,
    config: config,
    description: "Run kernel checkpatch.pl on C source files",
    name: crate::processors::names::CHECKPATCH,
    execute: execute_product,
    guard: scan_root,
    tools: ["checkpatch.pl".to_string(), "perl".to_string()],
    config_json: true,
);
