use clap::{Parser, Subcommand};

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
}

#[derive(Subcommand)]
enum AuthCmd {
    Check,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Auth { command } => match command {
            AuthCmd::Check => println!("Checking auth"),
        },
    }
}
