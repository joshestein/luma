use anyhow::Context;
use clap::Subcommand;

use crate::client;

#[derive(clap::Args)]
pub struct OptionalEventFields {
    #[arg(short, long)]
    description_md: Option<String>,

    #[arg(short, long)]
    end_at: Option<String>,

    #[arg(short, long)]
    max_capacity: Option<u8>,

    #[arg(short, long)]
    visibility: Option<String>,
}

#[derive(Subcommand)]
pub enum Cmd {
    /// Get event by ID or URL
    Get {
        /// Event ID (evt-...) or URL/slug
        #[arg(short, long)]
        event: String,
    },

    /// List all events
    List {
        /// ISO 8601 datetime
        #[arg(short, long)]
        before: Option<String>,

        /// ISO 8601 datetime
        #[arg(short, long)]
        after: Option<String>,
    },

    Create {
        #[arg(short, long)]
        name: String,

        #[arg(short, long)]
        start_at: String,

        #[arg(short, long)]
        timezone: String,

        #[command(flatten)]
        rest: OptionalEventFields,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { event } => get_event(&resolve_event_id(&event)?),
        Cmd::List { before, after } => list_events(before, after),
        Cmd::Create {
            name,
            start_at,
            timezone,
            rest,
        } => todo!(),
    }
}

pub fn resolve_event_id(event: &str) -> anyhow::Result<String> {
    if event.starts_with("evt-") {
        Ok(event.to_owned())
    } else {
        lookup_event_id(event)
    }
}

fn get_event(event_id: &str) -> anyhow::Result<()> {
    let body = client::get("/v1/events/get")?
        .query(&[("event_id", event_id)])
        .send()?
        .error_for_status()?
        .text()?;

    println!("{body}");
    Ok(())
}

fn list_events(before: Option<String>, after: Option<String>) -> anyhow::Result<()> {
    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(before) = before {
        params.push(("before", before));
    }
    if let Some(after) = after {
        params.push(("after", after));
    }

    let body = client::get("/v1/calendars/events/list")?
        .query(&params)
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
        .query(&[("slug", slug)])
        .send()?
        .error_for_status()?
        .json::<serde_json::Value>()?;

    let entity = &body["entity"];
    match entity["type"].as_str() {
        Some("event") => entity["event"]["id"]
            .as_str()
            .map(str::to_owned)
            .context("event not found for URL"),
        Some("calendar") => {
            anyhow::bail!("'{slug}' is a calendar, not an event")
        }
        Some(other) => anyhow::bail!("'{slug}' is {other}, not an event"),
        None => anyhow::bail!("no entity found for '{slug}'"),
    }
}
