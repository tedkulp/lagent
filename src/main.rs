mod agent;
mod cli;
mod scope;
mod state;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    let _ = cli;
    println!("TODO: dispatch");
}
