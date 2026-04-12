//! Analyzer plugin registry.
//!
//! Every dependency analyzer (cpp, icpp, python, markdown, tera) submits an
//! [`AnalyzerPlugin`] entry via `inventory::submit!`. Analyzers are separate
//! from processors — they run after product discovery to add dependency edges
//! (e.g. C/C++ `#include` graphs, Python imports).

use anyhow::Result;

use crate::analyzers::DepAnalyzer;

/// An analyzer plugin. Each analyzer file submits one via `inventory::submit!`.
pub struct AnalyzerPlugin {
    pub name: &'static str,
    pub description: &'static str,
    pub is_native: bool,
    /// Create an analyzer from its TOML config section.
    /// Receives the instance name (iname) for cache-tagging, the raw
    /// `[analyzer.IN]` table value, and a verbose flag.
    pub create: fn(&str, &toml::Value, bool) -> Result<Box<dyn DepAnalyzer>>,
    /// Return the default config as a TOML string, or None if the analyzer has no config.
    pub defconfig_toml: fn() -> Option<String>,
}

unsafe impl Sync for AnalyzerPlugin {}

inventory::collect!(AnalyzerPlugin);

pub(crate) fn all_analyzer_plugins() -> impl Iterator<Item = &'static AnalyzerPlugin> {
    inventory::iter::<AnalyzerPlugin>.into_iter()
}

/// Return sorted analyzer names from the registry.
pub(crate) fn all_analyzer_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = all_analyzer_plugins().map(|p| p.name).collect();
    names.sort();
    names
}

/// Find an analyzer plugin by name.
pub(crate) fn find_analyzer_plugin(name: &str) -> Option<&'static AnalyzerPlugin> {
    all_analyzer_plugins().find(|p| p.name == name)
}

/// Build a clap value parser that accepts any registered analyzer name.
pub(crate) fn analyzer_name_parser() -> clap::builder::PossibleValuesParser {
    clap::builder::PossibleValuesParser::new(all_analyzer_names())
}
