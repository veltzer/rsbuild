simple_checker!(StylelintProcessor, crate::config::StylelintConfig,
    "Lint CSS/SCSS files with stylelint",
    crate::processors::names::STYLELINT,
    tool_field_extra: command ["node".to_string()],
);
