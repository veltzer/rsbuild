simple_checker!(JshintProcessor, crate::config::JshintConfig,
    "Lint JavaScript files with jshint",
    crate::processors::names::JSHINT,
    tool_field_extra: linter ["node".to_string()],
);
