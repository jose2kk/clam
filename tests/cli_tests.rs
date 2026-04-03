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

// ── remove command tests ──

#[test]
fn remove_with_force_deletes_profile() {
    let home = TempDir::new().unwrap();

    // Add two profiles (first auto-activates to "work")
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).args(["add", "personal"]).assert().success();

    // Switch to work so personal is not active
    clmux(&home).args(["use", "work"]).assert().success();

    // Remove personal with --force
    clmux(&home)
        .args(["remove", "personal", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    // Profile directory should be gone
    assert!(
        !home.path().join("profiles").join("personal").exists(),
        "Profile directory should be removed"
    );

    // config.toml should not contain "personal"
    let config_content = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
    assert!(
        !config_content.contains("personal"),
        "config.toml should not contain 'personal', got: {}",
        config_content
    );
}

#[test]
fn remove_active_profile_refused() {
    let home = TempDir::new().unwrap();

    // Add profile (auto-activates)
    clmux(&home).args(["add", "work"]).assert().success();

    // Try to remove active profile
    clmux(&home)
        .args(["remove", "work", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot remove active profile"))
        .stderr(predicate::str::contains("clmux use"));
}

#[test]
fn remove_nonexistent_profile_fails() {
    let home = TempDir::new().unwrap();

    clmux(&home)
        .args(["remove", "nonexistent", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn remove_no_force_non_tty_fails() {
    let home = TempDir::new().unwrap();

    // Add two profiles, switch so "personal" is not active
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).args(["add", "personal"]).assert().success();

    // Remove without --force (test runner has non-TTY stdin)
    clmux(&home)
        .args(["remove", "personal"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot prompt"))
        .stderr(predicate::str::contains("--force"));
}

#[test]
fn remove_last_non_active_profile_allowed() {
    let home = TempDir::new().unwrap();

    // Add two profiles
    clmux(&home).args(["add", "a"]).assert().success();
    clmux(&home).args(["add", "b"]).assert().success();

    // Activate "b"
    clmux(&home).args(["use", "b"]).assert().success();

    // Remove "a" (the only non-active profile)
    clmux(&home)
        .args(["remove", "a", "--force"])
        .assert()
        .success();

    // Only "b" should remain in config
    let config_content = std::fs::read_to_string(home.path().join("config.toml")).unwrap();
    assert!(
        !config_content.contains("\"a\""),
        "config.toml should not contain 'a', got: {}",
        config_content
    );
    assert!(
        config_content.contains("\"b\""),
        "config.toml should still contain 'b', got: {}",
        config_content
    );
}

// ── end-to-end lifecycle tests ──

#[test]
fn test_full_lifecycle() {
    let home = TempDir::new().unwrap();

    // Start with nothing
    clmux(&home).arg("list").assert().success().stdout("");
    clmux(&home).arg("current").assert().code(1).stdout("");

    // Add first profile -- auto-activates (D-04)
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home).arg("current").assert().success().stdout("work\n");

    // Add second profile -- does NOT auto-activate (D-04)
    clmux(&home).args(["add", "personal"]).assert().success();
    clmux(&home).arg("current").assert().success().stdout("work\n");

    // List shows both, work is active (D-01)
    clmux(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("* work"))
        .stdout(predicate::str::contains("  personal"));

    // Switch to personal
    clmux(&home).args(["use", "personal"]).assert().success();
    clmux(&home)
        .arg("current")
        .assert()
        .success()
        .stdout("personal\n");

    // Status shows personal info (D-02)
    clmux(&home)
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("Profile: personal"))
        .stdout(predicate::str::contains("Status:"));

    // Remove work (not active, use --force for non-TTY)
    clmux(&home)
        .args(["remove", "work", "--force"])
        .assert()
        .success();

    // Verify work is gone
    clmux(&home)
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("personal"))
        .stdout(predicate::str::contains("work").not());

    // Cannot remove active profile (D-07)
    clmux(&home)
        .args(["remove", "personal", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot remove active profile"));
}

#[test]
fn test_error_messages_are_actionable() {
    let home = TempDir::new().unwrap();

    // Use nonexistent profile -> suggests clmux list
    clmux(&home)
        .args(["use", "ghost"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("clmux list"));

    // Remove nonexistent profile -> suggests clmux list
    clmux(&home)
        .args(["remove", "ghost", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("clmux list"));

    // Remove active profile -> suggests clmux use
    clmux(&home).args(["add", "work"]).assert().success();
    clmux(&home)
        .args(["remove", "work", "--force"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("clmux use"));

    // Status with no profile -> suggests clmux add
    let home2 = TempDir::new().unwrap();
    clmux(&home2)
        .arg("status")
        .assert()
        .failure()
        .stderr(predicate::str::contains("clmux add"));
}

#[test]
fn test_name_validation_edge_cases() {
    let home = TempDir::new().unwrap();

    // Path traversal attempts
    clmux(&home).args(["add", "../evil"]).assert().failure();
    clmux(&home)
        .args(["add", "../../etc/passwd"])
        .assert()
        .failure();
    clmux(&home).args(["add", "."]).assert().failure();
    clmux(&home).args(["add", ".."]).assert().failure();

    // Invalid characters
    clmux(&home).args(["add", "has space"]).assert().failure();
    clmux(&home).args(["add", "has/slash"]).assert().failure();
    clmux(&home).args(["add", ""]).assert().failure();

    // Valid names
    clmux(&home).args(["add", "valid-name"]).assert().success();
    clmux(&home).args(["add", "valid_name"]).assert().success();
    clmux(&home).args(["add", "CamelCase"]).assert().success();
    clmux(&home).args(["add", "name123"]).assert().success();
}

#[test]
fn test_exit_codes() {
    let home = TempDir::new().unwrap();

    // Success cases: exit 0
    clmux(&home).args(["add", "work"]).assert().code(0);
    clmux(&home).arg("list").assert().code(0);
    clmux(&home).arg("current").assert().code(0);
    clmux(&home).arg("status").assert().code(0);
    clmux(&home).args(["use", "work"]).assert().code(0);

    // Error cases: exit 1
    clmux(&home).args(["use", "nonexistent"]).assert().code(1);
    clmux(&home).args(["add", "../evil"]).assert().code(1);
    clmux(&home)
        .args(["remove", "work", "--force"])
        .assert()
        .code(1); // active profile

    // Special: current with no active = exit 1
    let home2 = TempDir::new().unwrap();
    clmux(&home2).arg("current").assert().code(1);
}

#[cfg(unix)]
#[test]
fn test_profile_dir_permissions() {
    let home = TempDir::new().unwrap();
    clmux(&home).args(["add", "secure"]).assert().success();
    let metadata = std::fs::metadata(home.path().join("profiles/secure")).unwrap();
    let mode = metadata.permissions().mode() & 0o777;
    assert_eq!(
        mode, 0o700,
        "Profile dir should have 0700 permissions, got {:o}",
        mode
    );
}
