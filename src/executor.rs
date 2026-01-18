use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::graph::BuildGraph;
use crate::object_store::ObjectStore;
use crate::processors::{BuildStats, ProcessStats, ProductDiscovery};

/// Executor handles running products through their processors
/// It respects dependency order and can parallelize independent products
pub struct Executor<'a> {
    processors: &'a HashMap<String, Box<dyn ProductDiscovery>>,
    parallel: usize,
}

impl<'a> Executor<'a> {
    pub fn new(processors: &'a HashMap<String, Box<dyn ProductDiscovery>>, parallel: usize) -> Self {
        Self {
            processors,
            parallel,
        }
    }

    /// Execute all products in the graph that need rebuilding
    pub fn execute(
        &self,
        graph: &BuildGraph,
        object_store: &mut ObjectStore,
        force: bool,
        verbose: bool,
    ) -> Result<BuildStats> {
        let order = graph.topological_sort()?;

        if self.parallel <= 1 {
            self.execute_sequential(graph, &order, object_store, force, verbose)
        } else {
            self.execute_parallel(graph, &order, object_store, force, verbose)
        }
    }

    /// Execute products sequentially
    fn execute_sequential(
        &self,
        graph: &BuildGraph,
        order: &[usize],
        object_store: &mut ObjectStore,
        force: bool,
        verbose: bool,
    ) -> Result<BuildStats> {
        let mut stats_by_processor: HashMap<String, ProcessStats> = HashMap::new();

        for &id in order {
            let product = graph.get_product(id).unwrap();
            let cache_key = product.cache_key();
            let input_checksum = ObjectStore::combined_input_checksum(&product.inputs)?;

            // Check if this product needs rebuilding
            if !force && !object_store.needs_rebuild(&cache_key, &input_checksum, &product.outputs) {
                if verbose {
                    println!("[{}] Skipping (unchanged): {}", product.processor, product.display());
                }
                let stats = stats_by_processor
                    .entry(product.processor.clone())
                    .or_insert_with(|| ProcessStats::new(&product.processor));
                stats.skipped += 1;
                continue;
            }

            // Try to restore from cache if outputs are missing
            if !force && object_store.restore_from_cache(&cache_key, &input_checksum, &product.outputs)? {
                if verbose {
                    println!("[{}] Restored from cache: {}", product.processor, product.display());
                }
                let stats = stats_by_processor
                    .entry(product.processor.clone())
                    .or_insert_with(|| ProcessStats::new(&product.processor));
                stats.skipped += 1;
                continue;
            }

            // Find the processor and execute
            if let Some(processor) = self.processors.get(&product.processor) {
                println!("[{}] Processing: {}", product.processor, product.display());
                processor.execute(product)?;

                // Cache outputs
                object_store.cache_outputs(&cache_key, &input_checksum, &product.outputs)?;

                let stats = stats_by_processor
                    .entry(product.processor.clone())
                    .or_insert_with(|| ProcessStats::new(&product.processor));
                stats.processed += 1;
            }
        }

        // Build aggregated stats
        let mut stats = BuildStats::default();
        for (_, proc_stats) in stats_by_processor {
            stats.add(proc_stats);
        }

        Ok(stats)
    }

    /// Execute products in parallel where dependencies allow
    fn execute_parallel(
        &self,
        graph: &BuildGraph,
        order: &[usize],
        object_store: &mut ObjectStore,
        force: bool,
        verbose: bool,
    ) -> Result<BuildStats> {
        // Group products into levels that can run in parallel
        let levels = self.compute_parallel_levels(graph, order);

        let stats_by_processor: Arc<Mutex<HashMap<String, ProcessStats>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let store = Arc::new(Mutex::new(std::mem::take(object_store)));
        let errors: Arc<Mutex<Vec<anyhow::Error>>> = Arc::new(Mutex::new(Vec::new()));

        for level in levels {
            // Determine which products in this level need work
            // (id, input_checksum, needs_rebuild, can_restore)
            let mut work_items: Vec<(usize, String, bool, bool)> = Vec::new();

            {
                let store_guard = store.lock().unwrap();
                for &id in &level {
                    let product = graph.get_product(id).unwrap();
                    let cache_key = product.cache_key();
                    let input_checksum = match ObjectStore::combined_input_checksum(&product.inputs) {
                        Ok(cs) => cs,
                        Err(e) => {
                            errors.lock().unwrap().push(e);
                            continue;
                        }
                    };

                    let needs = force || store_guard.needs_rebuild(&cache_key, &input_checksum, &product.outputs);
                    work_items.push((id, input_checksum, needs, false));
                }
            }

            // Process this level in parallel using thread pool
            let chunk_size = (work_items.len() + self.parallel - 1) / self.parallel;
            let chunks: Vec<_> = work_items.chunks(chunk_size.max(1)).collect();

            thread::scope(|s| {
                for chunk in chunks {
                    let stats_ref = Arc::clone(&stats_by_processor);
                    let store_ref = Arc::clone(&store);
                    let errors_ref = Arc::clone(&errors);

                    s.spawn(move || {
                        for (id, input_checksum, needs_rebuild, _) in chunk {
                            let product = graph.get_product(*id).unwrap();
                            let cache_key = product.cache_key();

                            if !needs_rebuild {
                                if verbose {
                                    println!("[{}] Skipping (unchanged): {}", product.processor, product.display());
                                }
                                let mut stats = stats_ref.lock().unwrap();
                                let proc_stats = stats
                                    .entry(product.processor.clone())
                                    .or_insert_with(|| ProcessStats::new(&product.processor));
                                proc_stats.skipped += 1;
                                continue;
                            }

                            // Try to restore from cache
                            if !force {
                                let restore_result = {
                                    let store_guard = store_ref.lock().unwrap();
                                    store_guard.restore_from_cache(&cache_key, input_checksum, &product.outputs)
                                };
                                match restore_result {
                                    Ok(true) => {
                                        if verbose {
                                            println!("[{}] Restored from cache: {}", product.processor, product.display());
                                        }
                                        let mut stats = stats_ref.lock().unwrap();
                                        let proc_stats = stats
                                            .entry(product.processor.clone())
                                            .or_insert_with(|| ProcessStats::new(&product.processor));
                                        proc_stats.skipped += 1;
                                        continue;
                                    }
                                    Err(e) => {
                                        errors_ref.lock().unwrap().push(e);
                                        continue;
                                    }
                                    Ok(false) => {}
                                }
                            }

                            if let Some(processor) = self.processors.get(&product.processor) {
                                println!("[{}] Processing: {}", product.processor, product.display());

                                if let Err(e) = processor.execute(product) {
                                    errors_ref.lock().unwrap().push(e);
                                    continue;
                                }

                                // Cache outputs
                                {
                                    let mut store_guard = store_ref.lock().unwrap();
                                    if let Err(e) = store_guard.cache_outputs(&cache_key, input_checksum, &product.outputs) {
                                        errors_ref.lock().unwrap().push(e);
                                        continue;
                                    }
                                }

                                let mut stats = stats_ref.lock().unwrap();
                                let proc_stats = stats
                                    .entry(product.processor.clone())
                                    .or_insert_with(|| ProcessStats::new(&product.processor));
                                proc_stats.processed += 1;
                            }
                        }
                    });
                }
            });

            // Check for errors after each level
            let errs = errors.lock().unwrap();
            if !errs.is_empty() {
                // Restore store before returning error
                *object_store = Arc::try_unwrap(store).unwrap().into_inner().unwrap();
                return Err(anyhow::anyhow!("Build failed: {}", errs[0]));
            }
        }

        // Restore the store
        *object_store = Arc::try_unwrap(store).unwrap().into_inner().unwrap();

        // Build aggregated stats
        let final_stats = Arc::try_unwrap(stats_by_processor).unwrap().into_inner().unwrap();
        let mut stats = BuildStats::default();
        for (_, proc_stats) in final_stats {
            stats.add(proc_stats);
        }

        Ok(stats)
    }

    /// Compute levels of products that can be executed in parallel
    /// Products in the same level have no dependencies on each other
    fn compute_parallel_levels(&self, graph: &BuildGraph, order: &[usize]) -> Vec<Vec<usize>> {
        let mut levels: Vec<Vec<usize>> = Vec::new();
        let mut product_level: HashMap<usize, usize> = HashMap::new();

        for &id in order {
            let product = graph.get_product(id).unwrap();

            // Find the maximum level of all dependencies
            let max_dep_level = graph.get_dependencies(id)
                .iter()
                .filter_map(|&dep_id| product_level.get(&dep_id))
                .max()
                .copied()
                .unwrap_or(0);

            // This product goes in the next level after its dependencies
            let my_level = if graph.get_dependencies(id).is_empty() {
                0
            } else {
                max_dep_level + 1
            };

            product_level.insert(product.id, my_level);

            // Ensure we have enough levels
            while levels.len() <= my_level {
                levels.push(Vec::new());
            }
            levels[my_level].push(id);
        }

        levels
    }

    /// Clean all products
    pub fn clean(&self, graph: &BuildGraph) -> Result<()> {
        for product in graph.products() {
            if let Some(processor) = self.processors.get(&product.processor) {
                processor.clean(product)?;
            }
        }
        Ok(())
    }
}
