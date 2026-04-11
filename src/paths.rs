use std::path::PathBuf;

use anyhow::{Context, Result};

/// Returns the base directory for clam data.
///
/// Uses `CLAM_HOME` env var if set, otherwise `~/.clam/`.
pub fn base_dir() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("CLAM_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".clam"))
}

/// Returns the path to config.toml (profile registry).
pub fn config_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("config.toml"))
}

/// Returns the path to state.toml (active profile pointer).
pub fn state_path() -> Result<PathBuf> {
    Ok(base_dir()?.join("state.toml"))
}

/// Returns the path to a profile's directory.
pub fn profile_dir(name: &str) -> Result<PathBuf> {
    Ok(base_dir()?.join("profiles").join(name))
}

/// Writes content to a file atomically using tempfile + rename.
///
/// Creates parent directories if needed.
pub fn atomic_write(path: &std::path::Path, content: &str) -> Result<()> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory {}", parent.display()))?;
    }

    let mut tmp =
        NamedTempFile::new_in(path.parent().context("File path has no parent directory")?)
            .context("Failed to create temporary file")?;

    tmp.write_all(content.as_bytes())
        .context("Failed to write temporary file")?;

    tmp.persist(path)
        .map_err(|e| e.error)
        .context("Failed to save file")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ENV_LOCK;

    // --- base_dir() tests (TEST-04) ---

    #[test]
    fn test_base_dir_respects_clam_home() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());

        let result = base_dir()?;
        assert_eq!(result, tmp.path());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    #[test]
    fn test_base_dir_defaults_to_home_clam() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        std::env::remove_var("CLAM_HOME");

        let result = base_dir()?;
        assert!(
            result.ends_with(".clam"),
            "Expected path ending with .clam, got: {}",
            result.display()
        );

        Ok(())
    }

    // --- profile_dir() tests (TEST-04) ---

    #[test]
    fn test_profile_dir_appends_name() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());

        let result = profile_dir("work")?;
        assert_eq!(result, tmp.path().join("profiles").join("work"));

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    // --- atomic_write() tests (TEST-05) ---

    #[test]
    fn test_atomic_write_creates_file() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("test.txt");

        atomic_write(&path, "hello")?;

        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "hello");
        Ok(())
    }

    #[test]
    fn test_atomic_write_creates_parent_dirs() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("nested").join("dir").join("file.txt");

        atomic_write(&path, "nested content")?;

        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "nested content");
        Ok(())
    }

    #[test]
    fn test_atomic_write_overwrites_existing() -> anyhow::Result<()> {
        let tmp = tempfile::tempdir()?;
        let path = tmp.path().join("test.txt");

        atomic_write(&path, "first")?;
        atomic_write(&path, "second")?;

        let content = std::fs::read_to_string(&path)?;
        assert_eq!(content, "second");
        Ok(())
    }
}
