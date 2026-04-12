//! Plugin registries.
//!
//! rsconstruct has two independent plugin registries collected via the `inventory`
//! crate at link time:
//!
//! - [`processor`] — [`ProcessorPlugin`](processor::ProcessorPlugin) entries for
//!   every built-in processor (checkers, generators, creators, mass-generators).
//! - [`analyzer`] — [`AnalyzerPlugin`](analyzer::AnalyzerPlugin) entries for
//!   every dependency analyzer.
//!
//! Both submodules are re-exported here so callers can keep using
//! `crate::registries::X` without caring which registry `X` belongs to.

pub(crate) mod analyzer;
pub(crate) mod processor;

pub use analyzer::*;
pub use processor::*;
