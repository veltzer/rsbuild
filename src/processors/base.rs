use serde::Serialize;
use crate::graph::Product;
use crate::processors::ProcessorType;

/// Common base for all processors. Holds fields needed by boilerplate
/// Processor methods so each processor doesn't repeat them.
pub struct ProcessorBase {
    /// Human-readable description
    pub description: &'static str,
    /// Generator or Checker
    pub processor_type: ProcessorType,
}

impl ProcessorBase {
    pub fn generator(_name: &'static str, description: &'static str) -> Self {
        Self { description, processor_type: ProcessorType::Generator }
    }

    pub fn creator(_name: &'static str, description: &'static str) -> Self {
        Self { description, processor_type: ProcessorType::Creator }
    }

    pub fn checker(_name: &'static str, description: &'static str) -> Self {
        Self { description, processor_type: ProcessorType::Checker }
    }

    pub fn explicit(_name: &'static str, description: &'static str) -> Self {
        Self { description, processor_type: ProcessorType::Explicit }
    }

    pub fn description(&self) -> &str {
        self.description
    }

    pub fn processor_type(&self) -> ProcessorType {
        self.processor_type
    }

    pub fn config_json<C: Serialize>(config: &C) -> Option<String> {
        serde_json::to_string(config).ok()
    }

    pub fn clean(product: &Product, name: &str, verbose: bool) -> anyhow::Result<usize> {
        crate::processors::clean_outputs(product, name, verbose)
    }

    pub fn clean_output_dir(product: &Product, name: &str, verbose: bool) -> anyhow::Result<usize> {
        crate::processors::clean_output_dir(product, name, verbose)
    }
}
