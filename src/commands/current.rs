use anyhow::Result;

use crate::state;

pub fn execute() -> Result<()> {
    let st = state::load()?;
    match st.active {
        Some(name) => {
            println!("{name}");
            Ok(())
        }
        None => {
            // Per D-03: no active profile = empty stdout + exit 1
            std::process::exit(1);
        }
    }
}
