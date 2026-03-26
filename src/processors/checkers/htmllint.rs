simple_checker!(HtmllintProcessor, crate::config::HtmllintConfig,
    "Lint HTML files with htmllint",
    crate::processors::names::HTMLLINT,
    tools: ["htmllint".to_string(), "node".to_string()],
);
