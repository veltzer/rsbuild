simple_checker!(HtmlhintProcessor, crate::config::HtmlhintConfig,
    "Lint HTML files with htmlhint",
    crate::processors::names::HTMLHINT,
    tool_field_extra: command ["node".to_string()],
);
