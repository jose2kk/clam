use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::os::unix::fs::PermissionsExt;

fn clmux(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("clmux").unwrap();
    cmd.env("CLMUX_HOME", home.path());
    cmd
}

#[test]
fn add_creates_profile_dir_and_config() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created"));

    // Profile directory exists
    let profile_dir = home.path().join("profiles").join("work");
    assert!(profile_dir.exists(), "Profile directory should exist");

    // Profile directory has 0700 permissions
    let meta = std::fs::metadata(&profile_dir).unwrap();
    let mode = meta.permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "Profile dir should have 0700 permissions");

    // config.toml contains the profile
    let config_content = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
    assert!(
        config_content.contains("work"),
        "config.toml should contain profile name"
    );
}

#[test]
fn add_first_profile_auto_activates() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "work"])
        .assert()
        .success()
        .stdout(predicate::str::contains("activated"));

    // state.toml should have active = "work"
    let state_content = std::fs::read_to_string(home.path().join("state.toml")).unwrap();
    assert!(
        state_content.contains("active = \"work\""),
        "state.toml should have active = \"work\", got: {}",
        state_content
    );
}

#[test]
fn add_second_profile_does_not_auto_activate() {
    let home = TempDir::new().unwrap();

    // Add first profile (auto-activates)
    clmux(&home).args(["add", "work"]).assert().success();

    // Add second profile (should NOT change active)
    clmux(&home)
        .args(["add", "personal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created profile"));

    // state.toml should still have active = "work"
    let state_content = std::fs::read_to_string(home.path().join("state.toml")).unwrap();
    assert!(
        state_content.contains("active = \"work\""),
        "active should still be 'work', got: {}",
        state_content
    );

    // personal profile directory should exist
    assert!(home.path().join("profiles").join("personal").exists());
}

#[test]
fn add_rejects_path_traversal() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "../evil"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn add_rejects_duplicate() {
    let home = TempDir::new().unwrap();

    // Add first time
    clmux(&home).args(["add", "work"]).assert().success();

    // Add again -- should fail
    clmux(&home)
        .args(["add", "work"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn add_rejects_empty_name() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", ""])
        .assert()
        .failure()
        .stderr(predicate::str::contains("empty"));
}

#[test]
fn add_rejects_dot_name() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn add_rejects_dotdot_name() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", ".."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn add_rejects_name_with_spaces() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "my profile"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}

#[test]
fn add_accepts_hyphens_and_underscores() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["add", "my-work_profile"])
        .assert()
        .success();

    assert!(home.path().join("profiles").join("my-work_profile").exists());
}

// ── use command tests ──

#[test]
fn use_switches_active_profile() {
    let home = TempDir::new().unwrap();

    // Add two profiles (first auto-activates)
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).args(["add", "personal"]).assert().success();

    // Switch to personal
    clmux(&home)
        .args(["use", "personal"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Switched"));

    // Verify state.toml updated
    let state_content = std::fs::read_to_string(home.path().join("state.toml")).unwrap();
    assert!(
        state_content.contains("active = \"personal\""),
        "state.toml should have active = \"personal\", got: {}",
        state_content
    );
}

#[test]
fn use_nonexistent_profile_fails() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["use", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"))
        .stderr(predicate::str::contains("clmux list"));
}

#[test]
fn use_switches_between_profiles() {
    let home = TempDir::new().unwrap();

    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).args(["add", "personal"]).assert().success();

    // Switch to personal
    clmux(&home).args(["use", "personal"]).assert().success();

    // Switch back to work
    clmux(&home).args(["use", "work"]).assert().success();

    // Verify state.toml has work
    let state_content = std::fs::read_to_string(home.path().join("state.toml")).unwrap();
    assert!(
        state_content.contains("active = \"work\""),
        "state.toml should have active = \"work\", got: {}",
        state_content
    );
}

#[test]
fn use_invalid_name_fails() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["use", "../evil"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}
