/// Macro to generate `ProductDiscovery` trait implementations for checker processors.
///
/// Eliminates boilerplate for `description()`, `auto_detect()`, `required_tools()`,
/// `discover()`, `execute()`, `config_json()`, and batch support.
///
/// # Required parameters
/// - `$processor:ty` — the processor struct type
/// - `config: $config_field:ident` — name of the config field on the struct
/// - `description: $desc:expr` — human-readable description string
/// - `name: $name:expr` — processor name passed to `discover_checker_products()`
/// - `execute: $execute:ident` — method on self for single-product execution
///
/// # Optional parameters (in any order after required ones)
/// - `guard: $guard_method:ident` — method on self returning bool; gates `auto_detect()` and `discover()`
/// - `tools: [$($tool:expr),+]` — string literal expressions for `required_tools()`
/// - `tool_field: $field:ident` — config sub-field to clone as a tool name
/// - `tool_field_extra: $field:ident [$($extra:expr),+]` — config field plus extra static tools
/// - `config_json: true` — emit `config_json()` using `serde_json::to_string`
/// - `native: true` — mark this processor as native (pure Rust, no external tools)
/// - `batch: $batch_method:ident` — method on self for batch execution
macro_rules! impl_checker {
    // --- @build: generate the impl block ---
    (@build $processor:ty, $config_field:ident, $desc:expr, $name:expr,
     guard: [$($guard:ident)?],
     tools_kind: $tools_kind:tt,
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     execute: $execute:ident,
    ) => {
        impl $crate::processors::ProductDiscovery for $processor {
            fn description(&self) -> &str {
                $desc
            }

            fn auto_detect(&self, file_index: &$crate::file_index::FileIndex) -> bool {
                impl_checker!(@auto_detect self, file_index, $config_field, [$($guard)?])
            }

            fn required_tools(&self) -> Vec<String> {
                impl_checker!(@tools self, $config_field, $tools_kind)
            }

            fn discover(
                &self,
                graph: &mut $crate::graph::BuildGraph,
                file_index: &$crate::file_index::FileIndex,
                instance_name: &str,
            ) -> anyhow::Result<()> {
                impl_checker!(@discover self, graph, file_index, $config_field, instance_name, [$($guard)?])
            }

            fn execute(&self, product: &$crate::graph::Product) -> anyhow::Result<()> {
                self.$execute(product)
            }

            impl_checker!(@config_json self, $config_field, $cj);

            impl_checker!(@native $native);

            impl_checker!(@batch self, $config_field, [$($batch)?]);

            fn max_jobs(&self) -> Option<usize> {
                self.$config_field.max_jobs
            }
        }
    };

    // --- auto_detect ---
    (@auto_detect $self:ident, $fi:ident, $cfg:ident, [scan_root]) => {
        $crate::processors::scan_root_valid(&$self.$cfg.scan) && !$fi.scan(&$self.$cfg.scan, true).is_empty()
    };
    (@auto_detect $self:ident, $fi:ident, $cfg:ident, [$guard:ident]) => {
        $self.$guard() && !$fi.scan(&$self.$cfg.scan, true).is_empty()
    };
    (@auto_detect $self:ident, $fi:ident, $cfg:ident, []) => {
        !$fi.scan(&$self.$cfg.scan, true).is_empty()
    };

    // --- tools ---
    // No tools
    (@tools $self:ident, $cfg:ident, [none]) => {
        Vec::new()
    };
    // Static tool names (expressions that may reference $self.$cfg)
    (@tools $self:ident, $cfg:ident, [literal: $($tool:expr),+]) => {
        vec![$($tool),+]
    };
    // Dynamic tool name from a config field
    (@tools $self:ident, $cfg:ident, [field: $tool_field:ident]) => {
        vec![$self.$cfg.$tool_field.clone()]
    };
    // Dynamic tool name from a config field plus extra static tools
    (@tools $self:ident, $cfg:ident, [field_and_extra: $tool_field:ident, [$($extra:expr),+]]) => {
        vec![$self.$cfg.$tool_field.clone(), $($extra),+]
    };
    // Optional tool from a config field (Option<String>)
    (@tools $self:ident, $cfg:ident, [option_field: $tool_field:ident]) => {
        $self.$cfg.$tool_field.iter().cloned().collect()
    };

    // --- discover ---
    // Shared body: build dep_inputs and call discover_checker_products
    (@discover_body $self:ident, $graph:ident, $fi:ident, $cfg:ident, $name:expr) => {{
        let mut dep_inputs = $self.$cfg.dep_inputs.clone();
        for ai in &$self.$cfg.dep_auto {
            dep_inputs.extend($crate::processors::config_file_inputs(ai));
        }
        $crate::processors::discover_checker_products(
            $graph, &$self.$cfg.scan, $fi, &dep_inputs, &$self.$cfg, $name,
        )
    }};
    // With scan_root guard (built-in)
    (@discover $self:ident, $graph:ident, $fi:ident, $cfg:ident, $name:expr, [scan_root]) => {{
        if !$crate::processors::scan_root_valid(&$self.$cfg.scan) {
            return Ok(());
        }
        impl_checker!(@discover_body $self, $graph, $fi, $cfg, $name)
    }};
    // With custom guard method
    (@discover $self:ident, $graph:ident, $fi:ident, $cfg:ident, $name:expr, [$guard:ident]) => {{
        if !$self.$guard() {
            return Ok(());
        }
        impl_checker!(@discover_body $self, $graph, $fi, $cfg, $name)
    }};
    // No guard
    (@discover $self:ident, $graph:ident, $fi:ident, $cfg:ident, $name:expr, []) => {{
        impl_checker!(@discover_body $self, $graph, $fi, $cfg, $name)
    }};

    // --- config_json ---
    (@config_json $self:ident, $cfg:ident, true) => {
        fn config_json(&self) -> Option<String> {
            serde_json::to_string(&self.$cfg).ok()
        }
    };
    (@config_json $self:ident, $cfg:ident, false) => {};

    // --- native ---
    (@native true) => {
        fn is_native(&self) -> bool { true }
    };
    (@native false) => {};

    // --- batch ---
    (@batch $self:ident, $cfg:ident, [$batch:ident]) => {
        fn supports_batch(&self) -> bool { self.$cfg.batch }

        fn execute_batch(&self, products: &[&$crate::graph::Product]) -> Vec<anyhow::Result<()>> {
            $crate::processors::execute_checker_batch(
                products,
                |files| self.$batch(files),
            )
        }
    };
    (@batch $self:ident, $cfg:ident, []) => {};

    // --- Entry point: parse options using TT muncher ---
    ($processor:ty,
     config: $config_field:ident,
     description: $desc:expr,
     name: $name:expr,
     execute: $execute:ident
     $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [],
            tools_kind: [none],
            config_json: false,
            native: false,
            batch: [],
            ; $($($rest)*)?
        );
    };

    // Parse guard
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [],
     tools_kind: $tk:tt,
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; guard: $guard:ident $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$guard],
            tools_kind: $tk,
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse tools (literal expressions like "cppcheck".to_string())
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: [none],
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; tools: [$($tool:expr),+] $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: [literal: $($tool),+],
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse tool_field (field name on config struct, e.g. `linter` → self.config.linter.clone())
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: [none],
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; tool_field: $tool_field:ident $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: [field: $tool_field],
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse tool_field_option (Option<String> field — emits option_field tools_kind)
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: [none],
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; tool_field_option: $tool_field:ident $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: [option_field: $tool_field],
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse tool_field_extra (field name + extra static tools, e.g. `tool_field_extra: linter ["python3".to_string()]`)
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: [none],
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; tool_field_extra: $tool_field:ident [$($extra:expr),+] $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: [field_and_extra: $tool_field, [$($extra),+]],
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse config_json
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: $tk:tt,
     config_json: false,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ; config_json: true $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: $tk,
            config_json: true,
            native: $native,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse native
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: $tk:tt,
     config_json: $cj:tt,
     native: false,
     batch: [$($batch:ident)?],
     ; native: true $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: $tk,
            config_json: $cj,
            native: true,
            batch: [$($batch)?],
            ; $($($rest)*)?
        );
    };

    // Parse batch
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: $tk:tt,
     config_json: $cj:tt,
     native: $native:tt,
     batch: [],
     ; batch: $batch_method:ident $(, $($rest:tt)*)?
    ) => {
        impl_checker!(@parse $processor, $config_field, $desc, $name, $execute,
            guard: [$($guard)?],
            tools_kind: $tk,
            config_json: $cj,
            native: $native,
            batch: [$batch_method],
            ; $($($rest)*)?
        );
    };

    // Terminal: no more tokens to parse
    (@parse $processor:ty, $config_field:ident, $desc:expr, $name:expr, $execute:ident,
     guard: [$($guard:ident)?],
     tools_kind: $tk:tt,
     config_json: $cj:tt,
     native: $native:tt,
     batch: [$($batch:ident)?],
     ;
    ) => {
        impl_checker!(@build $processor, $config_field, $desc, $name,
            guard: [$($guard)?],
            tools_kind: $tk,
            config_json: $cj,
            native: $native,
            batch: [$($batch)?],
            execute: $execute,
        );
    };
}


mod simple;

mod aspell;
mod ascii;
mod checkpatch;
mod clippy;
mod clang_tidy;
mod cppcheck;
mod cpplint;
mod duplicate_files;
mod encoding;

mod license_header;
mod make;
mod marp_images;
mod markdownlint;
mod mdl;


mod script;
mod shellcheck;
mod zspell;
mod ijq;
mod ijsonlint;
mod itaplo;
mod iyamllint;
mod iyamlschema;
mod json_schema;
mod luacheck;
pub(crate) mod terms;

pub use simple::SimpleChecker;
pub use aspell::AspellProcessor;
pub use ascii::AsciiProcessor;
pub use checkpatch::CheckpatchProcessor;
pub use clippy::ClippyProcessor;
pub use clang_tidy::ClangTidyProcessor;
pub use cppcheck::CppcheckProcessor;
pub use cpplint::CpplintProcessor;
pub use duplicate_files::DuplicateFilesProcessor;
pub use encoding::EncodingProcessor;
pub use license_header::LicenseHeaderProcessor;
pub use make::MakeProcessor;
pub use marp_images::MarpImagesProcessor;
pub use markdownlint::MarkdownlintProcessor;
pub use mdl::MdlProcessor;


pub use script::ScriptProcessor;
pub use shellcheck::ShellcheckProcessor;
pub use zspell::ZspellProcessor;
pub use ijq::IjqProcessor;
pub use ijsonlint::IjsonlintProcessor;
pub use itaplo::ItaploProcessor;
pub use iyamllint::IyamllintProcessor;
pub use iyamlschema::IyamlschemaProcessor;
pub use luacheck::LuacheckProcessor;
pub use json_schema::JsonSchemaProcessor;
pub use terms::TermsProcessor;
