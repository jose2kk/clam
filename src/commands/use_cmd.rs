use anyhow::Result;

use crate::{config, output, profile, state};

pub fn execute(name: &str) -> Result<()> {
    // 1. Validate name (PROF-06)
    profile::validate_profile_name(name)?;

    // 2. Check profile exists in registry
    let cfg = config::load()?;
    if !cfg.profiles.iter().any(|p| p.name == name) {
        anyhow::bail!("Profile '{name}' not found. Run `clam list` to see available profiles.");
    }

    // 3. Update state atomically (DATA-01)
    let mut st = state::load()?;
    st.active = Some(name.to_string());
    state::save(&st)?;

    output::success(&format!("Switched to profile '{name}'."));
    Ok(())
}
