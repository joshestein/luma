use anyhow::Context;
use std::env;

const BASE: &'static str = "https://public-api.luma.com/v1/";

pub fn get(path: &str) -> anyhow::Result<reqwest::blocking::RequestBuilder> {
    Ok(build()?.get(format!("{BASE}{path}")))
}

fn build() -> anyhow::Result<reqwest::blocking::Client> {
    let key = api_key()?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("x-luma-api-key", key.parse()?);
    reqwest::blocking::Client::builder()
        .default_headers(headers)
        .build()
        .context("building HTTP client")
}

fn api_key() -> anyhow::Result<String> {
    env::var("LUMA_API_KEY").context("LUMA_API_KEY not set")
}
