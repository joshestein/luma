use anyhow::{Context, bail};
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

const BASE: &'static str = "https://public-api.luma.com";

static CLIENT: OnceLock<reqwest::blocking::Client> = OnceLock::new();

pub fn get(path: &str) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
    Ok(client()?.get(format!("{BASE}{path}")))
}

pub fn post(path: &str) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
    Ok(client()?.post(format!("{BASE}{path}")))
}

/// Return a shared HTTP client, building it once on first use
fn client() -> anyhow::Result<&'static reqwest::blocking::Client> {
    if let Some(c) = CLIENT.get() {
        return Ok(c);
    }
    let c = build()?;
    Ok(CLIENT.get_or_init(|| c))
}

/// Send a request and return the response body, surfacing the API's error
/// body on any non-2xx status
pub fn send(req: reqwest::blocking::RequestBuilder) -> anyhow::Result<String> {
    let resp = req.send()?;
    let status = resp.status();
    let body = resp.text()?;
    if !status.is_success() {
        bail!("request failed ({status}): {body}");
    }
    Ok(body)
}

pub fn paginate(path: &str, params: &[(&str, String)]) -> anyhow::Result<Vec<serde_json::Value>> {
    let mut entries = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let mut q = params.to_vec();
        if let Some(c) = &cursor {
            q.push(("pagination_cursor", c.clone()));
        }
        let body = send(get(path)?.query(&q))?;
        let page: serde_json::Value = serde_json::from_str(&body)?;
        if let Some(arr) = page["entries"].as_array() {
            entries.extend(arr.iter().cloned());
        }
        match (page["has_more"].as_bool(), page["next_cursor"].as_str()) {
            (Some(true), Some(c)) => cursor = Some(c.to_owned()),
            _ => break,
        }
    }

    Ok(entries)
}

fn build() -> anyhow::Result<reqwest::blocking::Client> {
    let key = api_key()?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-luma-api-key", key.parse()?);
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()
        .context("building HTTP client")
}

fn api_key() -> anyhow::Result<String> {
    env::var("LUMA_API_KEY").context("LUMA_API_KEY not set")
}
