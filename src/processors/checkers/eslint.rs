simple_checker!(EslintProcessor, crate::config::EslintConfig,
    "Lint JavaScript/TypeScript files with eslint",
    crate::processors::names::ESLINT,
    tool_field_extra: command ["node".to_string()],
);
