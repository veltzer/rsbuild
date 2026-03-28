//! Shared redb database utilities.

use anyhow::Result;
use redb::Database;
use std::path::Path;

/// Open or create a redb database.
///
/// Provides specific error messages depending on the failure:
/// - Lock contention: another rsconstruct process is running
/// - Other errors: likely corruption, suggest `rsconstruct cache clear`
pub fn open_or_recreate(db_path: &Path, label: &str) -> Result<Database> {
    Database::create(db_path).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("already open") || msg.contains("lock") {
            anyhow::anyhow!(
                "Another rsconstruct process is using {}: {}\n\
                 Wait for it to finish, or check for stale processes.",
                db_path.display(),
                e
            )
        } else {
            anyhow::anyhow!(
                "{} is corrupted: {}\n\
                 Run `rsconstruct cache clear` to delete it and rebuild.\n\
                 Cause: {}",
                label,
                db_path.display(),
                e
            )
        }
    })
}
