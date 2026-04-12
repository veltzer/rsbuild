//! In-process C/C++ dependency analyzer (`icpp`).
//!
//! Uses a pure-Rust regex scanner to find `#include` directives — no external tools.
//! For projects that need compiler-accurate scanning (macros, conditional includes),
//! use the `cpp` analyzer instead.

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::config::IcppAnalyzerConfig;
use crate::errors;
use crate::deps_cache::DepsCache;
use crate::file_index::FileIndex;
use crate::graph::BuildGraph;

use super::DepAnalyzer;

/// In-process C/C++ dependency analyzer using a pure-Rust regex scanner.
pub struct IcppDepAnalyzer {
    config: IcppAnalyzerConfig,
    #[allow(dead_code)]
    verbose: bool,
}

impl IcppDepAnalyzer {
    pub fn new(config: IcppAnalyzerConfig, verbose: bool) -> Self {
        Self { config, verbose }
    }

    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config.src_exclude_dirs.iter().any(|seg| path_str.contains(seg))
    }

    /// Resolve a single `#include` directive to a project-local file, if any.
    /// Searches relative to the including file's directory, then configured include_paths.
    fn resolve_include(&self, include: &str, including_dir: &Path) -> Option<PathBuf> {
        let candidate = including_dir.join(include);
        if candidate.is_file() {
            return Some(candidate);
        }
        for inc_dir in &self.config.include_paths {
            let candidate = Path::new(inc_dir).join(include);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
        None
    }

    /// Scan a single file for `#include` directives. Returns resolved dep paths.
    /// Errors if a `"quoted"` include can't be resolved (system headers via `<angle>`
    /// are allowed to be unresolved — they may live in system include paths).
    fn scan_file_includes(&self, source: &Path) -> Result<Vec<PathBuf>> {
        let content = errors::ctx(fs::read_to_string(source), &format!("Failed to read {}", source.display()))?;

        static INCLUDE_RE: OnceLock<Regex> = OnceLock::new();
        let re = INCLUDE_RE.get_or_init(|| {
            // Capture group 1: opening delimiter ("" or "<"), group 2: include path
            Regex::new(r#"^\s*#\s*include\s*(["<])([^>"]+)[>"]"#).expect(errors::INVALID_REGEX)
        });

        let parent = source.parent().unwrap_or(Path::new(""));
        let mut deps = Vec::new();
        for line in content.lines() {
            if let Some(caps) = re.captures(line) {
                let is_quoted = &caps[1] == "\"";
                let include = &caps[2];
                match self.resolve_include(include, parent) {
                    Some(resolved) => deps.push(resolved),
                    None if is_quoted => {
                        anyhow::bail!(
                            "Include not found: #include \"{}\" in {}",
                            include, source.display()
                        );
                    }
                    None => {} // <angle> include not resolved — likely a system header
                }
            }
        }
        Ok(deps)
    }

    /// Recursively scan `source` for transitive includes. Returns the full set
    /// of project-local header files it depends on (excluding the source itself).
    /// Propagates errors from `scan_file_includes` (including "Include not found").
    fn scan_includes(&self, source: &Path) -> Result<Vec<PathBuf>> {
        let mut seen: HashSet<PathBuf> = HashSet::new();
        let mut headers: Vec<PathBuf> = Vec::new();
        let mut queue: Vec<PathBuf> = vec![source.to_path_buf()];

        while let Some(file) = queue.pop() {
            let direct_deps = self.scan_file_includes(&file)?;
            for dep in direct_deps {
                if seen.insert(dep.clone()) {
                    headers.push(dep.clone());
                    queue.push(dep);
                }
            }
        }

        Ok(headers)
    }
}

impl DepAnalyzer for IcppDepAnalyzer {
    fn description(&self) -> &str {
        "Scan C/C++ source files for #include dependencies (in-process, regex-based)"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        let extensions = [".c", ".cc", ".cpp", ".cxx", ".h", ".hh", ".hpp", ".hxx"];
        for ext in extensions {
            if file_index.has_extension(ext) {
                return true;
            }
        }
        false
    }

    fn analyze(&self, graph: &mut BuildGraph, deps_cache: &mut DepsCache, _file_index: &FileIndex, verbose: bool) -> Result<()> {
        let cpp_extensions: HashSet<&str> = [".c", ".cc", ".cpp", ".cxx"].iter().copied().collect();

        super::analyze_with_scanner(
            graph,
            deps_cache,
            "icpp",
            |p| {
                if p.inputs.is_empty() {
                    return None;
                }
                let source = &p.inputs[0];
                if self.is_excluded(source) {
                    return None;
                }
                let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
                let ext_with_dot = format!(".{}", ext);
                if cpp_extensions.contains(ext_with_dot.as_str()) {
                    Some(source.clone())
                } else {
                    None
                }
            },
            |source| self.scan_includes(source),
            verbose,
        )
    }
}

inventory::submit! {
    crate::registry::AnalyzerPlugin {
        name: "icpp",
        description: "Scan C/C++ source files for #include dependencies (in-process, regex-based)",
        is_native: true,
        create: |toml_value, verbose| {
            let cfg: IcppAnalyzerConfig = toml::from_str(&toml::to_string(toml_value)?)?;
            Ok(Box::new(IcppDepAnalyzer::new(cfg, verbose)))
        },
        defconfig_toml: || {
            toml::to_string_pretty(&crate::config::IcppAnalyzerConfig::default()).ok()
        },
    }
}
