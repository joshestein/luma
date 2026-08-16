use clap::{Parser, Subcommand};

mod client;
mod events;
mod guests;

#[derive(Parser)]
#[command(name = "luma")]
#[command(version = "1.0")]
#[command(after_long_help = ESCAPE_HATCH)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Shown on `luma --help`. Documents how to reach API routes this CLI does not
/// wrap, so the information travels with the binary even without the repo.
const ESCAPE_HATCH: &str = "\
UNSUPPORTED ROUTES:
  This CLI wraps a subset of the Luma API. For routes it does not implement,
  call the API directly using the same credentials:

    OpenAPI spec:  https://public-api.luma.com/openapi.json
    Base URL:      https://public-api.luma.com
    Auth header:   x-luma-api-key: <LUMA_API_KEY>
    Rate limits:   200 req/min (calendar key), 500 req/min (org key)

  Example:
    curl -H \"x-luma-api-key: $LUMA_API_KEY\" \\
      https://public-api.luma.com/v1/calendar/list-events";

#[derive(Subcommand)]
enum Commands {
    Auth {
        #[command(subcommand)]
        command: AuthCmd,
    },
    Events {
        #[command(subcommand)]
        command: events::Cmd,
    },
    Guests {
        #[command(subcommand)]
        command: guests::Cmd,
    },
}

#[derive(Subcommand)]
enum AuthCmd {
    /// Verify credentials work
    Check,
}

fn check_auth() -> anyhow::Result<()> {
    client::send(client::get("/v1/users/get-self")?)?;

    println!("Authenticated!");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCmd::Check => {
                check_auth()?;
            }
        },
        Commands::Events { command } => events::run(command)?,
        Commands::Guests { command } => guests::run(command)?,
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
