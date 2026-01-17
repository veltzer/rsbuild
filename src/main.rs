mod builder;
mod checksum;
mod cli;
mod config;
mod linter;
mod processor;
mod template;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use builder::Builder;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut builder = Builder::new()?;

    match cli.command {
        Commands::Build { force, verbose } => {
            builder.build(force, verbose)?;
        }
        Commands::Clean => {
            builder.clean()?;
        }
    }

    Ok(())
}
