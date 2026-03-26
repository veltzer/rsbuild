simple_checker!(SlidevProcessor, crate::config::SlidevConfig,
    "Build Slidev presentations",
    crate::processors::names::SLIDEV,
    tools: ["slidev".to_string(), "node".to_string()], subcommand: "build",
);
