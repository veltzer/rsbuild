simple_checker!(RuffProcessor, crate::config::RuffConfig,
    "Lint Python files with ruff",
    crate::processors::names::RUFF,
    tool_field: linter, subcommand: "check",
);
