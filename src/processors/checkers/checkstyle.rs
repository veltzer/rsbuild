simple_checker!(CheckstyleProcessor, crate::config::CheckstyleConfig,
    "Check Java code style with checkstyle",
    crate::processors::names::CHECKSTYLE,
    tools: ["checkstyle".to_string()],
);
