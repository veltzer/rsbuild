mod builder;
mod cli;
mod config;
mod executor;
mod graph;
mod object_store;
mod processors;

use anyhow::{bail, Result};
use clap::Parser;
use cli::{CacheAction, Cli, Commands, parse_shell, print_completions};
use config::Config;
use builder::Builder;
use object_store::ObjectStore;
use std::env;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { force, jobs } => {
            let mut builder = Builder::new()?;
            builder.build(force, cli.verbose, jobs)?;
        }
        Commands::Clean => {
            let mut builder = Builder::new()?;
            builder.clean()?;
        }
        Commands::Cache { action } => {
            let project_root = env::current_dir()?;
            let config = Config::load(&project_root)?;
            let mut store = ObjectStore::new(project_root, config.cache.restore_method)?;

            match action {
                CacheAction::Clear => {
                    store.clear()?;
                    println!("Cache cleared.");
                }
                CacheAction::Size => {
                    let (bytes, count) = store.size()?;
                    println!("Cache size: {} bytes ({} objects)", bytes, count);
                }
                CacheAction::Trim => {
                    let (bytes, count) = store.trim()?;
                    store.save()?;
                    println!("Removed {} bytes ({} unreferenced objects)", bytes, count);
                }
            }
        }
        Commands::Complete { shells } => {
            let shells_to_generate = if shells.is_empty() {
                // Load from config file
                let config = Config::load(&env::current_dir()?)?;
                let mut parsed_shells = Vec::new();
                for shell_name in &config.completions.shells {
                    match parse_shell(shell_name) {
                        Some(shell) => parsed_shells.push(shell),
                        None => bail!("Unknown shell in config: {}", shell_name),
                    }
                }
                parsed_shells
            } else {
                shells
            };

            for shell in shells_to_generate {
                print_completions(shell);
            }
        }
        Commands::Graph { format, view } => {
            let builder = Builder::new()?;
            if let Some(viewer) = view {
                builder.view_graph(viewer)?;
            } else {
                builder.print_graph(format)?;
            }
        }
    }

    Ok(())
}
