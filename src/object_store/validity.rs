use std::path::{Path, PathBuf};

use super::{ExplainAction, ObjectStore, RebuildReason};

impl ObjectStore {
    /// Check if a product needs rebuilding
    /// Returns true if inputs changed or outputs are missing
    pub fn needs_rebuild(&self, cache_key: &str, input_checksum: &str, output_paths: &[PathBuf]) -> bool {
        // Check if we have a cache entry
        let entry = match self.get_entry(cache_key) {
            Some(e) => e,
            None => return true,
        };

        // Check if input checksum matches
        if entry.input_checksum != input_checksum {
            return true;
        }

        // For checkers (empty outputs), cache entry with matching checksum = up-to-date
        if output_paths.is_empty() {
            return false;
        }

        // Check if all outputs exist at their original paths
        for output_path in output_paths {
            if !output_path.exists() {
                // Output missing - check if we can restore from cache
                let rel_path = Self::path_string(output_path);
                let cached_output = entry.outputs.iter()
                    .find(|o| o.path == rel_path);

                match cached_output {
                    Some(out) if self.has_object(&out.checksum) => {
                        // Can restore from cache, but still "needs rebuild" to trigger restore
                        return true;
                    }
                    _ => return true,
                }
            }
        }

        false
    }

    /// Check if outputs can be restored from cache (read-only, does not restore)
    /// Returns true if all missing outputs are available in cache
    pub fn can_restore(&self, cache_key: &str, input_checksum: &str, output_paths: &[PathBuf]) -> bool {
        // For checkers (empty outputs), cache entry with matching checksum = restorable
        if output_paths.is_empty() {
            return self.get_entry(cache_key)
                .map(|e| e.input_checksum == input_checksum)
                .unwrap_or(false);
        }

        let entry = match self.get_entry(cache_key) {
            Some(e) if e.input_checksum == input_checksum => e,
            _ => return false,
        };

        for output_path in output_paths {
            if output_path.exists() {
                continue;
            }

            let rel_path = Self::path_string(output_path);
            let cached_output = entry.outputs.iter()
                .find(|o| o.path == rel_path);

            match cached_output {
                Some(out) if self.has_object(&out.checksum) => {}
                _ => return false,
            }
        }

        true
    }

    /// Explain what action will be taken for a product and why.
    /// Mirrors the logic in needs_rebuild/can_restore but returns structured reasons.
    pub fn explain_action(&self, cache_key: &str, input_checksum: &str, output_paths: &[PathBuf], force: bool) -> ExplainAction {
        if force {
            return ExplainAction::Rebuild(RebuildReason::Force);
        }

        let entry = match self.get_entry(cache_key) {
            Some(e) => e,
            None => return ExplainAction::Rebuild(RebuildReason::NoCacheEntry),
        };

        if entry.input_checksum != input_checksum {
            // Inputs changed — check if restorable (shouldn't be, since checksum differs)
            return ExplainAction::Rebuild(RebuildReason::InputsChanged);
        }

        // For checkers (empty outputs), matching checksum means up-to-date
        if output_paths.is_empty() {
            return ExplainAction::Skip;
        }

        // Check outputs — must verify ALL missing outputs are restorable before
        // reporting Restore, otherwise the actual restore would fail and fall through
        // to a rebuild.
        let mut first_missing: Option<String> = None;
        for output_path in output_paths {
            if !output_path.exists() {
                let rel_path = Self::path_string(output_path);
                let cached_output = entry.outputs.iter().find(|o| o.path == rel_path);
                match cached_output {
                    Some(out) if self.has_object(&out.checksum) => {
                        if first_missing.is_none() {
                            first_missing = Some(rel_path);
                        }
                    }
                    _ => {
                        return ExplainAction::Rebuild(RebuildReason::OutputMissing(rel_path));
                    }
                }
            }
        }

        match first_missing {
            Some(path) => ExplainAction::Restore(RebuildReason::OutputMissing(path)),
            None => ExplainAction::Skip,
        }
    }

    /// Check if a product with an output directory needs rebuilding.
    /// Returns true if no cache entry, input checksum differs, or output dir doesn't exist.
    pub fn needs_rebuild_output_dir(&self, cache_key: &str, input_checksum: &str, dir: &Path) -> bool {
        let entry = match self.get_entry(cache_key) {
            Some(e) => e,
            None => return true,
        };

        if entry.input_checksum != input_checksum {
            return true;
        }

        // If the output directory doesn't exist, we need a rebuild (or restore)
        !dir.exists()
    }

    /// Check if an output directory can be restored from cache.
    /// Returns true if cache entry exists with matching checksum and all objects are available.
    pub fn can_restore_output_dir(&self, cache_key: &str, input_checksum: &str) -> bool {
        let entry = match self.get_entry(cache_key) {
            Some(e) if e.input_checksum == input_checksum => e,
            _ => return false,
        };

        entry.outputs.iter().all(|o| self.has_object(&o.checksum))
    }

    /// Explain what action will be taken for a product with an output directory.
    pub fn explain_action_output_dir(&self, cache_key: &str, input_checksum: &str, dir: &Path, force: bool) -> ExplainAction {
        if force {
            return ExplainAction::Rebuild(RebuildReason::Force);
        }

        let entry = match self.get_entry(cache_key) {
            Some(e) => e,
            None => return ExplainAction::Rebuild(RebuildReason::NoCacheEntry),
        };

        if entry.input_checksum != input_checksum {
            return ExplainAction::Rebuild(RebuildReason::InputsChanged);
        }

        if dir.exists() {
            return ExplainAction::Skip;
        }

        // Directory missing — check if all objects are available for restore
        let all_available = entry.outputs.iter().all(|o| self.has_object(&o.checksum));
        let dir_display = dir.display().to_string();
        if all_available {
            ExplainAction::Restore(RebuildReason::OutputMissing(dir_display))
        } else {
            ExplainAction::Rebuild(RebuildReason::OutputMissing(dir_display))
        }
    }
}
