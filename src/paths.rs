use std::path::PathBuf;

use anyhow::{Context, Result};

/// Returns the base directory for clmux data.
///
/// Uses `CLMUX_HOME` env var if set, otherwise `~/.clmux/`.
pub fn base_dir() -> Result<PathBuf> {
    if let Ok(home) = std::env::var("CLMUX_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home = dirs::home_dir().context("Could not determine home directory")?;
    Ok(home.join(".clmux"))
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

    let mut tmp = NamedTempFile::new_in(
        path.parent()
            .context("File path has no parent directory")?,
    )
    .context("Failed to create temporary file")?;

    tmp.write_all(content.as_bytes())
        .context("Failed to write temporary file")?;

    tmp.persist(path)
        .map_err(|e| e.error)
        .context("Failed to save file")?;

    Ok(())
}
