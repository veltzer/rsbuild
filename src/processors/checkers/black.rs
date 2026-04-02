simple_checker!(BlackProcessor, crate::config::BlackConfig,
    "Check Python formatting with black",
    crate::processors::names::BLACK,
    tools: ["black".to_string(), "python3".to_string()],
    prepend_args: ["--check"],
);
