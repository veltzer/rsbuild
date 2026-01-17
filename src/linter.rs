use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::checksum::ChecksumCache;
use crate::config::LintConfig;
use crate::processor::{Processable, ProcessStats, Processor};

const LINT_STUB_DIR: &str = "out/lint";

/// Represents a single Python file to be linted
pub struct LintItem {
    /// Path to the Python file
    source_path: PathBuf,
    /// Path to the stub file marking successful lint
    stub_path: PathBuf,
    /// Linter command to use
    linter: String,
    /// Additional linter arguments
    linter_args: Vec<String>,
    /// Project root for relative path display
    project_root: PathBuf,
}

impl LintItem {
    pub fn new(
        source_path: PathBuf,
        stub_dir: &Path,
        project_root: &Path,
        linter: &str,
        linter_args: &[String],
    ) -> Self {
        // Create stub path preserving directory structure
        let relative_path = source_path
            .strip_prefix(project_root)
            .unwrap_or(&source_path);
        let stub_name = format!(
            "{}.lint",
            relative_path.display().to_string().replace(['/', '\\'], "_")
        );
        let stub_path = stub_dir.join(stub_name);

        Self {
            source_path,
            stub_path,
            linter: linter.to_string(),
            linter_args: linter_args.to_vec(),
            project_root: project_root.to_path_buf(),
        }
    }

    /// Run the linter on this file
    fn lint(&self) -> Result<()> {
        let mut cmd = Command::new(&self.linter);

        // Add check mode for ruff (don't auto-fix)
        if self.linter == "ruff" {
            cmd.arg("check");
        }

        // Add any configured arguments
        for arg in &self.linter_args {
            cmd.arg(arg);
        }

        cmd.arg(&self.source_path);
        cmd.current_dir(&self.project_root);

        let output = cmd
            .output()
            .context(format!("Failed to run linter: {}", self.linter))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow::anyhow!(
                "Linting failed:\n{}{}",
                stdout,
                stderr
            ));
        }

        // Create stub file on success
        if let Some(parent) = self.stub_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&self.stub_path, "linted").context("Failed to create lint stub file")?;

        Ok(())
    }
}

impl Processable for LintItem {
    fn source_path(&self) -> &Path {
        &self.source_path
    }

    fn cache_key(&self) -> String {
        let relative_path = self
            .source_path
            .strip_prefix(&self.project_root)
            .unwrap_or(&self.source_path);
        format!("lint:{}", relative_path.display())
    }

    fn display_name(&self) -> String {
        self.source_path
            .strip_prefix(&self.project_root)
            .unwrap_or(&self.source_path)
            .display()
            .to_string()
    }

    fn process(&self) -> Result<()> {
        self.lint()
    }
}

pub struct Linter {
    project_root: PathBuf,
    lint_config: LintConfig,
    stub_dir: PathBuf,
    processor: Processor,
}

impl Linter {
    pub fn new(project_root: PathBuf, lint_config: LintConfig) -> Self {
        let stub_dir = project_root.join(LINT_STUB_DIR);
        let processor = Processor::new("lint");
        Self {
            project_root,
            lint_config,
            stub_dir,
            processor,
        }
    }

    /// Check if linting should be enabled for this project
    pub fn should_lint(&self) -> bool {
        let pyproject_exists = self.project_root.join("pyproject.toml").exists();
        let tests_dir = self.project_root.join("tests");
        let tests_has_python = tests_dir.exists() && self.has_python_files(&tests_dir);

        pyproject_exists || tests_has_python
    }

    /// Lint all Python files, respecting checksums for incremental builds
    pub fn lint_all(
        &self,
        cache: &mut ChecksumCache,
        force: bool,
        verbose: bool,
    ) -> Result<ProcessStats> {
        if !self.should_lint() {
            return Ok(ProcessStats::new("lint"));
        }

        // Ensure stub directory exists
        if !self.stub_dir.exists() {
            fs::create_dir_all(&self.stub_dir)
                .context("Failed to create lint stub directory")?;
        }

        // Collect all lint items
        let items = self.find_python_files();

        // Process all files using the unified processor
        self.processor.process_all(&items, cache, force, verbose)
    }

    /// Find all Python files that should be linted
    fn find_python_files(&self) -> Vec<LintItem> {
        let paths = if self.project_root.join("pyproject.toml").exists() {
            self.find_py_files_in_project()
        } else {
            let tests_dir = self.project_root.join("tests");
            if tests_dir.exists() {
                self.find_py_files_in_dir(&tests_dir)
            } else {
                Vec::new()
            }
        };

        paths
            .into_iter()
            .map(|path| {
                LintItem::new(
                    path,
                    &self.stub_dir,
                    &self.project_root,
                    &self.lint_config.linter,
                    &self.lint_config.args,
                )
            })
            .collect()
    }

    /// Clean all lint stub files
    pub fn clean(&self) -> Result<()> {
        if self.stub_dir.exists() {
            fs::remove_dir_all(&self.stub_dir)
                .context("Failed to remove lint stub directory")?;
            println!("Removed lint stub directory: {}", self.stub_dir.display());
        }
        Ok(())
    }

    fn has_python_files(&self, dir: &Path) -> bool {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .any(|e| e.path().extension().and_then(|s| s.to_str()) == Some("py"))
    }

    fn find_py_files_in_dir(&self, dir: &Path) -> Vec<PathBuf> {
        WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("py"))
            .map(|e| e.path().to_path_buf())
            .collect()
    }

    fn find_py_files_in_project(&self) -> Vec<PathBuf> {
        WalkDir::new(&self.project_root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();

                // Skip common non-source directories
                let path_str = path.to_string_lossy();
                if path_str.contains("/.venv/")
                    || path_str.contains("/__pycache__/")
                    || path_str.contains("/.git/")
                    || path_str.contains("/out/")
                    || path_str.contains("/node_modules/")
                    || path_str.contains("/.tox/")
                    || path_str.contains("/build/")
                    || path_str.contains("/dist/")
                    || path_str.contains("/.eggs/")
                {
                    return false;
                }

                path.extension().and_then(|s| s.to_str()) == Some("py")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    }
}
