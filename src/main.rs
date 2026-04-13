mod cli;
mod scope;

use clap::Parser;
use cli::Cli;

fn main() {
    let cli = Cli::parse();
    let _ = cli;
    println!("TODO: dispatch");
}
