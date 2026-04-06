use anyhow::{Context, Result};

use crate::{config, output, profile, state};

pub fn execute(name: &str, inherit: bool) -> Result<()> {
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

    // 5. Inherit global Claude Code config via symlinks
    if inherit {
        inherit_global_config(name)?;
    }

    // 6. Detect if this is the first profile before adding
    let is_first = cfg.profiles.is_empty();

    // 7. Add to config registry and save atomically (DATA-02)
    cfg.profiles.push(config::ProfileEntry {
        name: name.to_string(),
    });
    config::save(&cfg)?;

    // 8. Auto-activate if this is the first profile (D-04)
    if is_first {
        let mut st = state::load()?;
        st.active = Some(name.to_string());
        state::save(&st)?;
        output::success(&format!("Created and activated profile '{}'.", name));
    } else {
        output::success(&format!("Created profile '{}'.", name));
    }

    if inherit {
        output::success("Inherited global Claude Code config (~/.claude/).");
    }

    Ok(())
}

/// Symlink shared config files from ~/.claude/ into the profile directory.
///
/// Skips session-specific items that should stay isolated per profile.
fn inherit_global_config(name: &str) -> Result<()> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let claude_dir = home.join(".claude");

    if !claude_dir.exists() {
        return Ok(()); // No global config to inherit
    }

    let profile_dir = crate::paths::profile_dir(name)?;

    // Items that are session-specific and should NOT be shared
    let skip = [
        "statsig",
        "telemetry",
        "worktrees",
        "tasks",
        "sessions",
        "session-env",
        "shell-snapshots",
        "cache",
        "paste-cache",
        "stats-cache.json",
        "mcp-needs-auth-cache.json",
        "history.jsonl",
        "file-history",
        "debug",
        "plans",
        "backups",
    ];

    let entries = std::fs::read_dir(&claude_dir)
        .with_context(|| format!("Failed to read {}", claude_dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let name_str = file_name.to_string_lossy();

        if skip.iter().any(|s| *s == name_str.as_ref()) {
            continue;
        }

        let target = profile_dir.join(&file_name);
        if !target.exists() {
            std::os::unix::fs::symlink(entry.path(), &target).with_context(|| {
                format!("Failed to symlink {} into profile", name_str)
            })?;
        }
    }

    Ok(())
}
