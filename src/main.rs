use clap::{Parser, Subcommand};

mod client;
mod events;
mod guests;

#[derive(Parser)]
#[command(name = "luma")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

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
    let resp = client::get("/v1/users/get-self")?.send()?;
    resp.error_for_status()?;

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
