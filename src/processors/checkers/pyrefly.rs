simple_checker!(PyreflyProcessor, crate::config::PyreflyConfig,
    "Type-check Python files with pyrefly",
    crate::processors::names::PYREFLY,
    tool_field: linter, subcommand: "check",
    prepend_args: ["--disable-project-excludes-heuristics"],
);
