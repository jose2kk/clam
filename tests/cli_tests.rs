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

// ── list command tests ──

#[test]
fn test_list_shows_active_marker() {
    let home = TempDir::new().unwrap();

    // Add two profiles (first auto-activates)
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).args(["add", "personal"]).assert().success();

    let output = clmux(&home)
        .args(["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("* work"), "Active profile should have '* ' prefix, got: {}", stdout);
    assert!(stdout.contains("  personal"), "Inactive profile should have '  ' prefix, got: {}", stdout);
}

#[test]
fn test_list_empty() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["list"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_list_no_ansi_when_piped() {
    let home = TempDir::new().unwrap();

    clmux(&home).args(["add", "work"]).assert().success();

    // assert_cmd runs without a TTY, so output should have no ANSI codes
    let output = clmux(&home)
        .args(["list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    // ANSI escape codes start with \x1b[
    assert!(!stdout.contains('\x1b'), "Piped output should not contain ANSI escape codes, got: {:?}", stdout);
}

// ── current command tests ──

#[test]
fn test_current_prints_active() {
    let home = TempDir::new().unwrap();

    clmux(&home).args(["add", "work"]).assert().success();

    clmux(&home)
        .args(["current"])
        .assert()
        .success()
        .stdout("work\n");
}

#[test]
fn test_current_no_active_exits_1() {
    let home = TempDir::new().unwrap();

    let output = clmux(&home)
        .args(["current"])
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();

    assert!(output.is_empty(), "stdout should be empty when no active profile");
}

// ── status command tests ──

#[test]
fn test_status_shows_active_profile() {
    let home = TempDir::new().unwrap();

    clmux(&home).args(["add", "work"]).assert().success();

    let output = clmux(&home)
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("Profile: work"), "Should show profile name, got: {}", stdout);
    assert!(stdout.contains("Path:"), "Should show path line, got: {}", stdout);
    assert!(stdout.contains("Status:"), "Should show status line, got: {}", stdout);
    assert!(stdout.contains("ok"), "Directory exists so status should be 'ok', got: {}", stdout);
}

#[test]
fn test_status_no_active_exits_1() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["status"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("No active profile"));
}

#[test]
fn test_status_missing_dir() {
    let home = TempDir::new().unwrap();

    clmux(&home).args(["add", "work"]).assert().success();

    // Delete the profile directory to simulate missing state
    let profile_dir = home.path().join("profiles").join("work");
    std::fs::remove_dir_all(&profile_dir).unwrap();

    let output = clmux(&home)
        .args(["status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("missing"), "Should show 'missing' when dir doesn't exist, got: {}", stdout);
}
