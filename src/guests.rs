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

        /// One of: approved, session, pending_approval, invited, declined, waitlist
        #[arg(long)]
        approval_status: Option<String>,

        /// Fetch a single page from this cursor instead of auto-paginating
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { event, id } => get_guest(&resolve_event_id(&event)?, &id),
        Cmd::List {
            event,
            approval_status,
            cursor,
        } => list_guests(&resolve_event_id(&event)?, approval_status, cursor),
    }
}

fn list_guests(
    event_id: &str,
    approval_status: Option<String>,
    cursor: Option<String>,
) -> anyhow::Result<()> {
    let path = "/v1/events/guests/list";

    let mut params: Vec<(&str, String)> = vec![("event_id", event_id.to_owned())];
    if let Some(approval_status) = approval_status {
        params.push(("approval_status", approval_status));
    }

    if let Some(cursor) = cursor {
        params.push(("pagination_cursor", cursor));
        let body = client::send(client::get(path)?.query(&params))?;
        println!("{body}");
        return Ok(());
    }

    let entries = client::paginate(path, &params)?;
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
