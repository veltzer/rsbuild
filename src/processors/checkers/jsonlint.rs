simple_checker!(JsonlintProcessor, crate::config::JsonlintConfig,
    "Lint JSON files with jsonlint",
    crate::processors::names::JSONLINT,
    tool_field_extra: command ["python3".to_string()],
);
