use std::env;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "luma")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCmd,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Verify credentials work
    Check,
}

fn api_key() -> anyhow::Result<String> {
    env::var("LUMA_API_KEY").context("LUMA_API_KEY not set")
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCmd::Check => {
                api_key()?;
            }
        },
    }

    Ok(())
}
