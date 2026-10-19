use anyhow::{Context, bail};
use chrono::{DateTime, NaiveDate, SecondsFormat, TimeDelta, Utc};
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

    #[arg(short, long)]
    cover_url: Option<String>,

    /// One of: public, guests-only
    #[arg(short, long)]
    location_visibility: Option<String>,
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

        /// One of: approved, pending
        #[arg(short, long)]
        status: Option<String>,

        /// Fetch a single page from this cursor instead of auto-paginating
        #[arg(long)]
        cursor: Option<String>,
    },

    Create {
        #[command(flatten)]
        args: CreateArgs,

        #[arg(long)]
        confirm: bool,
    },
    Update {
        #[command(flatten)]
        args: UpdateArgs,

        #[arg(long)]
        confirm: bool,
    },

    Clone {
        #[arg(short, long)]
        event: String,

        #[arg(short, long)]
        name: Option<String>,

        #[arg(short, long)]
        start_at: Option<String>,

        #[arg(short, long, default_value = "private")]
        visibility: String,

        #[arg(long)]
        confirm: bool,

        #[arg(short, long)]
        description_md: Option<String>,
    },
}

pub fn run(cmd: Cmd) -> anyhow::Result<()> {
    match cmd {
        Cmd::Get { event } => get_event(&resolve_event_id(&event)?),
        Cmd::List {
            before,
            after,
            status,
            cursor,
        } => list_events(before, after, status, cursor),
        Cmd::Create { args, confirm } => create_event(args, confirm),
        Cmd::Update { args, confirm } => update_event(args, confirm),
        Cmd::Clone {
            event,
            name,
            start_at,
            visibility,
            confirm,
            description_md,
        } => clone_event(
            &resolve_event_id(&event)?,
            CloneOverrides {
                name,
                start_at,
                visibility,
                description_md,
            },
            confirm,
        ),
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
    let body = client::send(client::get("/v1/events/get")?.query(&[("event_id", event_id)]))?;

    println!("{body}");
    Ok(())
}

fn list_events(
    before: Option<String>,
    after: Option<String>,
    status: Option<String>,
    cursor: Option<String>,
) -> anyhow::Result<()> {
    let path = "/v1/calendars/events/list";

    let mut params: Vec<(&str, String)> = Vec::new();
    if let Some(before) = before {
        params.push(("before", before));
    }
    if let Some(after) = after {
        params.push(("after", after));
    }
    if let Some(status) = status {
        params.push(("status", status));
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

fn build_create_body(args: CreateArgs) -> serde_json::Map<String, serde_json::Value> {
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
                cover_url,
                location_visibility,
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
    insert_opt!(body, "cover_url", cover_url);
    insert_opt!(body, "location_visibility", location_visibility);

    body
}

/// POST `body` to `path`, gated on `--confirm`. Without it, pretty-print the
/// request body to stderr and refuse (non-zero exit) without calling the API,
/// so stdout only ever carries a real server response.
fn submit(
    confirm: bool,
    path: &str,
    body: &serde_json::Map<String, serde_json::Value>,
) -> anyhow::Result<String> {
    if !confirm {
        eprintln!("{}", serde_json::to_string_pretty(body)?);
        bail!("refusing to write without --confirm");
    }
    client::send(client::post(path)?.json(body))
}

fn create_event(args: CreateArgs, confirm: bool) -> anyhow::Result<()> {
    let body = build_create_body(args);
    let resp = submit(confirm, "/v1/events/create", &body)?;
    report_created(&resp)
}

/// Create returns only `{id}`. Fetch the event so we can echo the public URL
/// alongside it, letting the caller preview the event right after creating it.
fn report_created(create_resp: &str) -> anyhow::Result<()> {
    let created: serde_json::Value = serde_json::from_str(create_resp)
        .with_context(|| format!("unexpected create response: {create_resp}"))?;
    let id = created["id"]
        .as_str()
        .with_context(|| format!("create response missing 'id': {create_resp}"))?;

    // The event exists now, so a failed URL lookup must not hide its id: a
    // caller who sees only an error would retry and create a duplicate.
    let out = match public_url(id) {
        Ok(url) => serde_json::json!({ "id": id, "url": url }),
        Err(e) => {
            eprintln!("warning: created {id}, but could not get its URL: {e:#}");
            serde_json::json!({ "id": id })
        }
    };
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn public_url(event_id: &str) -> anyhow::Result<String> {
    let got = client::send(client::get("/v1/events/get")?.query(&[("event_id", event_id)]))?;
    let event: serde_json::Value = serde_json::from_str(&got)?;
    // Spec types `url` as a bare string with no example; treat a leading
    // scheme as an already-absolute URL, otherwise it's a slug under luma.com.
    let url_field = event["url"]
        .as_str()
        .context("event/get response missing 'url'")?;
    Ok(if url_field.starts_with("http") {
        url_field.to_string()
    } else {
        format!("https://luma.com/{url_field}")
    })
}

fn update_event(args: UpdateArgs, confirm: bool) -> anyhow::Result<()> {
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
                cover_url,
                location_visibility,
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
    insert_opt!(body, "cover_url", cover_url);
    insert_opt!(body, "location_visibility", location_visibility);

    let resp = submit(confirm, "/v1/events/update", &body)?;
    println!("{resp}");
    Ok(())
}

/// Source keys clone doesn't copy. Every other field is copied, so writable
/// fields the API adds later carry over without a change here.
const CLONE_SKIP_KEYS: &[&str] = &[
    // Get-only: identifiers, derived values, and manage-view extras.
    "platform",
    "id",
    "user_id",
    "calendar_id",
    "created_at",
    "url",
    "duration_interval",
    "location_type",
    "coordinate",
    "hosts",
    "guest_counts",
    "spots_remaining",
    "display_price",
    "access",
    "description",
    "require_approval",
    "tax_config",
    // Create accepts these, but copying them gives a wrong clone: past events
    // often have registration closed, and `meeting_url` would share the
    // source's meeting room.
    "registration_open",
    "meeting_url",
];

/// Create only accepts covers hosted here.
const LUMA_CDN: &str = "https://images.lumacdn.com/";

struct CloneOverrides {
    name: Option<String>,
    start_at: Option<String>,
    visibility: String,
    description_md: Option<String>,
}

fn clone_event(event_id: &str, overrides: CloneOverrides, confirm: bool) -> anyhow::Result<()> {
    let source = client::send(client::get("/v1/events/get")?.query(&[("event_id", event_id)]))?;
    let source: serde_json::Map<String, serde_json::Value> = serde_json::from_str(&source)?;

    // The view variant omits registration questions, capacity, emails, etc.,
    // so a clone built from it would be incomplete without any error.
    if source.get("access").and_then(|v| v.as_str()) != Some("manage") {
        bail!("can only clone events you manage");
    }

    // Ticket types aren't part of the get response. The spec shows no
    // pagination here; `paginate` stops after one page if there is none.
    let ticket_types = client::paginate(
        "/v1/events/ticket-types/list",
        &[
            ("event_id", event_id.to_owned()),
            ("include_hidden", "true".to_owned()),
        ],
    )?;

    let body = build_clone_body(source, ticket_types, overrides)?;
    let resp = submit(confirm, "/v1/events/create", &body)?;
    report_created(&resp)
}

/// Turn a get response into a create body: copy every field create accepts,
/// apply the CLI overrides, and fix the fields whose get shape differs. Warns
/// on stderr for set fields the clone loses.
fn build_clone_body(
    mut source: serde_json::Map<String, serde_json::Value>,
    ticket_types: Vec<serde_json::Value>,
    overrides: CloneOverrides,
) -> anyhow::Result<serde_json::Map<String, serde_json::Value>> {
    // How far the start moves; end and ticket sale windows move with it.
    let delta = match &overrides.start_at {
        Some(new_start) => {
            let orig_start = source
                .get("start_at")
                .and_then(|v| v.as_str())
                .context("source event missing 'start_at'")?;
            DateTime::parse_from_rfc3339(new_start)? - DateTime::parse_from_rfc3339(orig_start)?
        }
        None => TimeDelta::zero(),
    };

    for key in ["meeting_url", "tax_config"] {
        if !source.get(key).is_none_or(serde_json::Value::is_null) {
            eprintln!("warning: source '{key}' is not copied to the clone");
        }
    }
    // Older events can have covers from elsewhere (e.g. Unsplash), which create rejects.
    if source
        .get("cover_url")
        .and_then(|v| v.as_str())
        .is_some_and(|url| !url.starts_with(LUMA_CDN))
    {
        eprintln!("warning: source 'cover_url' is not on {LUMA_CDN}, not copied to the clone");
        source.remove("cover_url");
    }

    // `geo_address_json` can't round-trip: the get response returns an Address
    // object, but create expects a `{ type, ... }` variant. Translate it.
    match source.get("geo_address_json").and_then(address_to_input) {
        Some(geo) => source.insert("geo_address_json".into(), geo),
        None => source.remove("geo_address_json"),
    };

    source.retain(|k, _| !CLONE_SKIP_KEYS.contains(&k.as_str()));

    if let Some(end) = source.get("end_at").and_then(|v| v.as_str()) {
        let shifted = shift_by(end, delta)?;
        source.insert("end_at".into(), shifted.into());
    }

    insert_opt!(source, "start_at", overrides.start_at);
    insert_opt!(source, "name", overrides.name);
    insert_opt!(source, "description_md", overrides.description_md);
    source.insert("visibility".into(), overrides.visibility.into());

    let ticket_types = ticket_types
        .into_iter()
        .map(|t| clone_ticket_type(t, delta))
        .collect::<anyhow::Result<Vec<_>>>()?;
    // Create needs at least one; leaving it out gives the default free ticket.
    if !ticket_types.is_empty() {
        source.insert("ticket_types".into(), ticket_types.into());
    }

    // Create rejects null on many fields (e.g. a question's `source`);
    // leaving a key out gives the default.
    source.retain(|_, v| !v.is_null());
    source.values_mut().for_each(strip_nulls);

    Ok(source)
}

/// Remove null-valued object fields at every depth.
fn strip_nulls(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            map.retain(|_, v| !v.is_null());
            map.values_mut().for_each(strip_nulls);
        }
        serde_json::Value::Array(items) => items.iter_mut().for_each(strip_nulls),
        _ => {}
    }
}

/// Strip the source id and move the sale window along with the event.
fn clone_ticket_type(
    mut ticket: serde_json::Value,
    delta: TimeDelta,
) -> anyhow::Result<serde_json::Value> {
    let obj = ticket
        .as_object_mut()
        .context("ticket type is not an object")?;
    obj.remove("id");
    for key in ["valid_start_at", "valid_end_at"] {
        if let Some(ts) = obj.get(key).and_then(|v| v.as_str()) {
            let shifted = shift_by(ts, delta)?;
            obj.insert(key.into(), shifted.into());
        }
    }
    Ok(ticket)
}

/// Move an ISO 8601 datetime or date by `delta`. Datetimes come back as UTC
/// RFC3339 with millis. Dates stay dates and move by whole days (truncated).
fn shift_by(ts: &str, delta: TimeDelta) -> anyhow::Result<String> {
    if let Ok(date) = NaiveDate::parse_from_str(ts, "%Y-%m-%d") {
        return Ok((date + TimeDelta::days(delta.num_days())).to_string());
    }
    let shifted = DateTime::parse_from_rfc3339(ts)?.with_timezone(&Utc) + delta;
    Ok(shifted.to_rfc3339_opts(SecondsFormat::Millis, true))
}

/// Convert a get-response `Address` into the `geo_address_json` shape that
/// create/update expects. Prefers the Google place id when present, so the
/// location resolves to the same place; otherwise falls back to the raw address.
fn address_to_input(addr: &serde_json::Value) -> Option<serde_json::Value> {
    let mut out = serde_json::Map::new();
    if let Some(place_id) = addr["google_maps_place_id"].as_str() {
        out.insert("type".into(), "google".into());
        out.insert("place_id".into(), place_id.into());
        if let Some(desc) = addr["description"].as_str() {
            out.insert("description".into(), desc.into());
        }
    } else {
        let address = addr["address"].as_str()?;
        out.insert("type".into(), "manual".into());
        out.insert("address".into(), address.into());
    }
    Some(out.into())
}

fn lookup_event_id(event_url: &str) -> anyhow::Result<String> {
    let slug = event_url
        .trim_end_matches("/")
        .rsplit("/")
        .next()
        .and_then(|s| s.split(['?', '#']).next())
        .filter(|s| !s.is_empty())
        .context("could not extract slug from URL")?;

    let body = client::send(client::get("/v1/entities/lookup")?.query(&[("slug", slug)]))?;
    let body: serde_json::Value = serde_json::from_str(&body)?;

    let entity = &body["entity"];
    match entity["type"].as_str() {
        Some("event") => entity["event"]["id"]
            .as_str()
            .map(str::to_owned)
            .context("event not found for URL"),
        Some("calendar") => {
            bail!("'{slug}' is a calendar, not an event")
        }
        Some(other) => bail!("'{slug}' is {other}, not an event"),
        None => bail!("no entity found for '{slug}'"),
    }
}

#[cfg(test)]
mod tests {
    use super::{CloneOverrides, address_to_input, build_clone_body, clone_ticket_type, shift_by};
    use chrono::TimeDelta;
    use serde_json::json;

    #[test]
    fn shift_by_moves_datetime() {
        let shifted = shift_by("2026-08-16T16:00:00.000Z", TimeDelta::days(7)).unwrap();
        assert_eq!(shifted, "2026-08-23T16:00:00.000Z");
    }

    #[test]
    fn shift_by_normalizes_offset_to_utc() {
        let shifted = shift_by("2026-08-14T21:00:00+02:00", TimeDelta::zero()).unwrap();
        assert_eq!(shifted, "2026-08-14T19:00:00.000Z");
    }

    #[test]
    fn shift_by_truncates_date_to_whole_days() {
        // 6d 12h moves a date-only value by 6 days, and it stays a date.
        let delta = TimeDelta::days(6) + TimeDelta::hours(12);
        assert_eq!(shift_by("2026-09-01", delta).unwrap(), "2026-09-07");
        assert_eq!(shift_by("2026-09-01", -delta).unwrap(), "2026-08-26");
    }

    #[test]
    fn shift_by_rejects_bad_input() {
        assert!(shift_by("not-a-date", TimeDelta::zero()).is_err());
    }

    #[test]
    fn ticket_type_drops_id_and_shifts_window() {
        let ticket = json!({
            "id": "evtticktyp-1",
            "name": "Early bird",
            "type": "paid",
            "cents": 2500,
            "currency": "usd",
            "valid_start_at": "2026-08-01",
            "valid_end_at": "2026-08-10T12:00:00.000Z",
            "max_capacity": null,
        });
        assert_eq!(
            clone_ticket_type(ticket, TimeDelta::days(7)).unwrap(),
            json!({
                "name": "Early bird",
                "type": "paid",
                "cents": 2500,
                "currency": "usd",
                "valid_start_at": "2026-08-08",
                "valid_end_at": "2026-08-17T12:00:00.000Z",
                "max_capacity": null,
            })
        );
    }

    fn source() -> serde_json::Map<String, serde_json::Value> {
        json!({
            "platform": "luma",
            "id": "evt-abc",
            "access": "manage",
            "url": "abc",
            "hosts": [],
            "tax_config": null,
            "name": "Meetup",
            "start_at": "2026-08-14T16:00:00.000Z",
            "end_at": "2026-08-14T19:00:00.000Z",
            "timezone": "Europe/Berlin",
            "visibility": "public",
            "cover_url": "https://images.lumacdn.com/uploads/a.png",
            "registration_open": false,
            "meeting_url": "https://zoom.us/j/1",
            "max_capacity": null,
            "waitlist_status": "enabled",
            "registration_questions": [{ "id": "q1", "label": "Why?", "required": true, "question_type": "text", "source": null }],
            "geo_address_json": { "address": "123 Main St", "google_maps_place_id": null, "description": null },
        })
        .as_object()
        .unwrap()
        .clone()
    }

    fn no_overrides() -> CloneOverrides {
        CloneOverrides {
            name: None,
            start_at: None,
            visibility: "private".into(),
            description_md: None,
        }
    }

    #[test]
    fn clone_body_copies_writable_fields_only() {
        let body = build_clone_body(source(), vec![], no_overrides()).unwrap();
        assert_eq!(
            serde_json::Value::from(body),
            json!({
                "name": "Meetup",
                "start_at": "2026-08-14T16:00:00.000Z",
                "end_at": "2026-08-14T19:00:00.000Z",
                "timezone": "Europe/Berlin",
                "visibility": "private",
                "cover_url": "https://images.lumacdn.com/uploads/a.png",
                "waitlist_status": "enabled",
                "registration_questions": [{ "id": "q1", "label": "Why?", "required": true, "question_type": "text" }],
                "geo_address_json": { "type": "manual", "address": "123 Main St" },
            })
        );
    }

    #[test]
    fn clone_body_applies_overrides_and_moves_tickets() {
        let overrides = CloneOverrides {
            name: Some("Meetup II".into()),
            start_at: Some("2026-08-21T16:00:00.000Z".into()),
            visibility: "public".into(),
            description_md: Some("New".into()),
        };
        let tickets =
            vec![json!({ "id": "t1", "name": "GA", "type": "free", "valid_end_at": "2026-08-14" })];
        let body = build_clone_body(source(), tickets, overrides).unwrap();
        assert_eq!(body["name"], "Meetup II");
        assert_eq!(body["start_at"], "2026-08-21T16:00:00.000Z");
        assert_eq!(body["end_at"], "2026-08-21T19:00:00.000Z");
        assert_eq!(body["visibility"], "public");
        assert_eq!(body["description_md"], "New");
        assert_eq!(
            body["ticket_types"],
            json!([{ "name": "GA", "type": "free", "valid_end_at": "2026-08-21" }])
        );
    }

    #[test]
    fn clone_body_drops_foreign_cover() {
        let mut src = source();
        src.insert(
            "cover_url".into(),
            "https://images.unsplash.com/photo-1".into(),
        );
        let body = build_clone_body(src, vec![], no_overrides()).unwrap();
        assert!(!body.contains_key("cover_url"));
    }

    // Address shape mirrors components.schemas.Address in the openapi spec.
    #[test]
    fn prefers_google_place_id() {
        let addr = json!({
            "address": "123 Main St",
            "google_maps_place_id": "ChIJxyz",
            "apple_maps_place_id": null,
            "description": "HQ",
            "city": "SF",
            "region": "CA",
            "country": "US",
            "city_state": "SF, CA",
            "full_address": "123 Main St, SF, CA",
        });
        assert_eq!(
            address_to_input(&addr).unwrap(),
            json!({ "type": "google", "place_id": "ChIJxyz", "description": "HQ" })
        );
    }

    #[test]
    fn google_without_description() {
        let addr = json!({ "address": "123 Main St", "google_maps_place_id": "ChIJxyz", "description": null });
        assert_eq!(
            address_to_input(&addr).unwrap(),
            json!({ "type": "google", "place_id": "ChIJxyz" })
        );
    }

    #[test]
    fn falls_back_to_manual_address() {
        let addr =
            json!({ "address": "123 Main St", "google_maps_place_id": null, "description": null });
        assert_eq!(
            address_to_input(&addr).unwrap(),
            json!({ "type": "manual", "address": "123 Main St" })
        );
    }

    #[test]
    fn null_geo_is_none() {
        assert_eq!(address_to_input(&json!(null)), None);
    }

    #[test]
    fn empty_object_is_none() {
        assert_eq!(address_to_input(&json!({})), None);
    }
}
