simple_checker!(MypyProcessor, crate::config::MypyConfig,
    "Type-check Python files with mypy",
    crate::processors::names::MYPY,
    tool_field_extra: linter ["python3".to_string()],
);
