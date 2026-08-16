use clap::Subcommand;

use crate::{client, events::resolve_event_id};

#[derive(Subcommand)]
pub enum Cmd {
    /// Get detailed information for a single guest
    Get {
        /// Event ID (evt-...) or URL/slug
        #[arg(long)]
        event: String,

        /// Guest identifier - the guest ID (gst-), a ticket key, a guest key (g-), or the user's email.
        #[arg(long)]
        id: String,
    },

    /// List all guests for an event
    List {
        /// Event ID (evt-...) or URL/slug
        #[arg(long)]
        event: String,

        /// Fetch a single page from this cursor instead of auto-paginating
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { event, id } => get_guest(&resolve_event_id(&event)?, &id),
        Cmd::List { event, cursor } => list_guests(&resolve_event_id(&event)?, cursor),
    }
}

fn list_guests(event_id: &str, cursor: Option<String>) -> anyhow::Result<()> {
    let path = "/v1/events/guests/list";

    if let Some(cursor) = cursor {
        let body = client::send(
            client::get(path)?
                .query(&[("event_id", event_id), ("pagination_cursor", &cursor)]),
        )?;
        println!("{body}");
        return Ok(());
    }

    let entries = client::paginate(path, &[("event_id", event_id.to_owned())])?;
    println!("{}", serde_json::to_string(&entries)?);
    Ok(())
}

fn get_guest(event_id: &str, id: &str) -> anyhow::Result<()> {
    let body = client::send(
        client::get("/v1/events/guests/get")?.query(&[("event_id", event_id), ("id", id)]),
    )?;

    println!("{body}");
    Ok(())
}
