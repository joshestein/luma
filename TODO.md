# TODO

- Enforce string enums as typed `clap::ValueEnum`s instead of `Option<String>`.
  - `location_visibility`: public, guests-only
  - `visibility`: check openapi.json for allowed values
  - Enum must also derive `serde::Deserialize` (kebab-case) for the clone round-trip, and impl `From<Enum> for serde_json::Value` so `insert_opt!` stays unchanged.
