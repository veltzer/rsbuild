simple_checker!(StandardProcessor, crate::config::StandardConfig,
    "Check JavaScript style with standard",
    crate::processors::names::STANDARD,
    tools: ["standard".to_string(), "node".to_string()],
);
