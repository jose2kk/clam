use std::io::{self, IsTerminal, Write};

use anyhow::{Context, Result};

use crate::{config, output, paths, profile, state};

pub fn execute(name: &str, force: bool) -> Result<()> {
    // 1. Validate name (PROF-06)
    profile::validate_profile_name(name)?;

    // 2. Check profile exists in registry
    let mut cfg = config::load()?;
    if !cfg.profiles.iter().any(|p| p.name == name) {
        anyhow::bail!("Profile '{name}' not found. Run `clam list` to see available profiles.");
    }

    // 3. Refuse to remove active profile (D-07)
    let st = state::load()?;
    if st.active.as_deref() == Some(name) {
        anyhow::bail!(
            "Cannot remove active profile '{name}'. Switch to another profile first with `clam use <name>`."
        );
    }

    // 4. Confirmation prompt (D-06) unless --force
    if !force {
        if !io::stdin().is_terminal() {
            anyhow::bail!(
                "Cannot prompt for confirmation. Use `clam remove {name} --force` to skip."
            );
        }
        print!("Remove profile {name}? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // 5. Remove profile directory (if it exists)
    let dir = paths::profile_dir(name)?;
    if dir.is_dir() {
        std::fs::remove_dir_all(&dir)
            .with_context(|| format!("Failed to remove profile directory: {}", dir.display()))?;
    }

    // 6. Remove from config registry and save atomically (DATA-02)
    cfg.profiles.retain(|p| p.name != name);
    config::save(&cfg)?;

    output::success(&format!("Removed profile '{name}'."));
    Ok(())
}
