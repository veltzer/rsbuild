simple_checker!(YqProcessor, crate::config::YqConfig,
    "Validate YAML files with yq",
    crate::processors::names::YQ,
    tools: ["yq".to_string()], subcommand: ".",
);
