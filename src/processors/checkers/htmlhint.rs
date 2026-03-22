use anyhow::Result;
use std::path::Path;

use crate::config::HtmlhintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct HtmlhintProcessor {
    config: HtmlhintConfig,
}

impl HtmlhintProcessor {
    pub fn new(config: HtmlhintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, None, &self.config.args, files)
    }
}

impl_checker!(HtmlhintProcessor,
    config: config,
    description: "Lint HTML files with htmlhint",
    name: crate::processors::names::HTMLHINT,
    execute: execute_product,
    tool_field_extra: linter ["node".to_string()],
    config_json: true,
    batch: lint_files,
);
