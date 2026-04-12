//! C/C++ dependency analyzer for scanning header files.
//!
//! Scans source files for #include directives and adds header dependencies
//! to products in the build graph.

use anyhow::Result;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::config::CppAnalyzerConfig;
use crate::deps_cache::DepsCache;
use crate::file_index::FileIndex;
use crate::graph::BuildGraph;
use crate::processors::{check_command_output, format_command, run_command_capture};

use super::DepAnalyzer;

/// C/C++ dependency analyzer that scans source files for #include directives.
pub struct CppDepAnalyzer {
    config: CppAnalyzerConfig,
    verbose: bool,
    /// Cached canonical project root path (for stripping absolute prefixes from compiler output)
    canonical_root: OnceLock<PathBuf>,
    /// Cached include paths from pkg-config
    pkg_config_include_paths: OnceLock<Vec<PathBuf>>,
    /// Cached include paths from include_path_commands
    command_include_paths: OnceLock<Vec<PathBuf>>,
}

impl CppDepAnalyzer {
    pub fn new(config: CppAnalyzerConfig, verbose: bool) -> Self {
        Self {
            config,
            verbose,
            canonical_root: OnceLock::new(),
            pkg_config_include_paths: OnceLock::new(),
            command_include_paths: OnceLock::new(),
        }
    }

    /// Get the canonical project root path (lazily computed).
    fn canonical_root(&self) -> &Path {
        self.canonical_root.get_or_init(|| {
            Path::new(".").canonicalize().unwrap_or_else(|_| PathBuf::from("."))
        })
    }

    /// Query pkg-config for include paths from configured packages.
    /// Uses `pkg-config --cflags-only-I` and strips the -I prefix.
    fn get_pkg_config_include_paths(&self) -> &[PathBuf] {
        self.pkg_config_include_paths.get_or_init(|| {
            if self.config.pkg_config.is_empty() {
                return Vec::new();
            }

            let mut cmd = Command::new("pkg-config");
            cmd.arg("--cflags-only-I");
            cmd.args(&self.config.pkg_config);

            if self.verbose {
                eprintln!("[cpp] Querying pkg-config: {}", format_command(&cmd));
            }

            let output = match run_command_capture(&mut cmd) {
                Ok(o) => o,
                Err(e) => {
                    eprintln!("[cpp] Failed to query pkg-config: {}", e);
                    return Vec::new();
                }
            };

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                eprintln!("[cpp] pkg-config failed: {}", stderr.trim());
                return Vec::new();
            }

            let stdout = String::from_utf8_lossy(&output.stdout);
            let paths: Vec<PathBuf> = stdout
                .split_whitespace()
                .filter_map(|flag| {
                    // Strip -I prefix
                    flag.strip_prefix("-I").map(PathBuf::from)
                })
                .collect();

            if self.verbose && !paths.is_empty() {
                eprintln!("[cpp] Found {} include paths from pkg-config", paths.len());
            }

            paths
        })
    }

    /// Run configured include_path_commands and collect their output as include paths.
    /// Each command is executed via `sh -c` and its stdout (trimmed) is added as an include path.
    /// This supports shell syntax like command substitution: "echo $(gcc -print-file-name=plugin)/include"
    fn get_command_include_paths(&self) -> &[PathBuf] {
        self.command_include_paths.get_or_init(|| {
            if self.config.include_path_commands.is_empty() {
                return Vec::new();
            }

            let mut paths = Vec::new();

            for cmd_str in &self.config.include_path_commands {
                if cmd_str.trim().is_empty() {
                    continue;
                }

                // Run via shell to support shell syntax (command substitution, etc.)
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd.arg(cmd_str);

                if self.verbose {
                    eprintln!("[cpp] Running include path command: sh -c '{}'", cmd_str);
                }

                let output = match run_command_capture(&mut cmd) {
                    Ok(o) => o,
                    Err(e) => {
                        eprintln!("[cpp] Failed to run '{}': {}", cmd_str, e);
                        continue;
                    }
                };

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    eprintln!("[cpp] Command '{}' failed: {}", cmd_str, stderr.trim());
                    continue;
                }

                let stdout = String::from_utf8_lossy(&output.stdout);
                let path_str = stdout.trim();

                if path_str.is_empty() {
                    continue;
                }

                let path = PathBuf::from(path_str);

                // Check if the path exists and is a directory
                if path.is_dir() {
                    if self.verbose {
                        eprintln!("[cpp] Added include path from command: {}", path.display());
                    }
                    paths.push(path);
                } else if self.verbose {
                    eprintln!("[cpp] Command output is not a directory: {}", path_str);
                }
            }

            if self.verbose && !paths.is_empty() {
                eprintln!("[cpp] Found {} include paths from commands", paths.len());
            }

            paths
        })
    }

    /// Check if a path is within the project root (not a system header).
    fn is_project_local(&self, path: &Path) -> bool {
        if let Ok(canonical) = path.canonicalize() {
            canonical.starts_with(self.canonical_root())
        } else {
            // If we can't canonicalize, assume it's not project-local
            false
        }
    }

    /// Check if a source path matches any of the configured exclude-dir segments.
    fn is_excluded(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config.src_exclude_dirs.iter().any(|seg| path_str.contains(seg))
    }

    /// Run gcc/g++ -MM to scan dependencies for a source file.
    fn scan_dependencies_compiler(&self, source: &Path, is_cpp: bool) -> Result<Vec<PathBuf>> {
        let compiler = if is_cpp { &self.config.cxx } else { &self.config.cc };

        let mut cmd = Command::new(compiler);
        cmd.arg("-MM");

        // Add include paths
        for inc in &self.config.include_paths {
            cmd.arg(format!("-I{}", inc));
        }

        // Add pkg-config include paths
        for inc in self.get_pkg_config_include_paths() {
            cmd.arg(format!("-I{}", inc.display()));
        }

        // Add include paths from commands
        for inc in self.get_command_include_paths() {
            cmd.arg(format!("-I{}", inc.display()));
        }

        // Add compile flags
        let flags = if is_cpp { &self.config.cxxflags } else { &self.config.cflags };
        for flag in flags {
            cmd.arg(flag);
        }

        cmd.arg(source);

        if self.verbose {
            eprintln!("[cpp] {}", format_command(&cmd));
        }

        let output = run_command_capture(&mut cmd)?;
        check_command_output(&output, format_args!("Dependency scan of {}", source.display()))?;

        let content = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(self.parse_dep_file(&content))
    }

    /// Parse a Makefile-style dependency file (.d) produced by gcc -MM.
    /// Format: target.o: source.c header1.h header2.h \
    ///           header3.h
    /// Returns the list of header files (excludes the source file itself and system headers).
    fn parse_dep_file(&self, content: &str) -> Vec<PathBuf> {
        // Join continuation lines (backslash-newline)
        let joined = content.replace("\\\n", " ");

        // Find the colon separating target from dependencies
        let deps_part = match joined.find(':') {
            Some(pos) => &joined[pos + 1..],
            None => return Vec::new(),
        };

        // Split by whitespace, skip the first token (the source file itself)
        let tokens: Vec<&str> = deps_part.split_whitespace().collect();
        if tokens.is_empty() {
            return Vec::new();
        }

        // First token is the source file; remaining are headers
        let canonical_root = self.canonical_root();
        tokens[1..]
            .iter()
            .filter_map(|token| {
                let path = PathBuf::from(token);

                // For absolute paths, check if they're within the project
                if path.is_absolute() {
                    if self.is_project_local(&path) {
                        // Convert to relative path
                        if let Ok(rel) = path.strip_prefix(canonical_root) {
                            Some(rel.to_path_buf())
                        } else if let Ok(canonical) = path.canonicalize() {
                            canonical.strip_prefix(canonical_root)
                                .ok()
                                .map(|p| p.to_path_buf())
                        } else {
                            None
                        }
                    } else {
                        // System header, skip it
                        None
                    }
                } else {
                    // Relative paths are assumed to be project-local
                    Some(path)
                }
            })
            .collect()
    }

    /// Scan dependencies using compiler -MM method.
    fn scan_dependencies(&self, source: &Path, is_cpp: bool) -> Result<Vec<PathBuf>> {
        self.scan_dependencies_compiler(source, is_cpp)
    }
}

impl DepAnalyzer for CppDepAnalyzer {
    fn description(&self) -> &str {
        "Scan C/C++ source files for #include dependencies"
    }

    fn auto_detect(&self, file_index: &FileIndex) -> bool {
        // Check if there are any C/C++ source files
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
            "cpp",
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
            |source| {
                let ext = source.extension().and_then(|s| s.to_str()).unwrap_or("");
                let is_cpp = ext == "cc" || ext == "cpp" || ext == "cxx";
                self.scan_dependencies(source, is_cpp)
            },
            verbose,
        )
    }
}

inventory::submit! {
    crate::registry::AnalyzerPlugin {
        name: "cpp",
        description: "Scan C/C++ source files for #include dependencies (using compiler -MM)",
        is_native: false,
        create: |toml_value, verbose| {
            let cfg: CppAnalyzerConfig = toml::from_str(&toml::to_string(toml_value)?)?;
            Ok(Box::new(CppDepAnalyzer::new(cfg, verbose)))
        },
        defconfig_toml: || {
            toml::to_string_pretty(&crate::config::CppAnalyzerConfig::default()).ok()
        },
    }
}
