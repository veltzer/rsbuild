simple_checker!(EslintProcessor, crate::config::EslintConfig,
    "Lint JavaScript/TypeScript files with eslint",
    crate::processors::names::ESLINT,
    tool_field_extra: linter ["node".to_string()],
);
