simple_checker!(TidyProcessor, crate::config::TidyConfig,
    "Validate HTML files with tidy",
    crate::processors::names::TIDY,
    tools: ["tidy".to_string()], subcommand: "-errors",
);
