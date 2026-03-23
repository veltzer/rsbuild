use anyhow::Result;
use std::path::Path;

use crate::config::HtmllintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct HtmllintProcessor {
    config: HtmllintConfig,
}

impl HtmllintProcessor {
    pub fn new(config: HtmllintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("htmllint", None, &self.config.args, files)
    }
}

impl_checker!(HtmllintProcessor,
    config: config,
    description: "Lint HTML files with htmllint",
    name: crate::processors::names::HTMLLINT,
    execute: execute_product,
    tools: ["htmllint".to_string(), "node".to_string()],
    config_json: true,
    batch: lint_files,
);
