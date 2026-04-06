use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profiles: Vec<ProfileEntry>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ProfileEntry {
    pub name: String,
}

/// Load config from config.toml. Returns default if file doesn't exist.
pub fn load() -> Result<Config> {
    let path = paths::config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let config: Config =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(config)
}

/// Save config to config.toml atomically.
pub fn save(config: &Config) -> Result<()> {
    let content = toml::to_string(config).context("Failed to serialize config")?;
    let path = paths::config_path()?;
    paths::atomic_write(&path, &content)
}
