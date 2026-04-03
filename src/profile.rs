use anyhow::{bail, Context, Result};

use crate::paths;

/// Validate that a profile name contains only safe characters.
///
/// Allowed: `[a-zA-Z0-9_-]+` (non-empty).
/// Rejected: empty, ".", "..", and anything with path separators or special chars.
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("Profile name cannot be empty.");
    }
    if name == "." || name == ".." {
        bail!(
            "Profile name '{}' is invalid. Use only letters, numbers, hyphens, and underscores.",
            name
        );
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        bail!(
            "Profile name '{}' is invalid. Use only letters, numbers, hyphens, and underscores.",
            name
        );
    }
    Ok(())
}

/// Create a profile directory with 0700 permissions.
///
/// The profile directory is opaque -- clmux never inspects its contents (DATA-04).
pub fn create_profile_dir(name: &str) -> Result<()> {
    use std::os::unix::fs::DirBuilderExt;

    validate_profile_name(name)?;

    let dir = paths::profile_dir(name)?;
    if dir.exists() {
        bail!("Profile '{}' already exists.", name);
    }

    // Ensure parent profiles/ directory exists
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create profiles directory at {}", parent.display()))?;
    }

    // Create profile directory with 0700 permissions
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .with_context(|| format!("Failed to create profile directory at {}", dir.display()))?;

    Ok(())
}
