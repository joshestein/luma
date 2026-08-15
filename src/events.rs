use anyhow::Context;
use clap::{ArgGroup, Subcommand};

use crate::client;

#[derive(Subcommand)]
pub enum Cmd {
    /// Get event by ID or URL
    #[command(group = ArgGroup::new("target").required(true).multiple(false))]
    Get {
        /// Event ID, starts with 'evt-'
        #[arg(long, group = "target")]
        id: Option<String>,

        /// Event URL or slug
        #[arg(long, group = "target")]
        url: Option<String>,
    },

    /// List all events
    List {
        /// ISO 8601 datetime
        #[arg(long)]
        before: Option<String>,

        /// ISO 8601 datetime
        #[arg(long)]
        after: Option<String>,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { id, url } => {
            let event_id = match (id, url) {
                (Some(id), None) => id,
                (None, Some(url)) => lookup_event_id(&url)?,
                _ => unreachable!("need either an event ID or URL"),
            };

            get_event(&event_id)
        }
        Cmd::List { before, after } => list_events(before, after),
    }
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
        .query(&[("slug", &slug)])
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
