# luma

A CLI wrapper around the [Luma public API](https://public-api.luma.com/openapi.json), written in Rust.

## Commands

```
luma auth check
luma events get    --event (EVT | URL)                         # evt- prefix = ID; else URL/slug → entities/lookup → get
luma events list   [--before ISO] [--after ISO]
luma events create --name N --start-at ISO --timezone TZ       # + optional --description-md --end-at --max-capacity --visibility
                   [--description-md M] [--end-at ISO] [--max-capacity N] [--visibility V]
luma events update --event-id EVT                              # any subset of the create fields
                   [--name N] [--start-at ISO] [--timezone TZ] [--description-md M] [--end-at ISO] [--max-capacity N] [--visibility V]
luma events clone  --event (EVT | URL) [--name N] [--start-at ISO] [--visibility V]   # fetch source, override, re-create
luma guests list   --event (EVT | URL)
luma guests get    --event (EVT | URL) --id (GUEST|EMAIL)
luma contacts list [--membership-status S] [--tags T]... [--query Q] [--cursor C]   # --tags repeatable, matches any
```

## Setup

Set `LUMA_API_KEY` in the environment or in a `.env` file.

Requires Rust 1.85+ (edition 2024). Ubuntu's apt `cargo` is too old.

## Deploy to VPS

Make sure rust installed:

```sh
# one-time
curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
```

Build:

```sh
cd ~/luma && git pull
cargo build --release
install -m755 target/release/luma ~/.local/bin/luma
```

