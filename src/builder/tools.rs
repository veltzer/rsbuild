use std::io::Write;
use std::process::Command;
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
        "pyrefly" => Some("pip install pyrefly"),
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
        "jq" => Some("apt install jq"),
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
        let install_yes = matches!(&action, ToolsAction::Install { yes: true, .. });

        let mut tool_map: std::collections::BTreeMap<String, Vec<String>> = std::collections::BTreeMap::new();
        for name in sorted_keys(&processors) {
            if !show_all && !self.config.processor.is_enabled(name) {
                continue;
            }
            for tool in processors[name].required_tools() {
                let procs = tool_map.entry(tool).or_default();
                if !procs.contains(name) {
                    procs.push(name.clone());
                }
            }
        }

        match action {
            ToolsAction::List { .. } => {
                if crate::json_output::is_json_mode() {
                    let entries: Vec<crate::json_output::ToolListEntry> = tool_map.iter()
                        .map(|(tool, procs)| crate::json_output::ToolListEntry {
                            tool: tool.clone(),
                            processors: procs.clone(),
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&entries)?);
                    return Ok(());
                }

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
            ToolsAction::Install { name, .. } => {
                let to_install: Vec<(String, &str)> = if let Some(ref name) = name {
                    let hint = install_hint(name).ok_or_else(|| {
                        anyhow::anyhow!("No install hint known for tool '{}'", name)
                    })?;
                    vec![(name.clone(), hint)]
                } else {
                    let mut missing = Vec::new();
                    for tool in tool_map.keys() {
                        if which::which(tool).is_err() {
                            if let Some(hint) = install_hint(tool) {
                                missing.push((tool.clone(), hint));
                            } else {
                                println!("{} {} (no install hint known)", tool, color::yellow("skipped"));
                            }
                        }
                    }
                    if missing.is_empty() {
                        println!("{}", color::green("All tools are already installed."));
                        return Ok(());
                    }
                    missing
                };

                println!("The following tools will be installed:");
                for (tool, hint) in &to_install {
                    println!("  {} — {}", color::bold(tool), color::dim(hint));
                }

                if !install_yes {
                    print!("Proceed? [y/N] ");
                    std::io::stdout().flush()?;
                    let mut answer = String::new();
                    std::io::stdin().read_line(&mut answer)?;
                    let answer = answer.trim().to_lowercase();
                    if answer != "y" && answer != "yes" {
                        println!("Aborted.");
                        return Ok(());
                    }
                }

                let mut any_failed = false;
                for (tool, hint) in &to_install {
                    println!("Installing {} — running: {}", color::bold(tool), color::dim(hint));
                    let status = Command::new("sh")
                        .arg("-c")
                        .arg(hint)
                        .status()?;
                    if status.success() {
                        println!("{} {}", tool, color::green("installed"));
                    } else {
                        println!("{} {} (exit code {})", tool, color::red("failed"),
                            status.code().map_or("unknown".to_string(), |c| c.to_string()));
                        any_failed = true;
                    }
                }
                if any_failed {
                    return Err(crate::exit_code::RsbError::new(
                        crate::exit_code::RsbExitCode::ToolError,
                        "Some tools failed to install",
                    ).into());
                }
            }
        }

        Ok(())
    }
}
