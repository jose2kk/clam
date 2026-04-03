use anyhow::Result;

pub fn execute() -> Result<()> {
    let (name, dir) = super::run::resolve_profile(None)?;

    // Unset any existing CLAUDE_* and ANTHROPIC_* vars for clean isolation
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            println!("unset {};", key);
        }
    }

    // Export profile-scoped environment
    println!("export CLAUDE_CONFIG_DIR=\"{}\";", dir.display());
    println!("export CLMUX_PROFILE=\"{}\";", name);

    Ok(())
}
