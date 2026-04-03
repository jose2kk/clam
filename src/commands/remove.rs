use anyhow::Result;

pub fn execute(name: &str, force: bool) -> Result<()> {
    let _ = (name, force);
    anyhow::bail!("Not yet implemented")
}
