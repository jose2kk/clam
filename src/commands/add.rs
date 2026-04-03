use anyhow::Result;

use crate::{config, output, profile, state};

pub fn execute(name: &str) -> Result<()> {
    // 1. Validate profile name (PROF-06)
    profile::validate_profile_name(name)?;

    // 2. Load current config
    let mut cfg = config::load()?;

    // 3. Check if profile already exists in registry
    if cfg.profiles.iter().any(|p| p.name == name) {
        anyhow::bail!("Profile '{}' already exists.", name);
    }

    // 4. Create profile directory with 0700 permissions (DATA-03, DATA-04)
    profile::create_profile_dir(name)?;

    // 5. Detect if this is the first profile before adding
    let is_first = cfg.profiles.is_empty();

    // 6. Add to config registry and save atomically (DATA-02)
    cfg.profiles.push(config::ProfileEntry {
        name: name.to_string(),
    });
    config::save(&cfg)?;

    // 7. Auto-activate if this is the first profile (D-04)
    if is_first {
        let mut st = state::load()?;
        st.active = Some(name.to_string());
        state::save(&st)?;
        output::success(&format!("Created and activated profile '{}'.", name));
    } else {
        output::success(&format!("Created profile '{}'.", name));
    }

    Ok(())
}
