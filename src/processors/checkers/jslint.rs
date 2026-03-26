simple_checker!(JslintProcessor, crate::config::JslintConfig,
    "Lint JavaScript files with jslint",
    crate::processors::names::JSLINT,
    tools: ["jslint".to_string(), "node".to_string()],
);
