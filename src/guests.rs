use clap::Subcommand;

use crate::{client, events::resolve_event_id};

#[derive(Subcommand)]
pub enum Cmd {
    /// List all guests for an event
    List {
        /// Event ID (evt-...) or URL/slug
        #[arg(long)]
        event: String,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::List { event } => list_guests(&resolve_event_id(&event)?),
    }
}

fn list_guests(event_id: &str) -> anyhow::Result<()> {
    let body = client::get("/v1/events/guests/list")?
        .query(&[("event_id", event_id)])
        .send()?
        .error_for_status()?
        .text()?;

    println!("{body}");
    Ok(())
}
