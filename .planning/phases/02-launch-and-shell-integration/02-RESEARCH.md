# Phase 2: Launch and Shell Integration - Research

**Researched:** 2026-04-03
**Domain:** Unix process execution, environment variable sanitization, shell integration (Rust CLI)
**Confidence:** HIGH

## Summary

Phase 2 adds three commands (`run`, `env`, `status` enhancement) that bridge clmux's profile management to Claude Code process launching. The core technical challenge is Unix `exec()` semantics (replacing the clmux process with Claude Code), environment variable sanitization (stripping 140+ `CLAUDE_*` and `ANTHROPIC_*` vars before injecting profile-scoped ones), and generating POSIX shell export syntax for `eval "$(clmux env)"`.

The existing codebase provides all necessary building blocks: `state::load()` for active profile, `config::load()` for profile validation, `paths::profile_dir()` for directory resolution, and `profile::validate_profile_name()` for `--profile` flag validation. The commands follow the established pattern of one file per subcommand in `src/commands/` with `pub fn execute(...)`.

**Primary recommendation:** Use `std::os::unix::process::CommandExt::exec()` for `clmux run` (replaces process, zero orphan risk), pattern-match `CLAUDE_*` and `ANTHROPIC_*` prefixes for env sanitization rather than maintaining a hardcoded list, and keep `clmux env` output to simple `export` statements with shell-safe quoting.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Broad denylist -- strip ALL `CLAUDE_*` and `ANTHROPIC_*` env vars before launching Claude Code to prevent leakage from parent shell
- **D-02:** After stripping, inject `CLAUDE_CONFIG_DIR` pointing to the active profile's directory AND `CLMUX_PROFILE` with the profile name
- **D-03:** Bare `clmux run` (no `--` args) defaults to launching the `claude` binary with the active profile's environment
- **D-04:** `clmux run -- <cmd> [args]` passes through to an arbitrary command (not just `claude`)
- **D-05:** `clmux run --profile <name>` uses a specific profile without switching the global active state
- **D-06:** Use Unix `exec()` to replace the clmux process -- no orphaned children, correct signal forwarding. Standard for wrapper tools.
- **D-07:** `clmux env` outputs bash/POSIX `export` syntax only -- `export CLAUDE_CONFIG_DIR=... ; export CLMUX_PROFILE=...`
- **D-08:** No shell auto-detection. Fish/PowerShell users use `clmux run` instead.
- **D-09:** `clmux env` exports both `CLAUDE_CONFIG_DIR` and `CLMUX_PROFILE` (matches what `clmux run` injects)

### Claude's Discretion
- Exact list of `CLAUDE_*` and `ANTHROPIC_*` env var patterns to strip (research should identify the full set)
- Whether `clmux env` also emits `unset` statements for stripped vars
- Error message wording when `claude` binary is not found in PATH
- Status directory health detail beyond exists/missing

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| LNCH-01 | Launch Claude Code scoped to active profile via `clmux run [-- args]`, setting `CLAUDE_CONFIG_DIR` | exec() pattern, env sanitization, profile resolution |
| LNCH-02 | Launch with specific profile via `clmux run --profile <name> [-- args]` | clap `--profile` flag with TrailingVarArg, profile validation reuse |
| LNCH-03 | `clmux run` uses Unix `exec()` to replace the process | `std::os::unix::process::CommandExt::exec()` API |
| LNCH-04 | `clmux run` sanitizes environment variables before launching | Prefix-based stripping of CLAUDE_* and ANTHROPIC_* |
| LNCH-05 | Shell environment exports via `clmux env` for eval integration | POSIX export syntax, shell-safe quoting |
| LNCH-06 | Verbose active profile info via `clmux status` | Enhancement of existing status command |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Prioritize readability over cleverness
- Follow existing project patterns strictly
- Avoid introducing unnecessary abstractions
- Always suggest how to test changes
- Never assume correctness without verification
- If task is complex, propose a plan first

## Standard Stack

### Core (already in Cargo.toml)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| clap | 4.6 | CLI argument parsing | Add `Run` and `Env` variants to `Commands` enum |
| anyhow | 1.0 | Error handling | Existing pattern with `.context()` chaining |
| std::os::unix::process::CommandExt | stdlib | `exec()` for process replacement | No new dependency needed |
| std::env | stdlib | Env var iteration and removal | `std::env::vars()` for prefix matching |

### No New Dependencies Required

Phase 2 needs no new crates. All functionality is covered by the existing stack plus Rust stdlib:
- `std::os::unix::process::CommandExt` provides `exec()`
- `std::env::vars()` provides env var enumeration for sanitization
- `std::process::Command` provides env manipulation (`env()`, `env_remove()`, `env_clear()`)

**Do NOT add the `which` crate** for finding the `claude` binary. `Command::exec()` delegates to `execvp()` which searches `PATH` automatically. A missing binary produces a clear OS error (`No such file or directory`).

## Architecture Patterns

### CLI Structure Extension

Add to `src/cli.rs`:
```rust
// In Commands enum:
/// Launch Claude Code with profile environment
Run {
    /// Use a specific profile (without switching active)
    #[arg(long)]
    profile: Option<String>,
    /// Command and arguments to run (default: claude)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
},
/// Print shell environment exports for eval
Env,
```

The `trailing_var_arg = true` with `allow_hyphen_values = true` tells clap to capture everything after `--` (or just trailing args) into the `args` Vec, including flags like `--help`.

### New Files
```
src/commands/
├── run.rs           # clmux run [--profile <name>] [-- args...]
├── env.rs           # clmux env
```

### Modified Files
```
src/cli.rs           # Add Run and Env variants
src/main.rs          # Add routing for Run and Env
src/commands/mod.rs  # Add pub mod run; pub mod env;
src/commands/status.rs  # Enhance with CLAUDE_CONFIG_DIR display
```

### Pattern: Profile Resolution (shared by run, env, status)

Both `run` and `env` need to resolve the target profile to a directory path. This logic should be extracted into a shared helper:

```rust
/// Resolve which profile to use.
/// If `profile_override` is Some, validate and use it (D-05: no state change).
/// Otherwise, read active profile from state.toml.
fn resolve_profile(profile_override: Option<&str>) -> anyhow::Result<(String, std::path::PathBuf)> {
    let name = match profile_override {
        Some(name) => {
            // Validate the profile exists in config
            crate::profile::validate_profile_name(name)?;
            let cfg = crate::config::load()?;
            if !cfg.profiles.iter().any(|p| p.name == name) {
                anyhow::bail!(
                    "Profile '{}' not found. Run `clmux list` to see available profiles.",
                    name
                );
            }
            name.to_string()
        }
        None => {
            let st = crate::state::load()?;
            st.active.ok_or_else(|| {
                anyhow::anyhow!("No active profile. Run `clmux add <name>` to create one.")
            })?
        }
    };
    let dir = crate::paths::profile_dir(&name)?;
    Ok((name, dir))
}
```

This can live in a new `src/launch.rs` module or directly in `commands/run.rs` with `pub(crate)` visibility for `env.rs` to reuse.

### Pattern: Environment Sanitization (LNCH-04)

```rust
use std::process::Command;

fn build_sanitized_command(binary: &str, args: &[String], profile_dir: &Path, profile_name: &str) -> Command {
    let mut cmd = Command::new(binary);
    cmd.args(args);

    // Strip ALL CLAUDE_* and ANTHROPIC_* env vars (D-01)
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            cmd.env_remove(&key);
        }
    }

    // Inject profile-scoped vars (D-02)
    cmd.env("CLAUDE_CONFIG_DIR", profile_dir);
    cmd.env("CLMUX_PROFILE", profile_name);

    cmd
}
```

**Why prefix matching instead of a hardcoded list:** Claude Code has 140+ environment variables starting with `CLAUDE_` or `ANTHROPIC_`. New ones are added regularly. A prefix-based strip ensures forward compatibility without maintenance burden. This aligns with D-01's "broad denylist" decision.

### Pattern: exec() for Process Replacement (LNCH-03, LNCH-06)

```rust
use std::os::unix::process::CommandExt;

// After building the sanitized Command:
let err = cmd.exec();
// exec() only returns on error (it replaces the process on success)
Err(anyhow::anyhow!("Failed to execute '{}': {}", binary, err))
```

Key properties of `exec()`:
- Replaces the current process image with the new one (no child, no parent)
- PID stays the same -- signals go directly to Claude Code
- No destructors run after `exec()` succeeds (this is fine -- we have no cleanup needed)
- On failure, returns `std::io::Error` with the OS error (e.g., `No such file or directory` for missing binary)

### Pattern: Shell Export Output (LNCH-05)

```rust
pub fn execute() -> anyhow::Result<()> {
    let (name, dir) = resolve_profile(None)?;
    let dir_str = dir.display();

    // Emit unset for known Claude/Anthropic vars in current env
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            println!("unset {};", key);
        }
    }

    // Emit exports (D-07, D-09)
    println!("export CLAUDE_CONFIG_DIR=\"{}\";", dir_str);
    println!("export CLMUX_PROFILE=\"{}\";", name);

    Ok(())
}
```

**Recommendation (Claude's Discretion): Emit `unset` statements.** This matches what `clmux run` does (strips then injects) and ensures `eval "$(clmux env)"` produces the same environment as `clmux run`. Without `unset`, a user with `ANTHROPIC_API_KEY` in their shell would still have it after `eval`, breaking profile isolation.

### Pattern: Status Enhancement (LNCH-06)

Enhance existing `src/commands/status.rs` to show `CLAUDE_CONFIG_DIR` value. This helps users verify what Claude Code would see:

```rust
// After existing output:
println!("Config:  CLAUDE_CONFIG_DIR={}", dir.display());
```

**Recommendation (Claude's Discretion): Add directory content count.** Show number of items in the profile directory as a rough health indicator without parsing Claude Code internals (respects DATA-04 opaque directories):

```rust
let item_count = std::fs::read_dir(&dir)
    .map(|entries| entries.count())
    .unwrap_or(0);
if dir_exists {
    println!("Items:   {} file(s)", item_count);
}
```

### Anti-Patterns to Avoid

- **Do NOT use `Command::spawn()` + `wait()`**: This creates a child process, requiring signal forwarding. `exec()` is simpler and correct per D-06.
- **Do NOT use `env_clear()`**: This strips `PATH`, `HOME`, `TERM`, `SHELL`, etc., breaking the launched process. Only strip `CLAUDE_*` and `ANTHROPIC_*` prefixes.
- **Do NOT hardcode the `claude` binary path**: Use PATH resolution via `execvp()` (what `Command::exec()` uses internally).
- **Do NOT add `which` crate**: Unnecessary -- `exec()` handles PATH lookup.
- **Do NOT parse `--` separator manually**: clap's `trailing_var_arg = true` handles this correctly.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Binary PATH lookup | Custom PATH search | `Command::new("claude")` + `exec()` | `execvp()` does this natively; custom code handles edge cases wrong (permissions, etc.) |
| Shell quoting | Custom string escaper | Double-quote wrapping for POSIX `export` | Profile dirs from `paths::profile_dir()` only contain safe chars (alphanumeric, hyphens, underscores in profile name, `~/.clmux/profiles/` prefix). No shell metacharacters possible. |
| Argument parsing with passthrough | Manual `--` splitting | clap `trailing_var_arg` | clap handles edge cases (no args after `--`, hyphen-prefixed args) |

## Common Pitfalls

### Pitfall 1: exec() Return Value Confusion
**What goes wrong:** Developer writes `let status = cmd.exec();` and tries to use `status` as a success indicator. `exec()` never returns on success -- it only returns on error.
**Why it happens:** Confusing `exec()` with `spawn().wait()` which does return a status.
**How to avoid:** Always treat the return of `exec()` as an error: `let err = cmd.exec(); return Err(err.into());`
**Warning signs:** Unreachable code after `exec()` call.

### Pitfall 2: Forgetting to Strip CLMUX_HOME from Sanitization
**What goes wrong:** The sanitization loop strips `CLAUDE_*` and `ANTHROPIC_*` but also accidentally strips `CLMUX_HOME` or `CLMUX_PROFILE` if those get caught by a too-broad pattern.
**Why it happens:** Copy-paste error or over-broad regex.
**How to avoid:** Only strip `CLAUDE_*` and `ANTHROPIC_*` prefixes. `CLMUX_*` vars are ours and should be preserved (except `CLMUX_PROFILE` which we re-inject with the correct value).
**Warning signs:** `CLMUX_HOME` not working when `clmux run` is nested.

### Pitfall 3: clap trailing_var_arg Captures --profile
**What goes wrong:** If `--profile` is placed after `--` in the user's command, clap may capture it as a trailing arg instead of parsing it as a flag.
**Why it happens:** `trailing_var_arg` captures everything after the first non-flag argument.
**How to avoid:** Clap's `--` separator explicitly marks where flags end. `clmux run --profile work -- --help` works correctly because `--profile` comes before `--`. Document this behavior.
**Warning signs:** Tests with `--profile` after `--` behave unexpectedly.

### Pitfall 4: Profile Directory Doesn't Exist
**What goes wrong:** User runs `clmux run --profile X` but the profile directory was deleted. `exec()` launches Claude Code, which creates a fresh config dir -- but the user may not realize they're in a blank profile.
**Why it happens:** No pre-flight check on directory existence.
**How to avoid:** Check `dir.is_dir()` before exec. If missing, emit a clear error: "Profile directory for 'X' is missing at /path. Run `clmux add X` to recreate it."
**Warning signs:** Users report "I lost my auth state" after running with a specific profile.

### Pitfall 5: Shell Quoting in clmux env Output
**What goes wrong:** If the profile directory path contains spaces or special characters, the `export` statement breaks the user's shell.
**Why it happens:** Path not quoted in output.
**How to avoid:** Always double-quote the value: `export CLAUDE_CONFIG_DIR="/path/to/dir";`. In practice, clmux profile dirs never contain spaces (names are validated to `[a-zA-Z0-9_-]+`), but `CLMUX_HOME` could be set to a path with spaces.
**Warning signs:** `eval "$(clmux env)"` produces shell errors when `CLMUX_HOME` has spaces.

## Code Examples

### Complete `run` Command Implementation Pattern
```rust
// src/commands/run.rs
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::Command;
use anyhow::{Context, Result};

pub fn execute(profile_override: Option<&str>, args: &[String]) -> Result<()> {
    let (name, dir) = resolve_profile(profile_override)?;

    // Verify profile directory exists
    if !dir.is_dir() {
        anyhow::bail!(
            "Profile directory for '{}' is missing at {}. Run `clmux add {}` to recreate it.",
            name, dir.display(), name
        );
    }

    // Determine binary and args
    let (binary, cmd_args) = if args.is_empty() {
        ("claude".to_string(), vec![])
    } else {
        (args[0].clone(), args[1..].to_vec())
    };

    // Build sanitized command
    let mut cmd = Command::new(&binary);
    cmd.args(&cmd_args);

    // Strip CLAUDE_* and ANTHROPIC_* (D-01)
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            cmd.env_remove(&key);
        }
    }

    // Inject profile env (D-02)
    cmd.env("CLAUDE_CONFIG_DIR", &dir);
    cmd.env("CLMUX_PROFILE", &name);

    // exec() replaces this process (D-06)
    let err = cmd.exec();
    Err(anyhow::anyhow!(
        "Failed to execute '{}': {}. Is it installed and in your PATH?",
        binary, err
    ))
}
```

### Complete `env` Command Implementation Pattern
```rust
// src/commands/env.rs
use anyhow::Result;

pub fn execute() -> Result<()> {
    let (name, dir) = super::run::resolve_profile(None)?;

    // Unset existing Claude/Anthropic vars
    for (key, _) in std::env::vars() {
        if key.starts_with("CLAUDE_") || key.starts_with("ANTHROPIC_") {
            println!("unset {};", key);
        }
    }

    // Export profile-scoped vars (D-07, D-09)
    println!("export CLAUDE_CONFIG_DIR=\"{}\";", dir.display());
    println!("export CLMUX_PROFILE=\"{}\";", name);

    Ok(())
}
```

### Testing exec() Without Actually Exec-ing

`exec()` replaces the process, so it cannot be tested with `assert_cmd` in the normal way. Instead, test by:

1. **Integration test with a mock binary:** Create a shell script that prints its env vars, place it in a temp PATH, and verify `clmux run -- mock-binary` outputs the expected env.

```rust
#[test]
fn run_passes_claude_config_dir() {
    let home = TempDir::new().unwrap();
    let bin_dir = TempDir::new().unwrap();

    // Create a mock "claude" that prints CLAUDE_CONFIG_DIR
    let mock = bin_dir.path().join("claude");
    std::fs::write(&mock, "#!/bin/sh\necho \"CLAUDE_CONFIG_DIR=$CLAUDE_CONFIG_DIR\"").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&mock, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Set up a profile
    clmux(&home).args(["add", "test"]).assert().success();

    // Run with mock claude in PATH
    let output = clmux(&home)
        .args(["run"])
        .env("PATH", format!("{}:/usr/bin:/bin", bin_dir.path().display()))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("CLAUDE_CONFIG_DIR="));
    assert!(stdout.contains("/profiles/test"));
}
```

Note: This works because `exec()` replaces clmux with the mock script, and `assert_cmd` captures the replaced process's output.

2. **Test env sanitization separately:** Test the env var filtering logic as a unit function.

3. **Test `clmux env` output parsing:** Run `clmux env` and verify the output is valid shell syntax.

```rust
#[test]
fn env_outputs_valid_export_syntax() {
    let home = TempDir::new().unwrap();
    clmux(&home).args(["add", "work"]).assert().success();

    let output = clmux(&home)
        .args(["env"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();

    assert!(stdout.contains("export CLAUDE_CONFIG_DIR="));
    assert!(stdout.contains("export CLMUX_PROFILE="));
    // Every line should be valid shell (export or unset)
    for line in stdout.lines() {
        assert!(
            line.starts_with("export ") || line.starts_with("unset "),
            "Unexpected line in env output: {}",
            line
        );
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `Command::spawn()` + `wait()` for wrappers | `CommandExt::exec()` for process replacement | Rust 1.9+ (stable since 2016) | No child process, no signal forwarding needed, correct PID inheritance |
| Hardcoded env var lists for sanitization | Prefix-based stripping | N/A (design decision) | Forward-compatible with new Claude Code env vars |
| Manual `--` argument splitting | clap `trailing_var_arg` + `allow_hyphen_values` | clap 4.0+ | Correct handling of all edge cases |

## Open Questions

1. **`clmux env` unset behavior**
   - What we know: D-01 says strip all CLAUDE_*/ANTHROPIC_* on run. D-07 says env outputs export syntax.
   - What's unclear: Should `clmux env` also emit `unset` for currently-set CLAUDE_*/ANTHROPIC_* vars?
   - Recommendation: **Yes, emit `unset` statements.** This ensures `eval "$(clmux env)"` produces the same clean environment as `clmux run`. Without `unset`, leakage through parent shell vars breaks isolation.

2. **Error message for missing `claude` binary**
   - What we know: `exec()` returns OS error "No such file or directory" when binary not found.
   - What's unclear: How user-friendly should the error be?
   - Recommendation: Wrap the OS error: "Failed to execute 'claude': No such file or directory. Is Claude Code installed and in your PATH? Install from https://claude.ai/download"

3. **Status directory health beyond exists/missing**
   - What we know: Phase 1 status shows name, path, exists/missing.
   - What's unclear: What else to show without violating DATA-04 (opaque directories)?
   - Recommendation: Add `CLAUDE_CONFIG_DIR` value and item count (number of files/dirs in profile dir). This is useful for debugging without parsing Claude Code internals.

## Sources

### Primary (HIGH confidence)
- [Rust std::os::unix::process::CommandExt docs](https://doc.rust-lang.org/std/os/unix/process/trait.CommandExt.html) -- exec() API, behavior on success/failure
- [Rust std::process::Command docs](https://doc.rust-lang.org/std/process/struct.Command.html) -- env(), env_remove(), env_clear() behavior
- [Claude Code environment variables gist](https://gist.github.com/unkn0wncode/f87295d055dd0f0e8082358a0b5cc467) -- Complete list of 140+ CLAUDE_* and ANTHROPIC_* env vars
- [Claude Code settings documentation](https://code.claude.com/docs/en/settings) -- CLAUDE_CONFIG_DIR scope and behavior

### Secondary (MEDIUM confidence)
- [CLAUDE_CONFIG_DIR behavior issue #3833](https://github.com/anthropics/claude-code/issues/3833) -- Known limitations of directory isolation
- Project research files: `.planning/research/ARCHITECTURE.md`, `.planning/research/PITFALLS.md`, `.planning/research/FEATURES.md` -- Prior research from Phase 1

### Tertiary (LOW confidence)
- None -- all findings verified against official sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new deps needed, all stdlib
- Architecture: HIGH -- follows existing codebase patterns exactly, exec() is well-documented
- Pitfalls: HIGH -- verified against Rust docs and prior project pitfalls research
- Env var list: MEDIUM -- gist may not be exhaustive, but prefix matching makes this moot

**Research date:** 2026-04-03
**Valid until:** 2026-05-03 (stable domain, exec() and env var semantics don't change)
