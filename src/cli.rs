use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "lagent",
    about = "Manage macOS LaunchAgents",
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Target ~/Library/LaunchAgents instead of /Library/LaunchAgents
    #[arg(long, global = true)]
    pub user: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all LaunchAgents in the target directory
    List,
    /// Show status of a specific agent
    Status {
        /// Agent label or plist filename (with or without .plist)
        agent: String,
    },
    /// Load and permanently enable an agent to start at login
    Enable {
        /// Agent label or plist filename
        agent: String,
    },
    /// Permanently disable and unload an agent
    Disable {
        /// Agent label or plist filename
        agent: String,
    },
    /// Start a loaded agent immediately
    Start {
        /// Agent label or plist filename
        agent: String,
    },
    /// Stop a running agent immediately
    Stop {
        /// Agent label or plist filename
        agent: String,
    },
    /// Stop then start an agent (does not reload plist)
    Restart {
        /// Agent label or plist filename
        agent: String,
    },
    /// Reload plist definition from disk; restart if was running and plist changed
    Reload {
        /// Agent label or plist filename
        agent: String,
    },
    /// Validate a plist for syntax and launchd schema correctness
    Validate {
        /// Agent label or plist filename
        agent: String,
    },
    /// Print shell completion script to stdout
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
}
