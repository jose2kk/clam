//! Per-profile global `CLAUDE.md` handling.
//!
//! Historically `clam add` symlinked every `~/.claude/` item — including
//! `CLAUDE.md` — into each profile, so all profiles shared ONE global
//! `CLAUDE.md` and edits leaked across profiles (and back into the global
//! file). Instead, each profile gets a REAL `CLAUDE.md` that imports the global
//! one via Claude Code's `@import` (so shared/common rules still load) and adds
//! its own profile-local section that stays isolated.

use std::path::Path;

use anyhow::Result;

use crate::paths;

/// Marks a `CLAUDE.md` as a clam-generated per-profile overlay. Lets `add` and
/// `repair` recognize files clam already created so they never clobber a
/// hand-authored real `CLAUDE.md`.
pub const PROFILE_LOCAL_MARKER: &str = "<!-- clam:profile-local -->";

/// Builds the contents of a profile's overlay `CLAUDE.md`.
///
/// Line 1 imports the global `~/.claude/CLAUDE.md` by absolute path (one hop —
/// well within Claude Code's import-depth limit), so shared rules keep loading
/// from a single place. The profile-local section below it is unique to `name`.
pub fn overlay_contents(claude_dir: &Path, name: &str) -> String {
    let global = claude_dir.join("CLAUDE.md");
    let global = global.display();
    let marker = PROFILE_LOCAL_MARKER;
    format!(
        r"@{global}

{marker}
# Profile-Local Rules ({name})
<!-- Rules below apply ONLY to the '{name}' profile and are invisible to other
     profiles. Shared rules load via the @import on the first line. -->
"
    )
}

/// Writes the overlay `CLAUDE.md` into `profile_dir`, replacing any existing
/// symlink there. The write is atomic (tempfile + rename).
pub fn write_overlay(profile_dir: &Path, claude_dir: &Path, name: &str) -> Result<()> {
    let path = profile_dir.join("CLAUDE.md");
    if path.is_symlink() {
        std::fs::remove_file(&path)?;
    }
    paths::atomic_write(&path, &overlay_contents(claude_dir, name))
}

/// True if `profile_dir/CLAUDE.md` is the legacy shared symlink pointing at the
/// global `~/.claude/CLAUDE.md`.
pub fn is_shared_symlink(profile_dir: &Path, claude_dir: &Path) -> bool {
    let link = profile_dir.join("CLAUDE.md");
    if !link.is_symlink() {
        return false;
    }
    std::fs::read_link(&link).is_ok_and(|t| t == claude_dir.join("CLAUDE.md"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;

    fn claude_dir(tmp: &Path) -> PathBuf {
        let d = tmp.join(".claude");
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("CLAUDE.md"), "# shared\n").unwrap();
        d
    }

    #[test]
    fn test_overlay_contents_imports_global_and_names_profile() {
        let tmp = tempfile::tempdir().unwrap();
        let cd = claude_dir(tmp.path());
        let out = overlay_contents(&cd, "work");

        let first_line = out.lines().next().unwrap();
        assert_eq!(first_line, format!("@{}/CLAUDE.md", cd.display()));
        assert!(out.contains(PROFILE_LOCAL_MARKER));
        assert!(out.contains("# Profile-Local Rules (work)"));
    }

    #[test]
    fn test_write_overlay_creates_real_file() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cd = claude_dir(tmp.path());
        let pdir = tmp.path().join("profile");
        fs::create_dir_all(&pdir)?;

        write_overlay(&pdir, &cd, "p")?;

        let f = pdir.join("CLAUDE.md");
        assert!(f.is_file());
        assert!(!f.is_symlink());
        assert_eq!(fs::read_to_string(&f)?, overlay_contents(&cd, "p"));
        Ok(())
    }

    #[test]
    fn test_write_overlay_replaces_shared_symlink() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cd = claude_dir(tmp.path());
        let pdir = tmp.path().join("profile");
        fs::create_dir_all(&pdir)?;
        symlink(cd.join("CLAUDE.md"), pdir.join("CLAUDE.md"))?;
        assert!(is_shared_symlink(&pdir, &cd));

        write_overlay(&pdir, &cd, "p")?;

        assert!(!pdir.join("CLAUDE.md").is_symlink());
        assert!(!is_shared_symlink(&pdir, &cd));
        // Global file must be untouched (we replaced the link, not its target).
        assert_eq!(fs::read_to_string(cd.join("CLAUDE.md"))?, "# shared\n");
        Ok(())
    }

    #[test]
    fn test_is_shared_symlink_false_for_real_file() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cd = claude_dir(tmp.path());
        let pdir = tmp.path().join("profile");
        fs::create_dir_all(&pdir)?;
        fs::write(pdir.join("CLAUDE.md"), "real\n")?;
        assert!(!is_shared_symlink(&pdir, &cd));
        Ok(())
    }

    #[test]
    fn test_is_shared_symlink_false_when_target_differs() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cd = claude_dir(tmp.path());
        let pdir = tmp.path().join("profile");
        fs::create_dir_all(&pdir)?;
        symlink(tmp.path().join("elsewhere.md"), pdir.join("CLAUDE.md"))?;
        assert!(!is_shared_symlink(&pdir, &cd));
        Ok(())
    }
}
