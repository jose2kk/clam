---
phase: 02-launch-and-shell-integration
verified: 2026-04-03T19:45:00Z
status: passed
score: 10/10 must-haves verified
re_verification: false
---

# Phase 2: Launch and Shell Integration — Verification Report

**Phase Goal:** Users can launch Claude Code scoped to the active profile with clean environment isolation
**Verified:** 2026-04-03T19:45:00Z
**Status:** passed
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths

All truths drawn from must_haves in 02-01-PLAN.md and 02-02-PLAN.md.

| #  | Truth | Status | Evidence |
|----|-------|--------|----------|
| 1  | `clmux run` launches a process with `CLAUDE_CONFIG_DIR` set to the active profile directory | VERIFIED | `src/commands/run.rs:73` sets `cmd.env("CLAUDE_CONFIG_DIR", &dir)`; `test_run_passes_claude_config_dir` confirms value contains `/profiles/test` |
| 2  | `clmux run --profile <name>` uses a specific profile without modifying state.toml | VERIFIED | `resolve_profile` branch for `Some(name)` calls `config::load()` only, never writes state; `test_run_with_profile_flag` asserts `state.toml` still reads `active = "work"` after running with `--profile personal` |
| 3  | `clmux run` replaces the clmux process via `exec()` — no orphaned children | VERIFIED | `src/commands/run.rs:1` imports `std::os::unix::process::CommandExt`; line 77 calls `cmd.exec()`, not `spawn()`; no `wait()` present |
| 4  | `clmux run` strips all `CLAUDE_*` and `ANTHROPIC_*` env vars before launching | VERIFIED | `src/commands/run.rs:66-70` iterates `std::env::vars()` removing keys matching both prefixes; `test_run_strips_claude_env_vars` confirms `CLAUDE_SOMETHING=leak` and `ANTHROPIC_KEY=secret` do not appear in child env output |
| 5  | `clmux run -- <cmd> [args]` passes through to an arbitrary command | VERIFIED | `src/commands/run.rs:56-60` sets binary from `args[0]` and `cmd_args` from `args[1..]`; tests use absolute paths to mock scripts that execute successfully |
| 6  | `clmux run` with no args defaults to launching `claude` | VERIFIED | `src/commands/run.rs:56-58` checks `args.is_empty()` and hard-codes `binary = "claude"` |
| 7  | `eval "$(clmux env)"` sets `CLAUDE_CONFIG_DIR` and `CLMUX_PROFILE` in the shell | VERIFIED | `src/commands/env.rs:14-15` prints `export CLAUDE_CONFIG_DIR="...";` and `export CLMUX_PROFILE="...";`; `test_env_outputs_valid_export_syntax` confirms both lines appear and are valid shell |
| 8  | `clmux env` emits `unset` statements for existing `CLAUDE_*/ANTHROPIC_*` vars | VERIFIED | `src/commands/env.rs:7-11` iterates env vars and prints `unset {key};` for matching keys; `test_env_includes_unset_for_existing_vars` confirms `unset CLAUDE_SOMETHING;` and `unset ANTHROPIC_KEY;` appear |
| 9  | `clmux env` output is valid POSIX shell syntax (every line starts with `export` or `unset`) | VERIFIED | `src/commands/env.rs` emits only `unset ...;` and `export ...="...";` lines; `test_env_outputs_valid_export_syntax` asserts every non-empty line satisfies this constraint |
| 10 | `clmux status` shows `CLAUDE_CONFIG_DIR` value and directory item count | VERIFIED | `src/commands/status.rs:32` prints `Config: CLAUDE_CONFIG_DIR=...`; lines 34-39 print `Items: N file(s)` when directory exists; `test_status_shows_config_dir` and `test_status_shows_item_count` confirm both outputs |

**Score:** 10/10 truths verified

---

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `src/commands/run.rs` | run command with resolve_profile, env sanitization, exec | VERIFIED | 84 lines; exports `execute` and `pub(crate) resolve_profile`; uses `CommandExt::exec()` |
| `src/cli.rs` | Run and Env CLI variants | VERIFIED | Contains `Run { profile: Option<String>, args: Vec<String> }` with `trailing_var_arg = true` and `allow_hyphen_values = true`; contains `Env,` variant |
| `src/main.rs` | Routing for Run and Env commands | VERIFIED | Match arms at lines 27-30 route `Commands::Run` to `commands::run::execute` and `Commands::Env` to `commands::env::execute` |
| `src/commands/env.rs` | env command with shell export output | VERIFIED | 18 lines; calls `super::run::resolve_profile(None)`; emits unset/export lines; no `todo!` |
| `src/commands/status.rs` | enhanced status with config dir and item count | VERIFIED | Line 32 emits `Config: CLAUDE_CONFIG_DIR=...`; lines 34-39 emit `Items: N file(s)` conditionally |
| `tests/cli_tests.rs` | integration tests for run, env, and enhanced status | VERIFIED | Contains all 11 required test functions; 43 tests total, all pass |

---

### Key Link Verification

#### From 02-01-PLAN.md

| From | To | Via | Status | Evidence |
|------|----|-----|--------|----------|
| `src/commands/run.rs` | `src/state.rs` | `state::load()` for active profile | WIRED | `src/commands/run.rs:28` calls `state::load()` in the `None` branch of `resolve_profile` |
| `src/commands/run.rs` | `src/config.rs` | `config::load()` for profile validation | WIRED | `src/commands/run.rs:18` calls `config::load()` in the `Some(name)` branch |
| `src/commands/run.rs` | `src/paths.rs` | `paths::profile_dir()` for directory resolution | WIRED | `src/commands/run.rs:36` calls `paths::profile_dir(&name)` |
| `src/main.rs` | `src/commands/run.rs` | command routing | WIRED | `src/main.rs:27-29` dispatches `Commands::Run` to `commands::run::execute` |

#### From 02-02-PLAN.md

| From | To | Via | Status | Evidence |
|------|----|-----|--------|----------|
| `src/commands/env.rs` | `src/commands/run.rs` | `super::run::resolve_profile` | WIRED | `src/commands/env.rs:4` calls `super::run::resolve_profile(None)` |
| `src/commands/env.rs` | stdout | `println!` with `export CLAUDE_CONFIG_DIR` | WIRED | `src/commands/env.rs:14` prints `export CLAUDE_CONFIG_DIR="{}";` |
| `src/commands/status.rs` | `src/paths.rs` | `paths::profile_dir` for directory path | WIRED | `src/commands/status.rs:18` calls `paths::profile_dir(&active)` |

---

### Data-Flow Trace (Level 4)

`clmux run` and `clmux env` do not render data — they exec into another process or print shell syntax. The "data" is the profile directory path, which flows from `state::load()` or `config::load()` through `resolve_profile()` directly to `cmd.env()` or `println!`. No intermediate state caches or props.

`clmux status` renders the directory path and item count from live filesystem reads, not hardcoded values.

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| `src/commands/run.rs` | `dir` (PathBuf) | `state::load()` or `config::load()` + `paths::profile_dir()` | Yes — reads state.toml at runtime | FLOWING |
| `src/commands/env.rs` | `(name, dir)` | `super::run::resolve_profile(None)` | Yes — delegates to same live reads | FLOWING |
| `src/commands/status.rs` | `item_count` | `std::fs::read_dir(&dir).count()` | Yes — live filesystem read | FLOWING |

---

### Behavioral Spot-Checks

Integration tests cover all key behaviors end-to-end. The test suite was executed directly rather than running the binary interactively (exec() makes interactive verification impractical without a real `claude` binary).

| Behavior | Test | Result | Status |
|----------|------|--------|--------|
| `clmux run` sets CLAUDE_CONFIG_DIR to profile dir | `test_run_passes_claude_config_dir` | PASS | VERIFIED |
| `clmux run` strips CLAUDE_*/ANTHROPIC_* vars | `test_run_strips_claude_env_vars` | PASS | VERIFIED |
| `clmux run --profile` uses named profile without state change | `test_run_with_profile_flag` | PASS | VERIFIED |
| `clmux run` fails on missing directory | `test_run_missing_directory_fails` | PASS | VERIFIED |
| `clmux run` fails with no active profile | `test_run_no_active_profile_fails` | PASS | VERIFIED |
| `clmux env` outputs valid POSIX syntax | `test_env_outputs_valid_export_syntax` | PASS | VERIFIED |
| `clmux env` emits unset for existing vars | `test_env_includes_unset_for_existing_vars` | PASS | VERIFIED |
| `clmux status` shows CLAUDE_CONFIG_DIR | `test_status_shows_config_dir` | PASS | VERIFIED |
| `clmux status` shows item count | `test_status_shows_item_count` | PASS | VERIFIED |
| Full test suite (43 tests) | `cargo test` | 43 passed, 0 failed | VERIFIED |

---

### Requirements Coverage

All six Phase 2 requirement IDs are claimed across the two plans. No orphaned IDs in REQUIREMENTS.md for Phase 2.

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| LNCH-01 | 02-01 | Launch Claude Code scoped to active profile via `clmux run`, setting `CLAUDE_CONFIG_DIR` | SATISFIED | `run.rs` sets `CLAUDE_CONFIG_DIR`; `test_run_passes_claude_config_dir` confirms |
| LNCH-02 | 02-01 | Launch Claude Code with a specific profile via `clmux run --profile <name>` | SATISFIED | `cli.rs` exposes `--profile` flag; `resolve_profile` handles override; `test_run_with_profile_flag` confirms |
| LNCH-03 | 02-01 | `clmux run` uses Unix `exec()` to replace the process | SATISFIED | `run.rs:1,77` uses `CommandExt::exec()`; no `spawn()` present |
| LNCH-04 | 02-01 | `clmux run` sanitizes `ANTHROPIC_API_KEY`, `CLAUDE_CONFIG_DIR`, etc. before launching | SATISFIED | `run.rs:66-70` strips all `CLAUDE_*` and `ANTHROPIC_*` prefixes; `test_run_strips_claude_env_vars` confirms |
| LNCH-05 | 02-02 | Print shell environment exports via `clmux env` for `eval "$(clmux env)"` | SATISFIED | `env.rs` outputs valid POSIX export/unset lines; 3 integration tests confirm |
| LNCH-06 | 02-02 | Verbose active profile info via `clmux status` (name, path, directory health) | SATISFIED | `status.rs` shows Profile, Path, Status, Config, and Items lines; 2 integration tests confirm |

**Orphaned requirements check:** REQUIREMENTS.md traceability table maps LNCH-01 through LNCH-06 to Phase 2. All six are claimed in plans. No orphaned requirements.

---

### Anti-Patterns Found

| File | Pattern | Severity | Impact |
|------|---------|----------|--------|
| — | — | — | None found |

Scan conducted on all Phase 2 source files. No TODO/FIXME/placeholder comments, no empty return stubs, no hardcoded empty data, no console.log-only implementations. The `todo!()` placeholder that existed in `src/commands/env.rs` after Plan 01 was fully replaced by Plan 02 — the current file contains no `todo!`.

---

### Human Verification Required

#### 1. End-to-end exec with real `claude` binary

**Test:** With `claude` installed and a profile created (`clmux add work`), run `clmux run` from a terminal that has `ANTHROPIC_API_KEY` set.
**Expected:** Claude Code starts with `CLAUDE_CONFIG_DIR` pointing to `~/.clmux/profiles/work/`; the `ANTHROPIC_API_KEY` env var is absent inside the Claude Code session (check with `echo $ANTHROPIC_API_KEY`).
**Why human:** `exec()` replaces the clmux process, making automated assertion of the resulting environment infeasible without a real `claude` binary.

#### 2. `eval "$(clmux env)"` shell integration

**Test:** In an interactive shell with `CLAUDE_SOMETHING=old` set, run `eval "$(clmux env)"` after setting an active profile.
**Expected:** `echo $CLAUDE_CONFIG_DIR` shows the profile directory; `echo $CLAUDE_SOMETHING` is empty.
**Why human:** Requires an interactive shell session; the eval side-effect cannot be asserted from a subprocess.

---

### Gaps Summary

No gaps. All 10 must-have truths verified, all 6 artifacts pass all four levels, all 7 key links confirmed wired, all 6 requirement IDs satisfied, no blocker anti-patterns found, and all 43 integration tests pass.

---

_Verified: 2026-04-03T19:45:00Z_
_Verifier: Claude (gsd-verifier)_
