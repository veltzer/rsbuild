simple_checker!(JqProcessor, crate::config::JqConfig,
    "Validate JSON files with jq",
    crate::processors::names::JQ,
    tool_field: linter, prepend_args: ["empty"],
);
