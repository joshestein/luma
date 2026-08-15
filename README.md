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
```
