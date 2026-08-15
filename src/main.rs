use std::env;

use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand};

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
    Events {
        #[command(subcommand)]
        command: EventsCmd,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Verify credentials work
    Check,
}

#[derive(Subcommand)]
enum EventsCmd {
    /// Get event by ID or URL
    #[command(group = ArgGroup::new("target").required(true).multiple(false))]
    Get {
        #[arg(long, group = "target")]
        id: Option<String>,

        #[arg(long, group = "target")]
        url: Option<String>,
    },
}

fn api_key() -> anyhow::Result<String> {
    env::var("LUMA_API_KEY").context("LUMA_API_KEY not set")
}

fn check_auth() -> anyhow::Result<()> {
    let key = api_key()?;

    let client = reqwest::blocking::Client::new();
    let resp = client
        .get("https://public-api.luma.com/v1/users/get-self")
        .header("x-luma-api-key", key)
        .send()?;

    resp.error_for_status()?;

    println!("Authenticated!");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCmd::Check => {
                check_auth()?;
            }
        },
        Commands::Events { command } => match command {
            EventsCmd::Get { id, url } => {
                let event_id = match (id, url) {
                    (Some(id), None) => id,
                    (None, Some(url)) => "TODO: resolve event ID from url".to_owned(),
                    _ => unreachable!("need either an event ID or URL"),
                };

                println!("{}", event_id)
            }
        },
    }

    Ok(())
}
