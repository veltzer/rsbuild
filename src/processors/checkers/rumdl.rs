simple_checker!(RumdlProcessor, crate::config::RumdlConfig,
    "Lint Markdown files using rumdl",
    crate::processors::names::RUMDL,
    tool_field: command, subcommand: "check",
);
