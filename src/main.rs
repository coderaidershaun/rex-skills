use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use rex_skills::commands::codebase;
use rex_skills::commands::commands;
use rex_skills::commands::skills::{self, Vendor};

#[derive(Parser)]
#[command(
    name = "rex",
    version,
    about = "Install the rex agent skill bundle and generate a CODEBASE.md tree"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install the rex agent skill bundle into the working directory.
    #[command(visible_alias = "init")]
    Skills {
        /// Which agent tool to install the bundle for.
        #[arg(long, value_enum, default_value_t = Vendor::Claude)]
        vendor: Vendor,
    },

    /// Write a tree outline of the working directory to CODEBASE.md.
    Codebase,

    /// List every available command with its full invocation path and description.
    Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let cwd = PathBuf::from(".");

    match cli.command {
        Command::Skills { vendor } => {
            skills::run(&cwd, vendor).context("rex skills failed")?;
        }
        Command::Codebase => {
            codebase::run(&cwd).context("rex codebase failed")?;
        }
        Command::Commands => {
            commands::run(&Cli::command());
        }
    }

    Ok(())
}
