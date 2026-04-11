use std::os::unix::process::CommandExt;
use std::process::Command;

use anyhow::{bail, Result};

use crate::{config, paths, profile, state};

/// Resolve which profile to use, returning (name, `directory_path`).
///
/// If `profile_override` is Some, validates and uses that profile without modifying state.
/// If None, uses the active profile from state.toml.
pub(crate) fn resolve_profile(
    profile_override: Option<&str>,
) -> Result<(String, std::path::PathBuf)> {
    let name = if let Some(name) = profile_override {
        profile::validate_profile_name(name)?;
        let cfg = config::load()?;
        if !cfg.profiles.iter().any(|p| p.name == name) {
            bail!("Profile '{name}' not found. Run `clam list` to see available profiles.");
        }
        name.to_string()
    } else {
        let st = state::load()?;
        match st.active {
            Some(name) => name,
            None => bail!("No active profile. Run `clam add <name>` to create one."),
        }
    };

    let dir = paths::profile_dir(&name)?;
    Ok((name, dir))
}

/// Execute a command with profile-scoped environment.
///
/// Sanitizes CLAUDE_* and ANTHROPIC_* env vars, sets `CLAUDE_CONFIG_DIR` to the
/// profile directory, and exec()s into the target binary (replacing the clam process).
pub fn execute(profile_override: Option<&str>, args: &[String]) -> Result<()> {
    let (name, dir) = resolve_profile(profile_override)?;

    if !dir.is_dir() {
        bail!(
            "Profile directory for '{}' is missing at {}. Run `clam add {}` to recreate it.",
            name,
            dir.display(),
            name
        );
    }

    let (binary, cmd_args): (&str, &[String]) = if args.is_empty() {
        ("claude", &[])
    } else {
        (args[0].as_str(), &args[1..])
    };

    let mut cmd = Command::new(binary);
    cmd.args(cmd_args);

    // Strip all CLAUDE_* and ANTHROPIC_* env vars for clean isolation
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            cmd.env_remove(&key);
        }
    }

    // Set profile-scoped environment
    cmd.env("CLAUDE_CONFIG_DIR", &dir);
    cmd.env("CLAM_PROFILE", &name);

    // exec() replaces the current process -- only returns on error
    let err = cmd.exec();
    Err(anyhow::anyhow!(
        "Failed to execute '{binary}': {err}. Is it installed and in your PATH?"
    ))
}
