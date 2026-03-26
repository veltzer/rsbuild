simple_checker!(PylintProcessor, crate::config::PylintConfig,
    "Lint Python files with pylint",
    crate::processors::names::PYLINT,
    tools: ["pylint".to_string(), "python3".to_string()],
);
