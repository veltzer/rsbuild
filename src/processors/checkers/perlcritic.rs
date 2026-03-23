use anyhow::Result;
use std::path::Path;

use crate::config::PerlcriticConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct PerlcriticProcessor {
    config: PerlcriticConfig,
}

impl PerlcriticProcessor {
    pub fn new(config: PerlcriticConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("perlcritic", None, &self.config.args, files)
    }
}

impl_checker!(PerlcriticProcessor,
    config: config,
    description: "Analyze Perl code with perlcritic",
    name: crate::processors::names::PERLCRITIC,
    execute: execute_product,
    tools: ["perlcritic".to_string(), "perl".to_string()],
    config_json: true,
    batch: lint_files,
);
