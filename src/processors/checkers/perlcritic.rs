simple_checker!(PerlcriticProcessor, crate::config::PerlcriticConfig,
    "Analyze Perl code with perlcritic",
    crate::processors::names::PERLCRITIC,
    tools: ["perlcritic".to_string(), "perl".to_string()],
);
