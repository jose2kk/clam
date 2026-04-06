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
