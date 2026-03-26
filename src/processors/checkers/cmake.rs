simple_checker!(CmakeProcessor, crate::config::CmakeConfig,
    "Lint CMakeLists.txt files with cmake --lint",
    crate::processors::names::CMAKE,
    tools: ["cmake".to_string()], subcommand: "--lint",
);
