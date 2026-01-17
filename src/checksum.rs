use anyhow::Result;
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChecksumCache {
    checksums: HashMap<PathBuf, String>,
}

impl ChecksumCache {
    pub fn new() -> Self {
        Self {
            checksums: HashMap::new(),
        }
    }

    pub fn load_from_file(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(serde_json::from_str(&content)?)
        } else {
            Ok(Self::new())
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = serde_json::to_string_pretty(&self)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn calculate_checksum(file_path: &Path) -> Result<String> {
        let contents = fs::read(file_path)?;
        let mut hasher = Sha256::new();
        hasher.update(contents);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    pub fn clear(&mut self) {
        self.checksums.clear();
    }

    /// Get checksum for a string key (used for lint cache entries)
    pub fn get_by_key(&self, key: &str) -> Option<&String> {
        self.checksums.get(&PathBuf::from(key))
    }

    /// Set checksum for a string key (used for lint cache entries)
    pub fn set_by_key(&mut self, key: String, checksum: String) {
        self.checksums.insert(PathBuf::from(key), checksum);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_checksum_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "Hello, World!").unwrap();

        let checksum1 = ChecksumCache::calculate_checksum(&file_path).unwrap();
        let checksum2 = ChecksumCache::calculate_checksum(&file_path).unwrap();

        // Same content should produce same checksum
        assert_eq!(checksum1, checksum2);

        // Checksum should be a valid hex string
        assert_eq!(checksum1.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_change_detection() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "Initial content").unwrap();

        let mut cache = ChecksumCache::new();
        let key = "test:file";

        // First check - no cached checksum
        assert!(cache.get_by_key(key).is_none());

        // Store the checksum
        let checksum1 = ChecksumCache::calculate_checksum(&file_path).unwrap();
        cache.set_by_key(key.to_string(), checksum1.clone());

        // Second check - should match
        assert_eq!(cache.get_by_key(key), Some(&checksum1));

        // Modify the file
        fs::write(&file_path, "Modified content").unwrap();
        let checksum2 = ChecksumCache::calculate_checksum(&file_path).unwrap();

        // Should be different
        assert_ne!(checksum1, checksum2);
    }

    #[test]
    fn test_cache_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let cache_path = temp_dir.path().join("cache.json");
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "Test content").unwrap();

        // Create cache and track a file
        let mut cache1 = ChecksumCache::new();
        let key = "test:file";
        let checksum = ChecksumCache::calculate_checksum(&file_path).unwrap();
        cache1.set_by_key(key.to_string(), checksum.clone());
        cache1.save_to_file(&cache_path).unwrap();

        // Load cache from file
        let cache2 = ChecksumCache::load_from_file(&cache_path).unwrap();

        // Should have the same checksum
        assert_eq!(cache2.get_by_key(key), Some(&checksum));
    }

    #[test]
    fn test_clear() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "Content").unwrap();

        let mut cache = ChecksumCache::new();
        let key = "test:file";
        let checksum = ChecksumCache::calculate_checksum(&file_path).unwrap();
        cache.set_by_key(key.to_string(), checksum);

        // Cache should have the file
        assert!(cache.get_by_key(key).is_some());

        // Clear the cache
        cache.clear();

        // Cache should be empty
        assert!(cache.get_by_key(key).is_none());
    }
}