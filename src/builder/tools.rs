use anyhow::Result;
use crate::cli::ToolsAction;
use crate::color;
use crate::tool_lock;
use super::{Builder, sorted_keys};

/// Return an install hint for a missing tool, if known.
fn install_hint(tool: &str) -> Option<&'static str> {
    match tool {
        "ruff" => Some("pip install ruff"),
        "pylint" => Some("pip install pylint"),
        "mypy" => Some("pip install mypy"),
        "black" => Some("pip install black"),
        "shellcheck" => Some("apt install shellcheck"),
        "cppcheck" => Some("apt install cppcheck"),
        "clang-tidy" => Some("apt install clang-tidy"),
        "gcc" => Some("apt install gcc"),
        "g++" => Some("apt install g++"),
        "clang" => Some("apt install clang"),
        "clang++" => Some("apt install clang"),
        "make" => Some("apt install make"),
        "cargo" => Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
        "rustc" => Some("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"),
        "aspell" => Some("apt install aspell"),
        "yamllint" => Some("pip install yamllint"),
        "jsonlint" => Some("npm install -g jsonlint"),
        "taplo" => Some("cargo install taplo-cli"),
        "mdl" => Some("gem install mdl"),
        "pandoc" => Some("apt install pandoc"),
        "sass" => Some("npm install -g sass"),
        "protoc" => Some("apt install protobuf-compiler"),
        "pytest" => Some("pip install pytest"),
        "rumdl" => Some("cargo install rumdl"),
        _ => None,
    }
}

impl Builder {
    /// Verify tool versions against .tools.versions lock file.
    /// Called at the start of build unless --ignore-tool-versions is passed.
    pub fn verify_tool_versions(&self) -> Result<()> {
        let processors = self.create_processors()?;
        let config = &self.config;
        let tool_commands = tool_lock::collect_tool_commands(
            &processors,
            &|name| config.processor.is_enabled(name),
        );
        if tool_commands.is_empty() {
            return Ok(());
        }
        tool_lock::verify_lock_file(&tool_commands)
    }

    /// Handle `rsb tools` subcommands
    pub fn tools(&self, action: ToolsAction) -> Result<()> {
        let processors = self.create_processors()?;

        let show_all = matches!(&action, ToolsAction::List { all: true } | ToolsAction::Check { all: true });

        let mut tool_map: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        for name in sorted_keys(&processors) {
            if !show_all && !self.config.processor.is_enabled(name) {
                continue;
            }
            for tool in processors[name].required_tools() {
                let procs = tool_map.entry(tool).or_default();
                if !procs.contains(&name) {
                    procs.push(name.clone());
                }
            }
        }

        match action {
            ToolsAction::List { .. } => {
                for (tool, procs) in &tool_map {
                    println!("{} ({})", tool, procs.join(", "));
                }
            }
            ToolsAction::Check { .. } => {
                let mut any_missing = false;
                for (tool, procs) in &tool_map {
                    let procs_str = procs.join(", ");
                    if let Ok(path) = which::which(tool) {
                        let path_str = path.display().to_string();
                        println!("{} ({}) {} {}", tool, procs_str, color::green("found"), color::dim(&path_str));
                    } else {
                        let hint = install_hint(tool)
                            .map(|h| format!(" — install with: {}", color::dim(h)))
                            .unwrap_or_default();
                        println!("{} ({}) {}{}", tool, procs_str, color::red("missing"), hint);
                        any_missing = true;
                    }
                }
                if any_missing {
                    return Err(crate::exit_code::RsbError::new(
                        crate::exit_code::RsbExitCode::ToolError,
                        "Some required tools are missing",
                    ).into());
                }
            }
            ToolsAction::Lock { check } => {
                let config = &self.config;
                let tool_commands = tool_lock::collect_tool_commands(
                    &processors,
                    &|name| config.processor.is_enabled(name),
                );

                if check {
                    tool_lock::verify_lock_file(&tool_commands)?;
                    println!("{}", color::green("Tool versions match lock file."));
                } else {
                    let lock = tool_lock::create_lock(&tool_commands)?;
                    for (name, info) in &lock.tools {
                        let first_line = info.version_output.lines().next().unwrap_or("");
                        println!("{} {} {}", name, color::green("locked"), color::dim(first_line));
                    }
                    tool_lock::write_lock_file(&lock)?;
                    println!("Wrote {}", color::bold(".tools.versions"));
                }
            }
        }

        Ok(())
    }
}
