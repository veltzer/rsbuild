simple_checker!(PhpLintProcessor, crate::config::PhpLintConfig,
    "Check PHP syntax with php -l",
    crate::processors::names::PHP_LINT,
    tools: ["php".to_string()], subcommand: "-l",
);
