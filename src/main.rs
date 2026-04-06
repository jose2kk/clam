mod cli;
mod commands;
mod config;
mod output;
mod paths;
mod profile;
mod state;

#[cfg(test)]
mod test_utils {
    /// Shared mutex for tests that modify `CLMUX_HOME` env var.
    /// All modules must use this single lock to prevent cross-module races.
    pub static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
}

use clap::Parser;

fn main() {
    if let Err(e) = run() {
        output::error(&format!("{e:#}"));
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Add { name, inherit } => commands::add::execute(&name, inherit),
        cli::Commands::List { json } => commands::list::execute(json),
        cli::Commands::Use { name } => commands::use_cmd::execute(&name),
        cli::Commands::Current => commands::current::execute(),
        cli::Commands::Remove { name, force } => commands::remove::execute(&name, force),
        cli::Commands::Status { json } => commands::status::execute(json),
        cli::Commands::Run { profile, args } => commands::run::execute(profile.as_deref(), &args),
        cli::Commands::Env { json } => commands::env::execute(json),
        cli::Commands::Completions { shell } => {
            commands::completions::execute(&shell);
            Ok(())
        }
    }
}
