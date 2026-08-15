use anyhow::Context;
use clap::Subcommand;

use crate::client;

macro_rules! insert_opt {
    ($map:expr, $key:literal, $val:expr) => {
        if let Some(v) = $val {
            $map.insert($key.into(), v.into());
        }
    };
}

#[derive(clap::Args)]
pub struct OptionalEventFields {
    #[arg(short, long)]
    description_md: Option<String>,

    #[arg(short, long)]
    end_at: Option<String>,

    #[arg(short, long)]
    max_capacity: Option<u16>,

    #[arg(short, long)]
    visibility: Option<String>,
}

#[derive(clap::Args)]
pub struct CreateArgs {
    #[arg(short, long)]
    name: String,

    #[arg(short, long)]
    start_at: String,

    #[arg(short, long)]
    timezone: String,

    #[command(flatten)]
    rest: OptionalEventFields,
}

#[derive(clap::Args)]
pub struct UpdateArgs {
    #[arg(long)]
    event_id: String,

    #[arg(long)]
    name: Option<String>,

    #[arg(short, long)]
    start_at: Option<String>,

    #[arg(short, long)]
    timezone: Option<String>,

    #[command(flatten)]
    rest: OptionalEventFields,
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

    Create(CreateArgs),
    Update(UpdateArgs),
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { event } => get_event(&resolve_event_id(&event)?),
        Cmd::List { before, after } => list_events(before, after),
        Cmd::Create(args) => create_event(args),
        Cmd::Update(args) => update_event(args),
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

fn create_event(args: CreateArgs) -> anyhow::Result<()> {
    let CreateArgs {
        name,
        start_at,
        timezone,
        rest:
            OptionalEventFields {
                description_md,
                end_at,
                max_capacity,
                visibility,
            },
    } = args;

    let mut body = serde_json::Map::new();
    body.insert("name".into(), name.into());
    body.insert("start_at".into(), start_at.into());
    body.insert("timezone".into(), timezone.into());

    insert_opt!(body, "description_md", description_md);
    insert_opt!(body, "end_at", end_at);
    insert_opt!(body, "max_capacity", max_capacity);
    insert_opt!(body, "visibility", visibility);

    let resp = client::post("/v1/events/create")?
        .json(&body)
        .send()?
        .error_for_status()?
        .text()?;

    println!("{resp}");
    Ok(())
}

fn update_event(args: UpdateArgs) -> anyhow::Result<()> {
    let UpdateArgs {
        event_id,
        name,
        start_at,
        timezone,
        rest:
            OptionalEventFields {
                description_md,
                end_at,
                max_capacity,
                visibility,
            },
    } = args;

    let mut body = serde_json::Map::new();
    body.insert("event_id".into(), event_id.into());

    insert_opt!(body, "name", name);
    insert_opt!(body, "start_at", start_at);
    insert_opt!(body, "timezone", timezone);
    insert_opt!(body, "description_md", description_md);
    insert_opt!(body, "end_at", end_at);
    insert_opt!(body, "max_capacity", max_capacity);
    insert_opt!(body, "visibility", visibility);

    let resp = client::post("/v1/events/update")?
        .json(&body)
        .send()?
        .error_for_status()?
        .text()?;

    println!("{resp}");
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
