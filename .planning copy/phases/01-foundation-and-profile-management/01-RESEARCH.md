# Phase 1: Foundation and Profile Management - Research

**Researched:** 2026-04-03
**Domain:** Rust CLI scaffolding, profile CRUD, atomic file I/O, name validation, colored output
**Confidence:** HIGH

## Summary

Phase 1 is a greenfield Rust CLI project delivering profile CRUD commands (`add`, `list`, `use`, `current`, `remove`), data integrity foundations (atomic writes, permissions, name validation), and baseline CLI output (color, errors). The project has extensive prior research in `.planning/research/` covering stack, architecture, features, and pitfalls -- all HIGH confidence and verified against official sources.

The standard stack is well-established: clap 4 (derive) for CLI parsing, serde + toml for config, anyhow for errors, dirs for home directory, owo-colors (with `supports-colors` feature) for conditional coloring, and tempfile for atomic writes. All crates are stable, actively maintained, and version-verified.

**Primary recommendation:** Follow the architecture from `.planning/research/ARCHITECTURE.md` closely -- flat module layout, separate `config.toml` (profile registry) and `state.toml` (active pointer), commands/ directory with one file per subcommand. The atomic write pattern via `tempfile::NamedTempFile::persist()` must be used from day one for both TOML files.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** `clmux list` uses simple list format -- one profile per line, asterisk prefix for active: `* work\n  personal`
- **D-02:** `clmux status` is compact -- active profile name, path, and directory-exists check (3 lines max)
- **D-03:** `clmux current` with no active profile prints nothing to stdout and exits with code 1 (scriptable, like `git branch --show-current`)
- **D-04:** First `clmux add` auto-activates the new profile; subsequent adds do not (user must `clmux use` explicitly)
- **D-05:** Profiles are bare directories -- no metadata, no description field, no config.toml per profile. Just name -> directory.
- **D-06:** `clmux remove <name>` prompts `Remove profile <name>? [y/N]` with `--force` flag to skip confirmation
- **D-07:** Active profile cannot be removed (must switch first -- clear error message)
- **D-08:** Last remaining profile CAN be removed -- user manages their own state, empty is valid
- **D-09:** Data stored at `~/.clmux/` (not XDG) -- simple, discoverable, matches `~/.aws/` pattern
- **D-10:** No global flags for v1 -- no `--quiet`, `--verbose`, `--no-color`. Rely on `NO_COLOR` env var for color control.
- **D-11:** Exit codes are simple: 0 = success, 1 = any error

### Claude's Discretion
- Status output detail level beyond the 3-line minimum -- can include additional useful info if concise
- Error message wording and suggestion phrasing
- Internal module organization within the constraints (flat, minimal abstraction)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| PROF-01 | Create named profile with isolated directory at `~/.clmux/profiles/<name>/` | Architecture pattern: Registry + Isolated Directories; `std::fs::create_dir_all` with permissions |
| PROF-02 | List all profiles with active profile visually marked | D-01 format decision; owo-colors `if_supports_color` for TTY-aware output |
| PROF-03 | Switch active profile by name, updating `state.toml` | Pointer-based switching pattern; atomic write via tempfile |
| PROF-04 | Print active profile name to stdout for scripting (`clmux current`) | D-03: empty stdout + exit 1 when no active profile |
| PROF-05 | Remove profile with confirmation, refuse active profile | D-06/D-07/D-08 decisions; stdin read for confirmation |
| PROF-06 | Profile name validation (alphanumeric, hyphens, underscores) | Path traversal prevention from pitfalls research |
| DATA-01 | Atomic state.toml writes via tempfile + rename | tempfile::NamedTempFile::persist() pattern |
| DATA-02 | Atomic config.toml writes via tempfile + rename | Same pattern as DATA-01 |
| DATA-03 | Profile directories created with 0700 permissions | `std::os::unix::fs::DirBuilderExt::mode(0o700)` |
| DATA-04 | Profile directories treated as opaque -- never parse Claude Code internals | Design constraint: clmux creates dirs, never reads their contents |
| DEVX-03 | Colored output to TTY, plain when piped (respects `NO_COLOR`) | owo-colors with `supports-colors` feature, `if_supports_color(Stdout, ...)` |
| DEVX-04 | Clear error messages with actionable suggestions | anyhow `.context()` for error chaining, formatted user-facing messages at CLI boundary |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Prioritize readability over cleverness
- Follow existing project patterns strictly (patterns established in this phase become canonical)
- Avoid introducing unnecessary abstractions
- Always suggest how to test changes
- Never assume correctness without verification
- Prefer small, reviewable diffs

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| Rust (stable) | 1.85+ | Language | Single static binary, strong types. MSRV effectively 1.81+ (owo-colors bottleneck) |
| clap | 4.6.x | CLI parsing + subcommands | Industry standard derive macro API. `features = ["derive"]` |
| serde | 1.0.x | Serialization framework | Required by toml crate. `features = ["derive"]` |
| toml | 1.1.x | TOML config read/write | Rust ecosystem convention. First-class serde integration |
| anyhow | 1.0.x | Application error handling | `.context()` chaining, automatic `Display` for user messages |
| dirs | 6.0.x | Home directory resolution | `dirs::home_dir()` for `~/.clmux/` base path |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| owo-colors | 4.2.x | Colored terminal output | All user-facing output. Enable `supports-colors` feature for `NO_COLOR`/TTY detection |
| tempfile | 3.x | Atomic file writes | Every write to config.toml and state.toml via `NamedTempFile::persist()` |

### Dev Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| assert_cmd | 2.x | CLI integration testing -- run binary, assert stdout/stderr/exit code |
| assert_fs | 1.x | Filesystem fixtures -- temp dirs simulating `~/.clmux/` |
| predicates | 3.x | Test assertions -- `predicate::str::contains()`, regex matching |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| owo-colors | colored | `colored` allocates Strings; owo-colors is zero-cost. owo-colors is correct choice |
| anyhow | thiserror | thiserror for libraries needing typed error enums. clmux is an end-user binary; anyhow suffices |
| toml | serde_json | JSON lacks comments, not human-editable. TOML is the Rust convention |
| dirs | directories | `directories` for XDG separation. clmux only needs home dir; `dirs` is simpler |

**Installation (Cargo.toml dependencies):**
```toml
[dependencies]
clap = { version = "4.6", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
toml = "1.1"
anyhow = "1.0"
dirs = "6.0"
owo-colors = { version = "4.2", features = ["supports-colors"] }
tempfile = "3"

[dev-dependencies]
assert_cmd = "2"
assert_fs = "1"
predicates = "3"
```

**Note:** Use semver-compatible ranges (e.g., `"4.6"` not `"=4.6.0"`) to get patch updates automatically.

## Architecture Patterns

### Recommended Project Structure

```
src/
  main.rs              # Entry point: parse CLI, dispatch, handle top-level errors
  cli.rs               # clap structs: Cli, Commands enum, per-command arg structs
  commands/
    mod.rs             # Re-exports
    add.rs             # clmux add <name>
    list.rs            # clmux list
    use_cmd.rs         # clmux use <name> (use_cmd because `use` is a Rust keyword)
    current.rs         # clmux current
    remove.rs          # clmux remove <name>
    status.rs          # clmux status
  profile.rs           # Profile name validation, CRUD logic
  config.rs            # Config file I/O (config.toml -- profile registry)
  state.rs             # State file I/O (state.toml -- active profile pointer)
  paths.rs             # Path resolution: ~/.clmux, profile dirs
  output.rs            # Colored output helpers, TTY-aware formatting
tests/
  cli_tests.rs         # Integration tests using assert_cmd
```

### Pattern 1: Clap Derive Subcommands

**What:** Define CLI structure as Rust types with derive macros. Clap generates parsing, help, and validation.
**When to use:** Always -- this is the entry point.

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "clmux", about = "Manage Claude Code profiles")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new profile
    Add {
        /// Profile name
        name: String,
    },
    /// List all profiles
    List,
    /// Switch active profile
    Use {
        /// Profile name to activate
        name: String,
    },
    /// Print active profile name
    Current,
    /// Remove a profile
    Remove {
        /// Profile name to remove
        name: String,
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
    /// Show active profile status
    Status,
}
```

### Pattern 2: Atomic TOML Write

**What:** Write TOML files via temp file + atomic rename to prevent corruption.
**When to use:** Every write to config.toml and state.toml.

```rust
use std::io::Write;
use tempfile::NamedTempFile;
use anyhow::{Context, Result};

pub fn atomic_write(path: &Path, content: &str) -> Result<()> {
    let dir = path.parent().context("Invalid file path")?;
    let mut tmp = NamedTempFile::new_in(dir)
        .context("Failed to create temp file")?;
    tmp.write_all(content.as_bytes())
        .context("Failed to write temp file")?;
    tmp.persist(path)
        .map_err(|e| e.error)
        .context("Failed to persist file")?;
    Ok(())
}
```

**Critical:** `NamedTempFile::new_in(dir)` creates the temp file in the same directory as the target, ensuring `persist()` (which uses rename) works atomically on the same filesystem.

### Pattern 3: Profile Name Validation

**What:** Reject any profile name that is not `[a-zA-Z0-9_-]+` to prevent path traversal.
**When to use:** Every command that accepts a profile name as input.

```rust
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Profile name cannot be empty");
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        anyhow::bail!(
            "Profile name '{}' is invalid. Use only letters, numbers, hyphens, and underscores.",
            name
        );
    }
    // Reject reserved names
    if name == "." || name == ".." {
        anyhow::bail!("Profile name '{}' is reserved", name);
    }
    Ok(())
}
```

### Pattern 4: TTY-Aware Colored Output

**What:** Use owo-colors `if_supports_color` to emit colors only when stdout is a TTY and `NO_COLOR` is not set.
**When to use:** All user-facing output (list, status, errors).

```rust
use owo_colors::{OwoColorize, Stream::Stdout};

// Active profile in list
println!(
    "* {}",
    name.if_supports_color(Stdout, |t| t.green().bold())
);

// Error messages
eprintln!(
    "{}",
    format!("Error: Profile '{}' not found.", name)
        .if_supports_color(Stdout, |t| t.red())
);
```

### Pattern 5: Confirmation Prompt

**What:** Read y/N from stdin for destructive operations. Default is No.
**When to use:** `clmux remove` (unless `--force`).

```rust
use std::io::{self, Write};

pub fn confirm(prompt: &str) -> Result<bool> {
    print!("{} [y/N] ", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().eq_ignore_ascii_case("y"))
}
```

### Anti-Patterns to Avoid

- **Symlink-based switching:** Race conditions, not atomic, breaks if tool crashes mid-switch. Use state.toml pointer instead.
- **Monolithic config file:** Separate config.toml (registry, rarely changes) from state.toml (active pointer, changes on switch).
- **Parsing Claude Code internals:** Profile dirs are opaque. Never read `.credentials.json` or any file inside profile directories.
- **`unwrap()` on file operations:** Always use `?` with `.context()` for user-facing error messages.
- **Storing absolute paths in config:** Derive paths at runtime from `~/.clmux/` base. Never store absolute paths in TOML files.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| CLI argument parsing | Custom arg parser | clap 4 derive macros | Handles subcommands, help, validation, completions, error formatting |
| Atomic file writes | `std::fs::write()` + hope | `tempfile::NamedTempFile::persist()` | `fs::write` truncates then writes -- not atomic. Temp+rename is the correct pattern |
| Home directory resolution | `std::env::var("HOME")` | `dirs::home_dir()` | Handles platform differences, edge cases with sudo, containers |
| Color/TTY detection | `isatty()` + `std::env::var("NO_COLOR")` | owo-colors `supports-colors` feature | Handles NO_COLOR, FORCE_COLOR, CI detection, terminal type detection |
| TOML serialization | String formatting | serde + toml crate | Round-trip correctness, type safety, handles escaping |
| Error context chaining | Custom error types | anyhow `.context()` | Single-line error wrapping with user-friendly messages |

## Common Pitfalls

### Pitfall 1: State File Corruption from Concurrent Writes

**What goes wrong:** Two terminals run `clmux use` simultaneously. `fs::write()` truncates then writes -- interrupted writes leave corrupt TOML.
**Why it happens:** `fs::write()` is NOT atomic. It truncates the file first.
**How to avoid:** Use `NamedTempFile::new_in(same_dir).persist(target)` for all writes. Same-directory requirement ensures rename is atomic.
**Warning signs:** Sporadic "failed to parse state.toml" errors, active profile resetting.

### Pitfall 2: Path Traversal via Profile Names

**What goes wrong:** User runs `clmux add "../../../tmp/evil"` and creates directories outside `~/.clmux/`.
**Why it happens:** Profile name used directly in path construction without validation.
**How to avoid:** Validate names against `[a-zA-Z0-9_-]+` regex BEFORE any path construction. Reject `.`, `..`, empty strings, names containing `/`.
**Warning signs:** Directories appearing outside `~/.clmux/profiles/`.

### Pitfall 3: Forgetting Directory Permissions

**What goes wrong:** Profile directories created with default permissions (0755), allowing other users on shared machines to read credentials.
**Why it happens:** `std::fs::create_dir_all` uses umask-modified default permissions.
**How to avoid:** Use `std::os::unix::fs::DirBuilderExt::mode(0o700)` or `std::fs::set_permissions` after creation. Verify permissions in tests.
**Warning signs:** `ls -la ~/.clmux/profiles/` shows permissions other than `drwx------`.

### Pitfall 4: owo-colors Without `supports-colors` Feature

**What goes wrong:** Colors always emit regardless of TTY or `NO_COLOR`, breaking piped output.
**Why it happens:** The base owo-colors crate always colorizes. The `supports-colors` feature enables `if_supports_color()` API for conditional coloring.
**How to avoid:** Enable `features = ["supports-colors"]` in Cargo.toml. Use `if_supports_color(Stdout, ...)` instead of direct `.green()` calls.
**Warning signs:** `clmux list | grep work` contains ANSI escape codes.

### Pitfall 5: Graceless Handling of Missing/Corrupt Config

**What goes wrong:** First run of `clmux list` panics because `config.toml` doesn't exist.
**Why it happens:** Code assumes config files exist rather than handling first-run state.
**How to avoid:** If config.toml is missing, treat as empty (no profiles). If state.toml is missing, treat as no active profile. If either file is corrupt, print a clear error and suggest deleting the file.
**Warning signs:** Panics on fresh install, unhelpful "file not found" errors.

### Pitfall 6: `use` is a Rust Keyword

**What goes wrong:** Cannot name a file `use.rs` or a function `use()` in Rust.
**Why it happens:** `use` is a reserved keyword.
**How to avoid:** Name the command handler file `use_cmd.rs` and the function `use_profile()` or `switch()`. The clap subcommand name (`Use` variant) works fine since it's a type name.
**Warning signs:** Compiler errors about unexpected keyword.

## Code Examples

### Main Entry Point Pattern

```rust
// src/main.rs
use anyhow::Result;
use clap::Parser;

mod cli;
mod commands;
mod config;
mod output;
mod paths;
mod profile;
mod state;

fn main() {
    if let Err(e) = run() {
        // Print error with context chain
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = cli::Cli::parse();
    match cli.command {
        cli::Commands::Add { name } => commands::add::execute(&name),
        cli::Commands::List => commands::list::execute(),
        cli::Commands::Use { name } => commands::use_cmd::execute(&name),
        cli::Commands::Current => commands::current::execute(),
        cli::Commands::Remove { name, force } => commands::remove::execute(&name, force),
        cli::Commands::Status => commands::status::execute(),
    }
}
```

### Config File Structure

```toml
# ~/.clmux/config.toml
[[profiles]]
name = "personal"

[[profiles]]
name = "work"
```

```rust
// src/config.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
}
```

### State File Structure

```toml
# ~/.clmux/state.toml
active = "work"
```

```rust
// src/state.rs
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
pub struct State {
    pub active: Option<String>,
}
```

### Integration Test Pattern

```rust
// tests/cli_tests.rs
use assert_cmd::Command;
use assert_fs::TempDir;
use predicates::prelude::*;

fn clmux_cmd(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("clmux").unwrap();
    cmd.env("CLMUX_HOME", home.path());
    cmd
}

#[test]
fn test_add_creates_profile() {
    let home = TempDir::new().unwrap();
    clmux_cmd(&home)
        .args(["add", "work"])
        .assert()
        .success();

    assert!(home.path().join("profiles/work").exists());
}

#[test]
fn test_add_rejects_invalid_name() {
    let home = TempDir::new().unwrap();
    clmux_cmd(&home)
        .args(["add", "../evil"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid"));
}
```

**Key test pattern:** Override `CLMUX_HOME` env var to point at a temp directory. This isolates tests from the real filesystem and enables parallel test execution.

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| structopt for CLI | clap 4 derive API | 2022 (clap 4.0) | structopt deprecated, merged into clap |
| `colored` crate | owo-colors | 2023+ | Zero-allocation, better NO_COLOR support |
| Manual error types | anyhow for apps, thiserror for libs | Stable since 2020 | Simpler error handling in CLI apps |
| `config` crate for TOML | Direct serde + toml | Always for simple tools | `config` is overkill for single-file TOML |

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain (rustc + cargo) | Everything | NOT INSTALLED | -- | Must install via `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |

**Missing dependencies with no fallback:**
- **Rust toolchain:** The project cannot build without rustc and cargo. The planner MUST include a Wave 0 / setup task that installs Rust via rustup. Target: stable channel, minimum 1.81+ (owo-colors MSRV).

**Missing dependencies with fallback:**
- None

## Open Questions

1. **`CLMUX_HOME` env var for testing**
   - What we know: Tests need to override `~/.clmux/` to use temp directories. The architecture research suggests `CLMUX_HOME` env var.
   - What's unclear: Whether to support `CLMUX_HOME` as a first-class feature (documented, for power users) or only as an internal testing mechanism.
   - Recommendation: Support it as a first-class feature (matches `AWS_CONFIG_FILE` pattern). It's 3 lines of code in `paths.rs` and enables testing, CI, and advanced use cases.

2. **Confirmation prompt behavior when stdin is not a TTY**
   - What we know: `clmux remove work` prompts for confirmation. If piped, stdin is not interactive.
   - What's unclear: Should non-TTY stdin default to "no" (safe) or require `--force`?
   - Recommendation: Default to "no" when stdin is not a TTY. Print error: "Cannot prompt for confirmation. Use --force to skip." This prevents accidental deletion in scripts.

3. **First `clmux add` auto-activation (D-04) with no state.toml**
   - What we know: D-04 says first add auto-activates. But "first" means first profile ever, not first profile in a session.
   - What's unclear: How to detect "first" -- check if state.toml has no active profile, or check if config.toml has zero profiles before this add?
   - Recommendation: Auto-activate if config.toml has zero existing profiles at the time of `add`. This is the simplest, most predictable check.

## Sources

### Primary (HIGH confidence)
- `.planning/research/STACK.md` -- Crate versions verified against docs.rs and crates.io
- `.planning/research/ARCHITECTURE.md` -- Project structure, component responsibilities, data flow
- `.planning/research/PITFALLS.md` -- Domain pitfalls with prevention strategies
- `.planning/research/FEATURES.md` -- Feature landscape and MVP phasing
- [docs.rs/owo-colors](https://docs.rs/owo-colors) -- `supports-colors` feature, `if_supports_color` API
- [docs.rs/tempfile NamedTempFile](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html) -- `persist()` method, atomic write semantics
- [clap derive tutorial](https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html) -- Subcommand derive pattern

### Secondary (MEDIUM confidence)
- [Rain's Rust CLI Recommendations](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) -- owo-colors recommendation for color management
- [owo-colors GitHub](https://github.com/owo-colors/owo-colors) -- Feature flag definitions

### Tertiary (LOW confidence)
- None -- all findings verified against primary sources

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all crates verified against docs.rs, versions confirmed in prior research
- Architecture: HIGH -- patterns drawn from established CLI tools (aws, kubectl, gh), documented in project research
- Pitfalls: HIGH -- domain-specific pitfalls documented with prevention strategies, cross-referenced with official Rust docs
- Environment: HIGH -- Rust toolchain absence confirmed by direct check

**Research date:** 2026-04-03
**Valid until:** 2026-05-03 (stable domain, no fast-moving dependencies)
