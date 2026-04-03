use anyhow::Result;
use owo_colors::OwoColorize;
use owo_colors::Stream::Stdout;

use crate::{config, state};

pub fn execute() -> Result<()> {
    let cfg = config::load()?;
    let st = state::load()?;
    let active = st.active.as_deref();

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
