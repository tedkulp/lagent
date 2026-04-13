mod agent;
mod cli;
mod commands;
mod launchctl;
mod scope;
mod state;
mod validate;

use anyhow::Result;
use clap::CommandFactory;
use clap::Parser;
use clap_complete::generate;
use cli::{Cli, Commands};
use owo_colors::OwoColorize;

fn main() {
    if let Err(e) = run() {
        eprintln!("{}: {}", "error".red(), e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list(cli.user),
        Commands::Status { agent } => commands::status(&agent, cli.user),
        Commands::Enable { agent } => commands::enable(&agent, cli.user),
        Commands::Disable { agent } => commands::disable(&agent, cli.user),
        Commands::Start { agent } => commands::start(&agent, cli.user),
        Commands::Stop { agent } => commands::stop(&agent, cli.user),
        Commands::Restart { agent } => commands::restart(&agent, cli.user),
        Commands::Reload { agent } => commands::reload(&agent, cli.user),
        Commands::Validate { agent } => commands::validate(&agent, cli.user),
        Commands::Completions { shell } => {
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "lagent", &mut std::io::stdout());
            Ok(())
        }
    }
}
