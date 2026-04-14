use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

use crate::{config, output, paths};

/// Subdirs that older clam versions symlinked to the global `~/.claude/`.
/// Both hold session-scoped data (transcripts, per-session todos) and must
/// be per-profile for isolation.
const SHARED_SUBS: &[&str] = &["projects", "todos"];

type UniqueByProfile<'a> = HashMap<&'a str, Vec<&'a str>>;
type Conflicts<'a> = Vec<(&'a str, Vec<&'a str>)>;

struct ProfileState {
    name: String,
    dir: PathBuf,
    bad_subs: Vec<String>,
    owned_cwds: Vec<String>,
}

pub fn execute(dry_run: bool, profile_filter: Option<&str>, force: bool) -> Result<()> {
    let cfg = config::load()?;
    let profiles: Vec<String> = match profile_filter {
        Some(name) => {
            if !cfg.profiles.iter().any(|p| p.name == name) {
                anyhow::bail!("Profile '{name}' not found.");
            }
            vec![name.to_string()]
        }
        None => cfg.profiles.iter().map(|p| p.name.clone()).collect(),
    };

    if profiles.is_empty() {
        output::success("No profiles configured.");
        return Ok(());
    }

    let claude_dir = dirs::home_dir()
        .context("Could not determine home directory")?
        .join(".claude");

    run_repair(&profiles, &claude_dir, dry_run, force)
}

fn run_repair(
    profile_names: &[String],
    claude_dir: &Path,
    dry_run: bool,
    force: bool,
) -> Result<()> {
    let states = scan_profiles(profile_names, claude_dir)?;

    if states.is_empty() {
        output::success("All profiles already isolated. Nothing to repair.");
        return Ok(());
    }

    let (unique_by_profile, conflicts) = resolve_ownership(&states);

    print_plan(&states, &unique_by_profile, &conflicts);

    if dry_run {
        println!("\nDry run — nothing changed.");
        return Ok(());
    }

    if !force {
        if !io::stdin().is_terminal() {
            anyhow::bail!("Cannot prompt for confirmation. Use --force to skip.");
        }
        print!("\nProceed with repair? [y/N] ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    for s in &states {
        let owned = unique_by_profile.get(s.name.as_str()).cloned().unwrap_or_default();
        repair_profile(s, claude_dir, &owned)?;
    }

    output::success("Repair complete.");
    Ok(())
}

fn scan_profiles(profile_names: &[String], claude_dir: &Path) -> Result<Vec<ProfileState>> {
    let mut states = Vec::new();
    for name in profile_names {
        let dir = paths::profile_dir(name)?;
        let bad_subs: Vec<String> = SHARED_SUBS
            .iter()
            .copied()
            .filter(|sub| is_bad_symlink(&dir.join(sub), &claude_dir.join(sub)))
            .map(String::from)
            .collect();
        if bad_subs.is_empty() {
            continue;
        }
        let owned_cwds = load_claude_json_projects(&dir.join(".claude.json"))?;
        states.push(ProfileState {
            name: name.clone(),
            dir,
            bad_subs,
            owned_cwds,
        });
    }
    Ok(states)
}

fn resolve_ownership(states: &[ProfileState]) -> (UniqueByProfile<'_>, Conflicts<'_>) {
    let mut claims: HashMap<&str, Vec<&str>> = HashMap::new();
    for s in states {
        for cwd in &s.owned_cwds {
            claims.entry(cwd.as_str()).or_default().push(s.name.as_str());
        }
    }

    let unique_by_profile: HashMap<&str, Vec<&str>> = states
        .iter()
        .map(|s| {
            let cwds: Vec<&str> = s
                .owned_cwds
                .iter()
                .filter(|c| claims.get(c.as_str()).is_some_and(|v| v.len() == 1))
                .map(String::as_str)
                .collect();
            (s.name.as_str(), cwds)
        })
        .collect();

    let mut conflicts: Vec<(&str, Vec<&str>)> = claims
        .into_iter()
        .filter(|(_, v)| v.len() > 1)
        .collect();
    conflicts.sort_by_key(|(k, _)| *k);
    (unique_by_profile, conflicts)
}

fn print_plan(
    states: &[ProfileState],
    unique_by_profile: &UniqueByProfile<'_>,
    conflicts: &[(&str, Vec<&str>)],
) {
    println!("Profiles needing repair:");
    for s in states {
        let subs = s.bad_subs.join(", ");
        let n = unique_by_profile.get(s.name.as_str()).map_or(0, Vec::len);
        println!("  {} — bad symlinks: {subs}; {n} owned cwd(s) to migrate", s.name);
    }
    if !conflicts.is_empty() {
        println!("\nConflicts (claimed by multiple profiles — left in ~/.claude/):");
        for (cwd, owners) in conflicts {
            println!("  {cwd} (owners: {})", owners.join(", "));
        }
    }
}

fn repair_profile(state: &ProfileState, claude_dir: &Path, owned_cwds: &[&str]) -> Result<()> {
    for sub in &state.bad_subs {
        let target = state.dir.join(sub);
        if target.is_symlink() {
            fs::remove_file(&target)
                .with_context(|| format!("Failed to unlink {}", target.display()))?;
        }
        fs::create_dir_all(&target)
            .with_context(|| format!("Failed to create {}", target.display()))?;
    }

    let projects_src = claude_dir.join("projects");
    let projects_dst = state.dir.join("projects");
    let mut moved_session_ids: HashSet<String> = HashSet::new();

    if projects_dst.is_dir() && projects_src.is_dir() {
        for cwd in owned_cwds {
            let flat = flatten_cwd(cwd);
            let src = projects_src.join(&flat);
            if !src.is_dir() {
                continue;
            }
            for entry in fs::read_dir(&src)
                .with_context(|| format!("Failed to read {}", src.display()))?
            {
                let entry = entry?;
                let fname = entry.file_name();
                let s = fname.to_string_lossy();
                if let Some(stem) = s.strip_suffix(".jsonl") {
                    moved_session_ids.insert(stem.to_string());
                } else if entry.path().is_dir() {
                    moved_session_ids.insert(s.into_owned());
                }
            }
            let dst = projects_dst.join(&flat);
            fs::rename(&src, &dst)
                .with_context(|| format!("Failed to move {} to {}", src.display(), dst.display()))?;
        }
    }

    let todos_src = claude_dir.join("todos");
    let todos_dst = state.dir.join("todos");
    if todos_src.is_dir() && todos_dst.is_dir() {
        for entry in fs::read_dir(&todos_src)
            .with_context(|| format!("Failed to read {}", todos_src.display()))?
        {
            let entry = entry?;
            let fname = entry.file_name();
            let s = fname.to_string_lossy();
            if s.len() < 36 {
                continue;
            }
            let sid = &s[..36];
            if moved_session_ids.contains(sid) {
                let dst = todos_dst.join(&fname);
                fs::rename(entry.path(), &dst)
                    .with_context(|| format!("Failed to move todo {s}"))?;
            }
        }
    }

    Ok(())
}

fn flatten_cwd(cwd: &str) -> String {
    cwd.replace('/', "-")
}

fn is_bad_symlink(link: &Path, expected_target: &Path) -> bool {
    if !link.is_symlink() {
        return false;
    }
    fs::read_link(link).is_ok_and(|t| t == expected_target)
}

fn load_claude_json_projects(path: &Path) -> Result<Vec<String>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    let v: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    let Some(projects) = v.get("projects").and_then(Value::as_object) else {
        return Ok(Vec::new());
    };
    Ok(projects.keys().cloned().collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::ENV_LOCK;
    use std::os::unix::fs::symlink;

    #[test]
    fn test_flatten_cwd() {
        assert_eq!(
            flatten_cwd("/Users/jose/projects/personal/clam"),
            "-Users-jose-projects-personal-clam"
        );
    }

    #[test]
    fn test_is_bad_symlink_true_when_points_into_claude_dir() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let claude_projects = tmp.path().join(".claude").join("projects");
        fs::create_dir_all(&claude_projects)?;
        let profile = tmp.path().join("profile");
        fs::create_dir_all(&profile)?;
        let link = profile.join("projects");
        symlink(&claude_projects, &link)?;

        assert!(is_bad_symlink(&link, &claude_projects));
        Ok(())
    }

    #[test]
    fn test_is_bad_symlink_false_for_real_dir() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let p = tmp.path().join("projects");
        fs::create_dir(&p)?;
        assert!(!is_bad_symlink(&p, &tmp.path().join("target")));
        Ok(())
    }

    #[test]
    fn test_is_bad_symlink_false_when_symlink_target_differs() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let other = tmp.path().join("other");
        fs::create_dir(&other)?;
        let link = tmp.path().join("link");
        symlink(&other, &link)?;
        assert!(!is_bad_symlink(&link, &tmp.path().join("expected")));
        Ok(())
    }

    #[test]
    fn test_load_claude_json_projects_returns_cwd_keys() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let p = tmp.path().join(".claude.json");
        fs::write(
            &p,
            r#"{"projects": {"/a/b": {"x":1}, "/c/d": {"y":2}}, "other": "ignored"}"#,
        )?;
        let mut cwds = load_claude_json_projects(&p)?;
        cwds.sort();
        assert_eq!(cwds, vec!["/a/b".to_string(), "/c/d".to_string()]);
        Ok(())
    }

    #[test]
    fn test_load_claude_json_missing_file_is_empty() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let cwds = load_claude_json_projects(&tmp.path().join("missing.json"))?;
        assert!(cwds.is_empty());
        Ok(())
    }

    #[test]
    fn test_load_claude_json_without_projects_key_is_empty() -> Result<()> {
        let tmp = tempfile::tempdir()?;
        let p = tmp.path().join(".claude.json");
        fs::write(&p, r#"{"other":"value"}"#)?;
        assert!(load_claude_json_projects(&p)?.is_empty());
        Ok(())
    }

    // --- Integration-style tests for run_repair ---

    /// Build a synthetic ~/.claude/ + ~/.clam/profiles/<name>/ layout under tmp.
    /// Returns the tmp dir guard so the caller keeps it alive.
    fn setup(tmp: &Path) -> (PathBuf, PathBuf) {
        let claude_dir = tmp.join(".claude");
        fs::create_dir_all(claude_dir.join("projects")).unwrap();
        fs::create_dir_all(claude_dir.join("todos")).unwrap();
        let clam_home = tmp.join(".clam");
        fs::create_dir_all(&clam_home).unwrap();
        (claude_dir, clam_home)
    }

    fn make_bad_profile(claude_dir: &Path, name: &str, cwds: &[&str]) -> Result<PathBuf> {
        let pdir = paths::profile_dir(name)?;
        fs::create_dir_all(&pdir)?;
        symlink(claude_dir.join("projects"), pdir.join("projects"))?;
        symlink(claude_dir.join("todos"), pdir.join("todos"))?;
        let projects_json: String = cwds
            .iter()
            .map(|c| format!(r#""{c}": {{}}"#))
            .collect::<Vec<_>>()
            .join(", ");
        fs::write(
            pdir.join(".claude.json"),
            format!(r#"{{"projects": {{{projects_json}}}}}"#),
        )?;
        Ok(pdir)
    }

    fn seed_session(claude_dir: &Path, cwd: &str, sid: &str) -> Result<()> {
        let flat = flatten_cwd(cwd);
        let proj = claude_dir.join("projects").join(&flat);
        fs::create_dir_all(&proj)?;
        fs::write(proj.join(format!("{sid}.jsonl")), "{}")?;
        fs::write(
            claude_dir.join("todos").join(format!("{sid}-agent-{sid}.json")),
            "{}",
        )?;
        Ok(())
    }

    #[test]
    fn test_run_repair_migrates_owned_session_and_todo() -> Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());
        let (claude_dir, _) = setup(tmp.path());

        let cwd = "/u/me/proj";
        let sid = "11111111-1111-1111-1111-111111111111";
        seed_session(&claude_dir, cwd, sid)?;
        let pdir = make_bad_profile(&claude_dir, "p", &[cwd])?;

        run_repair(&["p".to_string()], &claude_dir, false, true)?;

        // Symlink replaced with real dir
        assert!(pdir.join("projects").is_dir());
        assert!(!pdir.join("projects").is_symlink());
        // Session moved
        let flat = flatten_cwd(cwd);
        assert!(pdir.join("projects").join(&flat).join(format!("{sid}.jsonl")).exists());
        // Todo moved
        assert!(pdir.join("todos").join(format!("{sid}-agent-{sid}.json")).exists());
        // Global pool emptied of the moved cwd
        assert!(!claude_dir.join("projects").join(&flat).exists());
        assert!(!claude_dir.join("todos").join(format!("{sid}-agent-{sid}.json")).exists());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    #[test]
    fn test_run_repair_is_noop_when_no_bad_symlinks() -> Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());
        let (claude_dir, _) = setup(tmp.path());

        let pdir = paths::profile_dir("clean")?;
        fs::create_dir_all(&pdir)?;
        fs::create_dir(pdir.join("projects"))?;
        fs::create_dir(pdir.join("todos"))?;

        run_repair(&["clean".to_string()], &claude_dir, false, true)?;

        assert!(pdir.join("projects").is_dir());
        assert!(pdir.join("todos").is_dir());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    #[test]
    fn test_run_repair_leaves_conflict_cwds_in_global_pool() -> Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());
        let (claude_dir, _) = setup(tmp.path());

        let shared = "/shared/cwd";
        let sid = "22222222-2222-2222-2222-222222222222";
        seed_session(&claude_dir, shared, sid)?;
        make_bad_profile(&claude_dir, "a", &[shared])?;
        make_bad_profile(&claude_dir, "b", &[shared])?;

        run_repair(&["a".to_string(), "b".to_string()], &claude_dir, false, true)?;

        // Shared cwd's session stays in global pool (unresolved conflict)
        let flat = flatten_cwd(shared);
        assert!(claude_dir.join("projects").join(&flat).is_dir());
        assert!(!paths::profile_dir("a")?.join("projects").join(&flat).exists());
        assert!(!paths::profile_dir("b")?.join("projects").join(&flat).exists());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    #[test]
    fn test_run_repair_dry_run_touches_nothing() -> Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());
        let (claude_dir, _) = setup(tmp.path());

        let cwd = "/only/here";
        let sid = "33333333-3333-3333-3333-333333333333";
        seed_session(&claude_dir, cwd, sid)?;
        let pdir = make_bad_profile(&claude_dir, "p", &[cwd])?;

        run_repair(&["p".to_string()], &claude_dir, true, true)?;

        // Symlinks still intact, nothing moved
        assert!(pdir.join("projects").is_symlink());
        let flat = flatten_cwd(cwd);
        assert!(claude_dir.join("projects").join(&flat).is_dir());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }

    #[test]
    fn test_run_repair_leaves_orphan_todos_in_global_pool() -> Result<()> {
        let _guard = ENV_LOCK.lock().unwrap();
        let tmp = tempfile::tempdir()?;
        std::env::set_var("CLAM_HOME", tmp.path());
        let (claude_dir, _) = setup(tmp.path());

        let cwd = "/owned";
        let owned_sid = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        seed_session(&claude_dir, cwd, owned_sid)?;

        // Orphan todo — no matching session in any owned cwd
        let orphan_sid = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        fs::write(
            claude_dir
                .join("todos")
                .join(format!("{orphan_sid}-agent-{orphan_sid}.json")),
            "{}",
        )?;

        make_bad_profile(&claude_dir, "p", &[cwd])?;

        run_repair(&["p".to_string()], &claude_dir, false, true)?;

        // Orphan todo stayed behind
        assert!(claude_dir
            .join("todos")
            .join(format!("{orphan_sid}-agent-{orphan_sid}.json"))
            .exists());
        // Owned todo moved
        assert!(paths::profile_dir("p")?
            .join("todos")
            .join(format!("{owned_sid}-agent-{owned_sid}.json"))
            .exists());

        std::env::remove_var("CLAM_HOME");
        Ok(())
    }
}
