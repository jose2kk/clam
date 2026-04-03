use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;
use serde_json::json;

use crate::{config, paths, state};

pub fn execute(json: bool) -> Result<()> {
    let cfg = config::load()?;
    let st = state::load()?;
    let active = st.active.as_deref();

    if json {
        let entries: Vec<serde_json::Value> = cfg
            .profiles
            .iter()
            .map(|p| {
                let path = paths::profile_dir(&p.name)
                    .map(|d| d.display().to_string())
                    .unwrap_or_default();
                json!({
                    "name": p.name,
                    "active": Some(p.name.as_str()) == active,
                    "path": path,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&entries)?);
        return Ok(());
    }

    for profile in &cfg.profiles {
        if Some(profile.name.as_str()) == active {
            let styled = profile
                .name
                .if_supports_color(Stdout, |t| t.green().bold().to_string());
            println!("* {}", styled);
        } else {
            println!("  {}", profile.name);
        }
    }

    Ok(())
}
