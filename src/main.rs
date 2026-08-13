use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

use rex_skills::commands::codebase;
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
    Skills {
        /// Which agent tool to install the bundle for.
        #[arg(long, value_enum, default_value_t = Vendor::Claude)]
        vendor: Vendor,
    },

    /// Write a tree outline of the working directory to CODEBASE.md.
    Codebase,
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
    }

    Ok(())
}
