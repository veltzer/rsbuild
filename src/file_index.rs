use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use crate::config::ScanConfig;

pub struct FileIndex {
    files: Vec<PathBuf>,
}

impl FileIndex {
    /// Build a file index by walking the project root once.
    /// Uses `ignore::WalkBuilder` which natively handles `.gitignore` and
    /// `.rsbignore` (via `add_custom_ignore_filename`).
    pub fn build(project_root: &Path) -> Result<Self> {
        let walker = ignore::WalkBuilder::new(project_root)
            .add_custom_ignore_filename(".rsbignore")
            .hidden(false) // don't skip hidden files by default (let .gitignore handle it)
            .build();

        let mut files: Vec<PathBuf> = Vec::new();
        for entry in walker {
            let entry = entry.context("Failed to read directory entry during file indexing")?;
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                files.push(entry.into_path());
            }
        }
        files.sort();

        Ok(Self { files })
    }

    /// Query the index for files matching the given criteria.
    ///
    /// - `root`: only include files under this directory
    /// - `extensions`: file extensions to match (e.g., `[".py", ".pyi"]`)
    /// - `exclude_dirs`: directory path segments to skip (e.g., `["/.git/", "/out/"]`)
    /// - `exclude_files`: file names to skip (e.g., `["setup.py"]`)
    /// - `exclude_paths`: paths relative to project root to skip (e.g., `["Makefile"]`)
    /// - `project_root`: the project root, used to compute relative paths for `exclude_paths`
    pub fn query(
        &self,
        root: &Path,
        extensions: &[&str],
        exclude_dirs: &[&str],
        exclude_files: &[&str],
        exclude_paths: &[&str],
        project_root: &Path,
    ) -> Vec<PathBuf> {
        self.files
            .iter()
            .filter(|path| {
                // Must be under root
                if !path.starts_with(root) {
                    return false;
                }

                // Check exclude dirs
                if !exclude_dirs.is_empty() {
                    let path_str = path.to_string_lossy();
                    if exclude_dirs.iter().any(|dir| path_str.contains(dir)) {
                        return false;
                    }
                }

                // Check extension match
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !extensions.iter().any(|ext| name.ends_with(ext)) {
                    return false;
                }

                // Check exclude files
                if !exclude_files.is_empty() && exclude_files.iter().any(|f| *f == name) {
                    return false;
                }

                // Check exclude paths (relative to project root)
                if !exclude_paths.is_empty() {
                    if let Ok(rel) = path.strip_prefix(project_root) {
                        let rel_str = rel.to_string_lossy();
                        if exclude_paths.iter().any(|p| *p == rel_str) {
                            return false;
                        }
                    }
                }

                true
            })
            .cloned()
            .collect()
    }

    /// Convenience wrapper using `ScanConfig` fields.
    ///
    /// - `project_root`: the project root directory
    /// - `scan`: processor scan configuration
    /// - `recursive`: if false, only include files at depth 1 from the scan root
    pub fn scan(
        &self,
        project_root: &Path,
        scan: &ScanConfig,
        recursive: bool,
    ) -> Vec<PathBuf> {
        let dir = scan.scan_dir();
        let root = if dir.is_empty() {
            project_root.to_path_buf()
        } else {
            project_root.join(dir)
        };
        let ext_refs: Vec<&str> = scan.extensions().iter().map(|s| s.as_str()).collect();
        let exclude_dir_refs: Vec<&str> = scan.exclude_dirs().iter().map(|s| s.as_str()).collect();
        let exclude_file_refs: Vec<&str> = scan.exclude_files().iter().map(|s| s.as_str()).collect();
        let exclude_path_refs: Vec<&str> = scan.exclude_paths().iter().map(|s| s.as_str()).collect();
        let mut results = self.query(&root, &ext_refs, &exclude_dir_refs, &exclude_file_refs, &exclude_path_refs, project_root);

        if !recursive {
            // Filter to max_depth=1 from scan root: only files directly in root
            results.retain(|path| {
                path.parent() == Some(root.as_path())
            });
        }

        results
    }
}
