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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mutex to serialize tests that modify `CLMUX_HOME` env var.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn test_empty_config_round_trip() {
        let config = Config::default();
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert!(deserialized.profiles.is_empty());
    }

    #[test]
    fn test_config_with_profiles_round_trip() {
        let config = Config {
            profiles: vec![
                ProfileEntry {
                    name: "work".into(),
                },
                ProfileEntry {
                    name: "personal".into(),
                },
            ],
        };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.profiles.len(), 2);
        assert_eq!(deserialized.profiles[0].name, "work");
        assert_eq!(deserialized.profiles[1].name, "personal");
    }

    #[test]
    fn test_config_with_single_profile_round_trip() {
        let config = Config {
            profiles: vec![ProfileEntry {
                name: "test-profile".into(),
            }],
        };
        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.profiles.len(), 1);
        assert_eq!(deserialized.profiles[0].name, "test-profile");
    }

    #[test]
    fn test_load_missing_file_returns_default() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLMUX_HOME", tmp.path());
        let config = load()?;
        assert!(config.profiles.is_empty());
        std::env::remove_var("CLMUX_HOME");
        Ok(())
    }

    #[test]
    fn test_save_and_load_round_trip() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLMUX_HOME", tmp.path());
        let config = Config {
            profiles: vec![ProfileEntry {
                name: "roundtrip".into(),
            }],
        };
        save(&config)?;
        let loaded = load()?;
        assert_eq!(loaded.profiles.len(), 1);
        assert_eq!(loaded.profiles[0].name, "roundtrip");
        std::env::remove_var("CLMUX_HOME");
        Ok(())
    }
}
