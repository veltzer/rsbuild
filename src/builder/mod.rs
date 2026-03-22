mod build;
mod clean;
mod config_cmd;
mod deps;
mod doctor;
mod graph;
pub(crate) mod processors;
pub(crate) mod sloc;
pub(crate) mod smart;
mod tools;

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use anyhow::Result;
use crate::analyzers::{CppDepAnalyzer, DepAnalyzer, PythonDepAnalyzer};
use crate::cli::BuildPhase;
use crate::color;
use crate::config::{Config, ProcessorConfig};
use crate::deps_cache::DepsCache;
use crate::errors;
use crate::file_index::FileIndex;
use crate::graph::BuildGraph;
use crate::object_store::{ObjectStore, ObjectStoreOptions};
use crate::processors as proc_mod;
use crate::processors::{LuaProcessor, ProcessorMap, ProductDiscovery, SpellcheckProcessor, names as proc_names};
use crate::remote_cache;
use crate::tool_lock;

/// Phase timing data collected during graph building.
pub(crate) type PhaseTimings = Vec<(String, Duration)>;

/// Severity level for validation issues.
pub enum ValidationSeverity {
    Error,
    Warning,
}

/// A single validation issue found during config validation.
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub message: String,
}

/// Controls which graph-building variant to use.
#[derive(Clone, Copy, PartialEq, Eq)]
enum GraphBuildMode {
    /// Full build: discover products, run analyzers, resolve dependencies
    Normal,
    /// Clean: only discover products to find output files (skip expensive analysis)
    ForClean,
}

/// Return the keys of a HashMap sorted alphabetically.
fn sorted_keys<V>(map: &HashMap<String, V>) -> Vec<&String> {
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    keys
}

/// Check if phases debug is enabled.
fn phases_debug() -> bool {
    crate::runtime_flags::phases_debug()
}

/// Labels for the three product states used by dry_run and status.
struct ProductStatusLabels<'a> {
    current: (Cow<'a, str>, &'static str),
    restorable: (Cow<'a, str>, &'static str),
    stale: (Cow<'a, str>, &'static str),
}

/// Auto-generate `create_builtin_processors` from the central registry.
/// SpellcheckProcessor is skipped here because its `new()` is fallible;
/// it is registered separately in `Builder::create_processors()`.
macro_rules! gen_builtin_processors {
    ( $( $const_name:ident, $field:ident, $config_type:ty, $proc_type:ident,
         ($($scan_args:tt)*); )* ) => {
        pub(crate) fn create_builtin_processors(cfg: &ProcessorConfig) -> ProcessorMap {
            let mut processors: ProcessorMap = HashMap::new();
            $(
                gen_builtin_processors!(@register processors, cfg,
                    $const_name, $field, $proc_type);
            )*
            processors
        }
    };
    // Skip SpellcheckProcessor (fallible new())
    (@register $processors:ident, $cfg:ident, SPELLCHECK, $field:ident, $proc_type:ident) => {
        // Registered separately in Builder::create_processors() with error handling
    };
    // Normal case
    (@register $processors:ident, $cfg:ident, $const_name:ident, $field:ident, $proc_type:ident) => {
        Builder::register(
            &mut $processors,
            proc_names::$const_name,
            proc_mod::$proc_type::new($cfg.$field.clone()),
        );
    };
}
for_each_processor!(gen_builtin_processors);

pub struct Builder {
    object_store: ObjectStore,
    config: Config,
    file_index: FileIndex,
}

impl Builder {
    pub fn new() -> Result<Self> {
        Config::require_config()?;
        let config = Config::load()?;

        // Validate: compression and hardlink restore are incompatible
        if config.cache.compression && config.cache.restore_method == crate::config::RestoreMethod::Hardlink {
            anyhow::bail!("Cannot use cache compression with hardlink restore method. \
                Set restore_method = \"copy\" or disable compression.");
        }

        // Create remote cache backend if configured
        let remote_backend = match &config.cache.remote {
            Some(url) => Some(remote_cache::create_backend(url)?),
            None => None,
        };

        let object_store = ObjectStore::new(ObjectStoreOptions {
            restore_method: config.cache.restore_method,
            compression: config.cache.compression,
            remote: remote_backend,
            remote_push: config.cache.remote_push,
            remote_pull: config.cache.remote_pull,
            mtime_check: config.cache.mtime_check,
        })?;
        let file_index = FileIndex::build()?;

        Ok(Self {
            object_store,
            config,
            file_index,
        })
    }

    /// Register a processor into the map.
    fn register(processors: &mut ProcessorMap, name: &str, proc: impl ProductDiscovery + 'static) {
        processors.insert(name.to_string(), Box::new(proc));
    }

    /// Create all available processors
    pub fn create_processors(&self) -> Result<ProcessorMap> {
        let cfg = &self.config.processor;
        let mut processors = create_builtin_processors(cfg);

        // Spellcheck processor (fallible init — propagate error only if enabled)
        match SpellcheckProcessor::new(cfg.spellcheck.clone()) {
            Ok(proc) => Self::register(&mut processors, proc_names::SPELLCHECK, proc),
            Err(e) if self.config.processor.is_enabled(proc_names::SPELLCHECK) => return Err(e),
            Err(_) => {}
        }

        // Lua plugin processors
        let lua_plugins = LuaProcessor::discover_plugins(
            &self.config.plugins.dir,
            &cfg.extra,
        )?;
        for (name, proc) in lua_plugins {
            if processors.contains_key(&name) {
                anyhow::bail!("Lua plugin '{}' conflicts with built-in processor", name);
            }
            processors.insert(name, Box::new(proc));
        }

        Ok(processors)
    }

    /// Detect and display config changes for each processor.
    /// Shows colored diffs when processor configuration has changed since last build.
    fn detect_config_changes(&self, processors: &ProcessorMap) {
        // Don't show config diffs in JSON or quiet mode
        if crate::json_output::is_json_mode() || crate::runtime_flags::quiet() {
            return;
        }

        for name in sorted_keys(processors) {
            let processor = processors.get(name).expect(errors::PROCESSOR_NOT_IN_MAP);

            // Skip processors that don't provide config JSON
            let config_json = match processor.config_json() {
                Some(json) => json,
                None => continue,
            };

            // Store the config and check if it changed
            if let Ok(Some(old_json)) = self.object_store.store_processor_config(name, &config_json) {
                // Config changed - show diff
                if let Some(diff) = ObjectStore::diff_configs(&old_json, &config_json) {
                    println!("{}",
                        color::yellow(&format!("Config changed for [{}]:", name)));
                    println!("{}", diff);
                }
            }
        }
    }

    /// Create all available dependency analyzers
    fn create_analyzers(&self, verbose: bool) -> HashMap<String, Box<dyn DepAnalyzer>> {
        let mut analyzers: HashMap<String, Box<dyn DepAnalyzer>> = HashMap::new();

        // C/C++ dependency analyzer
        let cpp_analyzer = CppDepAnalyzer::new(
            self.config.analyzer.cpp.clone(),
            verbose,
        );
        analyzers.insert("cpp".to_string(), Box::new(cpp_analyzer));

        // Python dependency analyzer
        let python_analyzer = PythonDepAnalyzer::new();
        analyzers.insert("python".to_string(), Box::new(python_analyzer));

        analyzers
    }

    /// Run all enabled dependency analyzers on the graph
    fn run_analyzers(&self, graph: &mut BuildGraph, verbose: bool) -> Result<()> {
        let analyzers = self.create_analyzers(verbose);
        let mut deps_cache = DepsCache::open()?;

        // Collect which analyzers should run
        let active_analyzers: Vec<&String> = sorted_keys(&analyzers).into_iter()
            .filter(|name| {
                let in_enabled_list = self.config.analyzer.is_enabled(name);
                if !in_enabled_list {
                    return false;
                }
                !self.config.analyzer.auto_detect || analyzers[*name].auto_detect(&self.file_index)
            })
            .collect();

        // Run each analyzer
        for name in &active_analyzers {
            analyzers[*name].analyze(graph, &mut deps_cache, &self.file_index, verbose)?;
        }

        Ok(())
    }

    /// Build the dependency graph using provided processors
    fn build_graph_with_processors(&self, processors: &ProcessorMap) -> Result<BuildGraph> {
        let (graph, _) = self.build_graph_with_processors_impl(processors, GraphBuildMode::Normal, BuildPhase::Build, None, false)?;
        Ok(graph)
    }

    /// Build the dependency graph with optional early stopping
    fn build_graph_with_processors_and_phase(&self, processors: &ProcessorMap, stop_after: BuildPhase, processor_filter: Option<&[String]>, verbose: bool) -> Result<(BuildGraph, PhaseTimings)> {
        self.build_graph_with_processors_impl(processors, GraphBuildMode::Normal, stop_after, processor_filter, verbose)
    }

    /// Build the dependency graph for clean (skip expensive dependency scanning)
    fn build_graph_for_clean_with_processors(&self, processors: &ProcessorMap) -> Result<BuildGraph> {
        let (graph, _) = self.build_graph_with_processors_impl(processors, GraphBuildMode::ForClean, BuildPhase::Build, None, false)?;
        Ok(graph)
    }

    /// Return the set of processor names whose files are detected in the project.
    /// Checks auto_detect regardless of enabled status.
    pub fn detected_processors(&self) -> Result<std::collections::HashSet<String>> {
        let processors = self.create_processors()?;
        let mut detected = std::collections::HashSet::new();
        for (name, proc) in &processors {
            if !proc.hidden() && proc.auto_detect(&self.file_index) {
                detected.insert(name.clone());
            }
        }
        Ok(detected)
    }

    /// Return the set of processor names whose files are detected AND whose
    /// required tools are all installed.
    pub fn detected_and_available_processors(&self) -> Result<std::collections::HashSet<String>> {
        let processors = self.create_processors()?;
        let mut available = std::collections::HashSet::new();
        for (name, proc) in &processors {
            if proc.hidden() || !proc.auto_detect(&self.file_index) {
                continue;
            }
            let tools = proc.required_tools();
            if tools.iter().all(|t| which::which(t).is_ok()) {
                available.insert(name.clone());
            }
        }
        Ok(available)
    }

    /// Check whether a processor should run based on enabled list and auto-detect.
    fn is_processor_active(&self, name: &str, processor: &dyn ProductDiscovery) -> bool {
        if !self.config.processor.is_enabled(name) {
            return false;
        }
        !self.config.processor.auto_detect || processor.auto_detect(&self.file_index)
    }

    /// Build the dependency graph using provided processors
    /// processor_filter: if Some, only run processors in this list (in addition to enabled check)
    fn build_graph_with_processors_impl(&self, processors: &ProcessorMap, mode: GraphBuildMode, stop_after: BuildPhase, processor_filter: Option<&[String]>, verbose: bool) -> Result<(BuildGraph, PhaseTimings)> {
        if phases_debug() {
            eprintln!("{}", color::bold("Phase: Building dependency graph..."));
        }
        let mut graph = BuildGraph::new();
        let mut phase_timings = PhaseTimings::new();

        // Collect which processors should run
        let active_processors: Vec<&String> = sorted_keys(processors).into_iter()
            .filter(|name| {
                if let Some(filter) = processor_filter
                    && !filter.iter().any(|f| f == *name) {
                        return false;
                    }
                self.is_processor_active(name, processors[*name].as_ref())
            })
            .collect();

        // Phase 1: Discover products
        if phases_debug() {
            eprintln!("{}", color::dim("  Phase: discover"));
        }
        let t = Instant::now();
        for name in &active_processors {
            if mode == GraphBuildMode::ForClean {
                processors[*name].discover_for_clean(&mut graph, &self.file_index)?;
            } else {
                processors[*name].discover(&mut graph, &self.file_index)?;
            }
        }
        phase_timings.push(("discover".to_string(), t.elapsed()));

        if stop_after == BuildPhase::Discover {
            return Ok((graph, phase_timings));
        }

        // Phase 2: Run dependency analyzers (only for regular builds, not clean)
        if mode == GraphBuildMode::Normal {
            if phases_debug() {
                eprintln!("{}", color::dim("  Phase: add_dependencies"));
            }
            let t = Instant::now();
            self.run_analyzers(&mut graph, verbose)?;
            phase_timings.push(("add_dependencies".to_string(), t.elapsed()));
        }

        if stop_after == BuildPhase::AddDependencies {
            return Ok((graph, phase_timings));
        }

        // Phase 3: Apply tool version hashes
        if phases_debug() {
            eprintln!("{}", color::dim("  Phase: apply_tool_version_hashes"));
        }
        let t = Instant::now();
        let config = &self.config;
        let tool_hashes = tool_lock::processor_tool_hashes(
            processors,
            &|name| config.processor.is_enabled(name),
        )?;
        if !tool_hashes.is_empty() {
            graph.apply_tool_version_hashes(&tool_hashes);
        }
        phase_timings.push(("tool_version_hashes".to_string(), t.elapsed()));

        // Phase 4: Resolve dependencies
        if phases_debug() {
            eprintln!("{}", color::dim("  Phase: resolve_dependencies"));
        }
        let t = Instant::now();
        graph.resolve_dependencies();
        phase_timings.push(("resolve".to_string(), t.elapsed()));

        // Note: BuildPhase::Resolve and BuildPhase::Build both complete the graph
        Ok((graph, phase_timings))
    }

    /// Build the dependency graph, optionally filtering to a single processor.
    /// If `include_all` is true, skip enabled/auto-detect checks.
    pub fn build_graph_filtered(
        &self,
        filter_name: Option<&str>,
        include_all: bool,
    ) -> Result<BuildGraph> {
        let processors = self.create_processors()?;
        let mut graph = BuildGraph::new();

        // Collect active processors
        let active_processors: Vec<&String> = sorted_keys(&processors).into_iter()
            .filter(|name| {
                if let Some(filter) = filter_name
                    && name.as_str() != filter {
                        return false;
                    }
                include_all || self.is_processor_active(name, processors[*name].as_ref())
            })
            .collect();

        // Phase 1: Discover products
        for name in &active_processors {
            processors[*name].discover(&mut graph, &self.file_index)?;
        }

        // Phase 2: Run dependency analyzers
        self.run_analyzers(&mut graph, false)?;

        graph.resolve_dependencies();
        Ok(graph)
    }

    /// Build the dependency graph (creates processors internally)
    fn build_graph(&self) -> Result<BuildGraph> {
        let processors = self.create_processors()?;
        self.build_graph_with_processors(&processors)
    }

    /// Build the dependency graph for cache operations (public).
    pub fn build_graph_for_cache(&self) -> Result<BuildGraph> {
        self.build_graph()
    }

    /// Compute the set of valid cache keys from the current build graph.
    pub fn valid_cache_keys(&self) -> Result<std::collections::HashSet<String>> {
        let graph = self.build_graph_for_cache()?;
        Ok(graph.products().iter().map(|p| p.cache_key()).collect())
    }

    /// Get a reference to the object store.
    pub fn object_store(&self) -> &ObjectStore {
        &self.object_store
    }

    /// Return directories that should be watched for file changes.
    /// Derived from processor scan configs plus standard project files.
    pub fn watch_paths(&self) -> Vec<PathBuf> {
        let mut paths: Vec<PathBuf> = Vec::new();

        // Always watch rsconstruct.toml
        let config_path = PathBuf::from("rsconstruct.toml");
        if config_path.exists() {
            paths.push(config_path);
        }

        // Add scan directories from all processor configs
        for dir in self.config.processor.scan_dirs() {
            let full = PathBuf::from(&dir);
            if full.exists() {
                paths.push(full);
            }
        }

        // Processors with empty scan_dir scan the project root — watch common
        // top-level files/dirs that wouldn't be covered by scan_dirs above.
        for name in &["pyproject.toml", "config"] {
            let p = PathBuf::from(name);
            if p.exists() {
                paths.push(p);
            }
        }

        // Plugins directory
        let plugins = PathBuf::from(&self.config.plugins.dir);
        if plugins.exists() {
            paths.push(plugins);
        }

        paths.sort();
        paths.dedup();
        paths
    }
}
