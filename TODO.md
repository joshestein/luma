# TODO

- Enforce string enums as typed `clap::ValueEnum`s instead of `Option<String>`.
  - `location_visibility`: public, guests-only
  - `visibility`: check openapi.json for allowed values
  - Impl `From<Enum> for serde_json::Value` so `insert_opt!` stays unchanged.
