use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clmux", about = "Manage Claude Code profiles")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new profile
    Add { name: String },
    /// List all profiles
    List,
    /// Switch active profile
    Use { name: String },
    /// Print active profile name
    Current,
    /// Remove a profile
    Remove {
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Show active profile status
    Status,
    /// Launch a command with profile environment
    Run {
        /// Use a specific profile (without switching active)
        #[arg(long)]
        profile: Option<String>,
        /// Command and arguments to run (default: claude)
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Print shell environment exports for eval
    Env,
}
