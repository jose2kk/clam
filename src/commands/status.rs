use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;

use crate::{paths, state};

pub fn execute(_json: bool) -> Result<()> {
    let st = state::load()?;

    let active = match st.active {
        Some(name) => name,
        None => {
            eprintln!("No active profile. Run `clmux add <name>` to create one.");
            std::process::exit(1);
        }
    };

    let dir = paths::profile_dir(&active)?;
    let dir_exists = dir.is_dir();

    let health: String = if dir_exists {
        format!("{}", "ok".if_supports_color(Stdout, |t| t.green().to_string()))
    } else {
        format!("{}", "missing".if_supports_color(Stdout, |t| t.red().to_string()))
    };

    let profile_display = format!("{}", active.if_supports_color(Stdout, |t| t.bold().to_string()));

    println!("Profile: {}", profile_display);
    println!("Path:    {}", dir.display());
    println!("Status:  {}", health);
    println!("Config:  CLAUDE_CONFIG_DIR={}", dir.display());

    if dir_exists {
        let item_count = std::fs::read_dir(&dir)
            .map(|entries| entries.count())
            .unwrap_or(0);
        println!("Items:   {} file(s)", item_count);
    }

    Ok(())
}
