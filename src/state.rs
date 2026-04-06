use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub active: Option<String>,
}

/// Load state from state.toml. Returns default if file doesn't exist.
pub fn load() -> Result<State> {
    let path = paths::state_path()?;
    if !path.exists() {
        return Ok(State::default());
    }
    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let state: State =
        toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(state)
}

/// Save state to state.toml atomically.
pub fn save(state: &State) -> Result<()> {
    let content = toml::to_string(state).context("Failed to serialize state")?;
    let path = paths::state_path()?;
    paths::atomic_write(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ENV_LOCK;

    #[test]
    fn test_default_state_round_trip() {
        let state = State::default();
        let serialized = toml::to_string(&state).unwrap();
        let deserialized: State = toml::from_str(&serialized).unwrap();
        assert!(deserialized.active.is_none());
    }

    #[test]
    fn test_state_with_active_round_trip() {
        let state = State {
            active: Some("work".into()),
        };
        let serialized = toml::to_string(&state).unwrap();
        let deserialized: State = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.active, Some("work".into()));
    }

    #[test]
    fn test_state_none_active_round_trip() {
        let state = State { active: None };
        let serialized = toml::to_string(&state).unwrap();
        let deserialized: State = toml::from_str(&serialized).unwrap();
        assert!(deserialized.active.is_none());
    }

    #[test]
    fn test_load_missing_file_returns_default() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLMUX_HOME", tmp.path());
        let state = load()?;
        assert!(state.active.is_none());
        std::env::remove_var("CLMUX_HOME");
        Ok(())
    }

    #[test]
    fn test_save_and_load_round_trip() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLMUX_HOME", tmp.path());
        let state = State {
            active: Some("roundtrip".into()),
        };
        save(&state)?;
        let loaded = load()?;
        assert_eq!(loaded.active, Some("roundtrip".into()));
        std::env::remove_var("CLMUX_HOME");
        Ok(())
    }

    #[test]
    fn test_save_none_and_load_round_trip() -> anyhow::Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLMUX_HOME", tmp.path());
        let state = State { active: None };
        save(&state)?;
        let loaded = load()?;
        assert!(loaded.active.is_none());
        std::env::remove_var("CLMUX_HOME");
        Ok(())
    }
}
