simple_checker!(JsonlintProcessor, crate::config::JsonlintConfig,
    "Lint JSON files with jsonlint",
    crate::processors::names::JSONLINT,
    tool_field_extra: linter ["python3".to_string()],
);
