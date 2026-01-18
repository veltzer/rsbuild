use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use crate::checksum::ChecksumCache;

/// A single build product with concrete inputs and outputs
#[derive(Debug, Clone)]
pub struct Product {
    /// Input files (real paths)
    pub inputs: Vec<PathBuf>,
    /// Output files (real paths)
    pub outputs: Vec<PathBuf>,
    /// Which processor handles this product
    pub processor: String,
    /// Unique identifier for this product
    pub id: usize,
}

impl Product {
    pub fn new(inputs: Vec<PathBuf>, outputs: Vec<PathBuf>, processor: &str, id: usize) -> Self {
        Self {
            inputs,
            outputs,
            processor: processor.to_string(),
            id,
        }
    }

    /// Display name for logging
    pub fn display(&self) -> String {
        let inputs: Vec<_> = self.inputs.iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        let outputs: Vec<_> = self.outputs.iter()
            .filter_map(|p| p.file_name())
            .filter_map(|n| n.to_str())
            .collect();
        format!("input: {}, output: {}", inputs.join(", "), outputs.join(", "))
    }

    /// Cache key for checksum tracking
    pub fn cache_key(&self) -> String {
        let inputs: Vec<_> = self.inputs.iter()
            .map(|p| p.display().to_string())
            .collect();
        format!("{}:{}", self.processor, inputs.join(":"))
    }
}

/// Build graph with dependency resolution
pub struct BuildGraph {
    products: Vec<Product>,
    /// Map from output path to product id
    output_to_product: HashMap<PathBuf, usize>,
    /// Adjacency list: product id -> list of product ids that depend on it
    dependents: HashMap<usize, Vec<usize>>,
    /// Reverse: product id -> list of product ids it depends on
    dependencies: HashMap<usize, Vec<usize>>,
}

impl BuildGraph {
    pub fn new() -> Self {
        Self {
            products: Vec::new(),
            output_to_product: HashMap::new(),
            dependents: HashMap::new(),
            dependencies: HashMap::new(),
        }
    }

    /// Add a product to the graph
    pub fn add_product(&mut self, inputs: Vec<PathBuf>, outputs: Vec<PathBuf>, processor: &str) -> usize {
        let id = self.products.len();
        let product = Product::new(inputs, outputs.clone(), processor, id);

        // Register outputs
        for output in &outputs {
            self.output_to_product.insert(output.clone(), id);
        }

        self.products.push(product);
        self.dependents.insert(id, Vec::new());
        self.dependencies.insert(id, Vec::new());

        id
    }

    /// Resolve dependencies between products
    pub fn resolve_dependencies(&mut self) {
        // For each product, check if any of its inputs are outputs of other products
        for product in &self.products {
            for input in &product.inputs {
                if let Some(&producer_id) = self.output_to_product.get(input) {
                    if producer_id != product.id {
                        // producer_id produces something that product.id needs
                        self.dependents.get_mut(&producer_id).unwrap().push(product.id);
                        self.dependencies.get_mut(&product.id).unwrap().push(producer_id);
                    }
                }
            }
        }
    }

    /// Topological sort - returns product ids in execution order
    /// Returns error if there's a cycle
    pub fn topological_sort(&self) -> Result<Vec<usize>> {
        let mut in_degree: HashMap<usize, usize> = HashMap::new();

        // Initialize in-degrees
        for product in &self.products {
            in_degree.insert(product.id, self.dependencies.get(&product.id).map_or(0, |d| d.len()));
        }

        // Start with products that have no dependencies
        let mut queue: Vec<usize> = in_degree.iter()
            .filter(|&(_, deg)| *deg == 0)
            .map(|(&id, _)| id)
            .collect();

        let mut result = Vec::new();
        let mut visited = HashSet::new();

        while let Some(id) = queue.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            result.push(id);

            // Reduce in-degree of dependents
            if let Some(deps) = self.dependents.get(&id) {
                for &dep_id in deps {
                    if let Some(deg) = in_degree.get_mut(&dep_id) {
                        *deg = deg.saturating_sub(1);
                        if *deg == 0 && !visited.contains(&dep_id) {
                            queue.push(dep_id);
                        }
                    }
                }
            }
        }

        if result.len() != self.products.len() {
            bail!("Cycle detected in build graph");
        }

        Ok(result)
    }

    /// Get a product by id
    pub fn get_product(&self, id: usize) -> Option<&Product> {
        self.products.get(id)
    }

    /// Get all products
    pub fn products(&self) -> &[Product] {
        &self.products
    }

    /// Check if a product needs rebuilding based on checksums
    pub fn needs_rebuild(&self, product: &Product, cache: &ChecksumCache, force: bool) -> Result<bool> {
        if force {
            return Ok(true);
        }

        let cache_key = product.cache_key();

        // Calculate combined checksum of all inputs
        let mut checksums = Vec::new();
        for input in &product.inputs {
            if input.exists() {
                checksums.push(ChecksumCache::calculate_checksum(input)?);
            } else {
                // Input doesn't exist yet, needs rebuild
                return Ok(true);
            }
        }
        let combined = checksums.join(":");

        // Check if outputs exist
        for output in &product.outputs {
            if !output.exists() {
                return Ok(true);
            }
        }

        // Check cache
        if let Some(cached) = cache.get_by_key(&cache_key) {
            if cached == &combined {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Update cache after successful build
    pub fn update_cache(&self, product: &Product, cache: &mut ChecksumCache) -> Result<()> {
        let cache_key = product.cache_key();

        let mut checksums = Vec::new();
        for input in &product.inputs {
            checksums.push(ChecksumCache::calculate_checksum(input)?);
        }
        let combined = checksums.join(":");

        cache.set_by_key(cache_key, combined);
        Ok(())
    }
}

impl Default for BuildGraph {
    fn default() -> Self {
        Self::new()
    }
}
