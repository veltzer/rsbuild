simple_checker!(TaploProcessor, crate::config::TaploConfig,
    "Check TOML files with taplo",
    crate::processors::names::TAPLO,
    tool_field: linter, subcommand: "check",
);
