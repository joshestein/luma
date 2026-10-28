use clap::Subcommand;

use crate::client;

#[derive(Subcommand)]
pub enum Cmd {
    /// List all contacts for the calendar
    List {
        /// Only relevant for calendar memberships
        #[arg(long, value_parser = ["approved", "pending", "approved-pending-payment", "declined"])]
        membership_status: Option<String>,

        /// Tag name or tag ID. Repeat to match contacts with any of the tags.
        #[arg(long)]
        tags: Vec<String>,

        /// Search over names and emails
        #[arg(long)]
        query: Option<String>,

        /// Fetch a single page from this cursor instead of auto-paginating
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::List {
            membership_status,
            tags,
            query,
            cursor,
        } => list_contacts(membership_status, tags, query, cursor),
    }
}

fn list_contacts(
    membership_status: Option<String>,
    tags: Vec<String>,
    query: Option<String>,
    cursor: Option<String>,
) -> anyhow::Result<()> {
    let path = "/v1/calendars/contacts/list";

    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(membership_status) = membership_status {
        params.push(("membership_status", membership_status));
    }
    params.extend(tags.into_iter().map(|tag| ("tags", tag)));
    if let Some(query) = query {
        params.push(("query", query));
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
