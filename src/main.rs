mod cli;
mod commands;
mod config;
mod output;
mod paths;
mod profile;
mod state;

use clap::Parser;

fn main() {
    if let Err(e) = run() {
        output::error(&format!("{:#}", e));
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Add { name } => commands::add::execute(&name),
        cli::Commands::List => commands::list::execute(),
        cli::Commands::Use { name } => commands::use_cmd::execute(&name),
        cli::Commands::Current => commands::current::execute(),
        cli::Commands::Remove { name, force } => commands::remove::execute(&name, force),
        cli::Commands::Status => commands::status::execute(),
    }
}
