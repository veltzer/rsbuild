//! Markdown dependency analyzer for scanning image and file references.
//!
//! Scans Markdown source files for image references (`![alt](path)`) and
//! adds referenced local files as dependencies to products in the build graph.

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::deps_cache::DepsCache;
use crate::errors;
use crate::file_index::FileIndex;
use crate::graph::BuildGraph;

use super::DepAnalyzer;

/// Markdown dependency analyzer that scans source files for image and link references.
pub struct MarkdownDepAnalyzer;

impl MarkdownDepAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Scan a Markdown file for local file references.
    /// Returns paths to local files referenced via `![alt](path)` or `[text](path)` syntax.
    fn scan_references(&self, source: &Path) -> Result<Vec<PathBuf>> {
        let content = fs::read_to_string(source)?;
        let mut refs = Vec::new();
        let mut seen = HashSet::new();

        // Match ![alt](path) and [text](path) — capture the path portion
        // Excludes URLs (http://, https://, ftp://, data:, #anchors)
        static REF_RE: OnceLock<Regex> = OnceLock::new();
        let ref_re = REF_RE.get_or_init(|| {
            Regex::new(r"!?\[(?:[^\]]*)\]\(([^)]+)\)").expect(errors::INVALID_REGEX)
        });

        let source_dir = source.parent().unwrap_or(Path::new("."));

        for caps in ref_re.captures_iter(&content) {
            let path_str = caps[1].trim();

            // Skip URLs, anchors, and data URIs
            if path_str.starts_with("http://")
                || path_str.starts_with("https://")
                || path_str.starts_with("ftp://")
                || path_str.starts_with("data:")
                || path_str.starts_with('#')
            {
                continue;
            }

            // Strip optional title: ![alt](path "title")
            let path_str = path_str.split_whitespace().next().unwrap_or(path_str);
            // Strip anchor fragments: path#section
            let path_str = path_str.split('#').next().unwrap_or(path_str);

            if path_str.is_empty() {
                continue;
            }

            // Resolve relative to the source file's directory
            let resolved = source_dir.join(path_str);
            if resolved.is_file() && !seen.contains(&resolved) {
                seen.insert(resolved.clone());
                refs.push(resolved);
            }
        }

        Ok(refs)
    }
}

impl DepAnalyzer for MarkdownDepAnalyzer {
    fn description(&self) -> &str {
        "Scan Markdown files for image and file reference dependencies"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        file_index.has_extension(".md")
    }

    fn analyze(&self, graph: &mut BuildGraph, deps_cache: &mut DepsCache, _file_index: &FileIndex, verbose: bool) -> Result<()> {
        super::analyze_with_scanner(
            graph,
            deps_cache,
            "markdown",
            |p| {
                if p.inputs.is_empty() {
                    return None;
                }
                let source = &p.inputs[0];
                let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
                if ext == "md" {
                    Some(source.clone())
                } else {
                    None
                }
            },
            |source| self.scan_references(source),
            verbose,
        )
    }
}
