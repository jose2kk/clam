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
            "Profile name '{name}' is invalid. Use only letters, numbers, hyphens, and underscores."
        );
    }
    let valid = name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
    if !valid {
        bail!(
            "Profile name '{name}' is invalid. Use only letters, numbers, hyphens, and underscores."
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
        bail!("Profile '{name}' already exists.");
    }

    // Ensure parent profiles/ directory exists
    if let Some(parent) = dir.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create profiles directory at {}",
                parent.display()
            )
        })?;
    }

    // Create profile directory with 0700 permissions
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&dir)
        .with_context(|| format!("Failed to create profile directory at {}", dir.display()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Valid names (assert is_ok) ---

    #[test]
    fn test_accepts_simple_name() {
        assert!(validate_profile_name("work").is_ok());
    }

    #[test]
    fn test_accepts_hyphens_and_underscores() {
        assert!(validate_profile_name("my-work_profile").is_ok());
    }

    #[test]
    fn test_accepts_mixed_case_and_numbers() {
        assert!(validate_profile_name("Work123").is_ok());
    }

    #[test]
    fn test_accepts_single_char() {
        assert!(validate_profile_name("a").is_ok());
    }

    // --- Invalid names (assert is_err) ---

    #[test]
    fn test_rejects_empty_string() {
        let err = validate_profile_name("").unwrap_err();
        assert!(
            err.to_string().contains("empty"),
            "Error should mention 'empty', got: {}",
            err
        );
    }

    #[test]
    fn test_rejects_dot() {
        assert!(validate_profile_name(".").is_err());
    }

    #[test]
    fn test_rejects_dotdot() {
        assert!(validate_profile_name("..").is_err());
    }

    #[test]
    fn test_rejects_path_traversal() {
        assert!(validate_profile_name("../evil").is_err());
    }

    #[test]
    fn test_rejects_spaces() {
        assert!(validate_profile_name("my profile").is_err());
    }

    #[test]
    fn test_rejects_slashes() {
        assert!(validate_profile_name("a/b").is_err());
    }

    #[test]
    fn test_rejects_special_chars() {
        assert!(validate_profile_name("name@host").is_err());
    }

    #[test]
    fn test_rejects_backslash() {
        assert!(validate_profile_name("a\\b").is_err());
    }
}
