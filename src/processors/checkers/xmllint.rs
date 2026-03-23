use anyhow::Result;
use std::path::Path;

use crate::config::XmllintConfig;
use crate::graph::Product;
use crate::processors::run_checker;

pub struct XmllintProcessor {
    config: XmllintConfig,
}

impl XmllintProcessor {
    pub fn new(config: XmllintConfig) -> Self {
        Self { config }
    }

    fn execute_product(&self, product: &Product) -> Result<()> {
        self.lint_files(&[product.primary_input()])
    }

    fn lint_files(&self, files: &[&Path]) -> Result<()> {
        run_checker("xmllint", Some("--noout"), &self.config.args, files)
    }
}

impl_checker!(XmllintProcessor,
    config: config,
    description: "Validate XML files with xmllint",
    name: crate::processors::names::XMLLINT,
    execute: execute_product,
    tools: ["xmllint".to_string()],
    config_json: true,
    batch: lint_files,
);
