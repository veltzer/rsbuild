simple_checker!(YamllintProcessor, crate::config::YamllintConfig,
    "Lint YAML files with yamllint",
    crate::processors::names::YAMLLINT,
    tool_field_extra: command ["python3".to_string()],
);
