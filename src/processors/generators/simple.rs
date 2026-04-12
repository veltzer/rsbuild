use anyhow::Result;

use crate::config::StandardConfig;
use crate::file_index::FileIndex;
use crate::graph::{BuildGraph, Product};
use crate::processors::{ProcessorBase, Processor, ProcessorType};

use super::DiscoverParams;

/// How a simple generator discovers its products.
#[derive(Copy, Clone)]
pub(crate) enum DiscoverMode {
    /// Discover one product per source x format (uses config.formats).
    MultiFormat,
    /// Discover one product per source file with a fixed output extension.
    SingleFormat(&'static str),
}

/// Data-driven generator processor. Replaces identical boilerplate across
/// generators that use StandardConfig with standard discover logic.
pub struct SimpleGenerator {
    base: ProcessorBase,
    config: StandardConfig,
    params: SimpleGeneratorParams,
}

#[derive(Copy, Clone)]
pub(crate) struct SimpleGeneratorParams {
    pub description: &'static str,
    pub extra_tools: &'static [&'static str],
    pub discover_mode: DiscoverMode,
    pub execute_fn: fn(&StandardConfig, &Product) -> Result<()>,
    pub is_native: bool,
}

impl SimpleGenerator {
    pub fn new(config: StandardConfig, params: SimpleGeneratorParams) -> Self {
        Self {
            base: ProcessorBase::generator("", params.description),
            config,
            params,
        }
    }
}

impl Processor for SimpleGenerator {
    fn scan_config(&self) -> &crate::config::StandardConfig {
        &self.config
    }

    fn standard_config(&self) -> Option<&StandardConfig> {
        Some(&self.config)
    }

    fn description(&self) -> &str {
        self.base.description()
    }

    fn processor_type(&self) -> ProcessorType {
        self.base.processor_type()
    }

    fn config_json(&self) -> Option<String> {
        ProcessorBase::config_json(&self.config)
    }

    fn clean(&self, product: &Product, verbose: bool) -> Result<usize> {
        ProcessorBase::clean(product, &product.processor, verbose)
    }

    fn is_native(&self) -> bool {
        self.params.is_native
    }

    fn required_tools(&self) -> Vec<String> {
        if self.params.is_native {
            self.params.extra_tools.iter().map(|t| t.to_string()).collect()
        } else {
            let mut tools = vec![self.config.command.clone()];
            for t in self.params.extra_tools {
                tools.push(t.to_string());
            }
            tools
        }
    }

    fn max_jobs(&self) -> Option<usize> {
        self.config.max_jobs
    }

    fn discover(&self, graph: &mut BuildGraph, file_index: &FileIndex, instance_name: &str) -> Result<()> {
        let params = DiscoverParams {
            scan: &self.config,
            dep_inputs: &self.config.dep_inputs,
            config: &self.config,
            output_dir: &self.config.output_dir,
            processor_name: instance_name,
        };
        match &self.params.discover_mode {
            DiscoverMode::MultiFormat => {
                super::discover_multi_format(graph, file_index, &params, &self.config.formats)
            }
            DiscoverMode::SingleFormat(ext) => {
                super::discover_single_format(graph, file_index, &params, ext)
            }
        }
    }

    fn supports_batch(&self) -> bool { false }

    fn execute(&self, product: &Product) -> Result<()> {
        (self.params.execute_fn)(&self.config, product)
    }
}
