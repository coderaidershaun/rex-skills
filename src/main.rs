use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};

use rex_skills::commands::codebase;
use rex_skills::commands::commands;
use rex_skills::commands::skills::{self, Vendor};
use rex_skills::commands::smells;

#[derive(Parser)]
#[command(
    name = "rex",
    version,
    about = "Install the rex agent skill bundle and print a tree of a codebase"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Install the rex agent skill bundle and add a codebase-map section to AGENTS.md.
    #[command(visible_alias = "init")]
    Skills {
        /// Which agent tool to install the bundle for.
        #[arg(long, value_enum, default_value_t = Vendor::Claude)]
        vendor: Vendor,
    },

    /// Print a tree outline of the working directory.
    Codebase(codebase::CodebaseOptions),

    /// List every `SMELL` flag comment in the Rust files as `path:line`, then the count.
    Smells,

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
        Command::Codebase(options) => {
            codebase::run(&cwd, options).context("rex codebase failed")?;
        }
        Command::Smells => {
            smells::run(&cwd).context("rex smells failed")?;
        }
        Command::Commands => {
            commands::run(&Cli::command());
        }
    }

    Ok(())
}
