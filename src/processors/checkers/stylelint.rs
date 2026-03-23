use anyhow::Result;
use std::path::Path;

use crate::config::StylelintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct StylelintProcessor {
    config: StylelintConfig,
}

impl StylelintProcessor {
    pub fn new(config: StylelintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker(&self.config.linter, None, &self.config.args, files)
    }
}

impl_checker!(StylelintProcessor,
    config: config,
    description: "Lint CSS/SCSS files with stylelint",
    name: crate::processors::names::STYLELINT,
    execute: execute_product,
    tool_field_extra: linter ["node".to_string()],
    config_json: true,
    batch: lint_files,
);
