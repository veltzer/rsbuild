//! Shared redb database utilities.

use anyhow::{Context, Result};
use redb::Database;
use std::fs;
use std::path::Path;

/// Open or create a redb database, with delete-and-retry on corruption.
///
/// If the database file is corrupted, it is deleted and recreated.
/// A warning is printed to stderr describing which database was recreated.
pub fn open_or_recreate(db_path: &Path, label: &str) -> Result<Database> {
    match Database::create(db_path) {
        Ok(db) => Ok(db),
        Err(_) => {
            eprintln!("Warning: {} corrupted, recreating", label);
            fs::remove_file(db_path)
                .with_context(|| format!("Failed to remove corrupted {}", label))?;
            Database::create(db_path)
                .with_context(|| format!("Failed to create {}", label))
        }
    }
}
