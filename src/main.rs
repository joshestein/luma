use anyhow::Context;
use clap::{ArgGroup, Parser, Subcommand};

mod client;

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

fn check_auth() -> anyhow::Result<()> {
    let resp = client::get("/v1/users/get-self")?.send()?;
    resp.error_for_status()?;

    println!("Authenticated!");
    Ok(())
}

fn get_event(event_id: &str) -> anyhow::Result<()> {
    let body = client::get("/v1/events/get")?
        .query(&[("event_id", &event_id)])
        .send()?
        .error_for_status()?
        .text()?;

    println!("{body}");
    Ok(())
}

fn lookup_event_id(event_url: &str) -> anyhow::Result<String> {
    let slug = event_url
        .trim_end_matches("/")
        .rsplit("/")
        .next()
        .and_then(|s| s.split(['?', '#']).next())
        .filter(|s| !s.is_empty())
        .context("could not extract slug from URL")?;

    let body = client::get("/v1/entities/lookup")?
        .query(&[("slug", &slug)])
        .send()?
        .error_for_status()?
        .json::<serde_json::Value>()?;

    body["entity"]["event"]["id"]
        .as_str()
        .map(str::to_owned)
        .context("event not found for URL")
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
                    (None, Some(url)) => lookup_event_id(&url)?,
                    _ => unreachable!("need either an event ID or URL"),
                };

                get_event(&event_id)?;
            }
        },
    }

    Ok(())
}
