simple_checker!(XmllintProcessor, crate::config::XmllintConfig,
    "Validate XML files with xmllint",
    crate::processors::names::XMLLINT,
    tools: ["xmllint".to_string()], subcommand: "--noout",
);
