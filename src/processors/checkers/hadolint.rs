simple_checker!(HadolintProcessor, crate::config::HadolintConfig,
    "Lint Dockerfiles with hadolint",
    crate::processors::names::HADOLINT,
    tools: ["hadolint".to_string()],
);
