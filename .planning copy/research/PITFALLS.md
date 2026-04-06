# Pitfalls Research

**Domain:** Rust CLI tool for Claude Code profile/account isolation via config directory management
**Researched:** 2026-04-02
**Confidence:** HIGH (domain-specific, verified against official docs and known issues)

## Critical Pitfalls

### Pitfall 1: CLAUDE_CONFIG_DIR Does Not Fully Isolate All State

**What goes wrong:**
Developers assume `CLAUDE_CONFIG_DIR` redirects *all* Claude Code state into the specified directory. It does not. Project-local `.claude/settings.local.json` files are still written per-workspace regardless of the env var. The `.claude/ide/` directory is created by VS Code integration independently. If clmux users expect "switching profiles means completely separate Claude Code worlds," they will encounter cross-contamination through these local files.

**Why it happens:**
The `CLAUDE_CONFIG_DIR` variable controls *global* state (credentials, session history, settings.json, plugins) but Claude Code's project-scoped settings are intentionally per-workspace. This is by design, not a bug -- but it violates the mental model of "one env var = full isolation."

**How to avoid:**
1. Document clearly what `CLAUDE_CONFIG_DIR` controls and what it does not.
2. Do NOT try to redirect project-local `.claude/` directories -- that path is per-repo, not per-account.
3. Verify the exact list of files that move with `CLAUDE_CONFIG_DIR` by testing, and document them in clmux's help output.
4. Consider a `clmux doctor` command that checks for state leakage.

**Warning signs:**
- Users report "my work account settings appear in my personal profile"
- `.claude/settings.local.json` in a project contains tool permissions from the wrong account
- IDE integration creates files outside the managed profile directory

**Phase to address:**
Phase 1 (Core profile management). This is foundational -- get the isolation boundary right before building anything else. Must test with real Claude Code to verify what moves and what stays.

---

### Pitfall 2: State File Corruption from Concurrent Profile Switches

**What goes wrong:**
Two terminal sessions both write to `~/.clmux/state.toml` simultaneously (e.g., `clmux use work` in one tab while another tab runs `clmux use personal`). Without atomic writes, the file can be partially written, leaving a truncated or corrupted TOML file. Next invocation of clmux fails to parse state and either crashes or silently uses no profile.

**Why it happens:**
Developers write config files using `fs::write()` which is not atomic -- it truncates the file then writes. If the process is interrupted (Ctrl+C, power loss) or another process reads during the write, the file can be in an invalid state. This is a classic TOCTOU (Time-of-check Time-of-use) problem.

**How to avoid:**
1. Use the atomic write pattern: write to a temp file in the same directory, `fsync()`, then `rename()`. On Unix, `rename()` within the same filesystem is atomic.
2. Use the `tempfile` crate's `NamedTempFile::persist()` method, which implements this pattern.
3. Keep `state.toml` minimal (just the active profile name) so writes are fast and the window for corruption is tiny.
4. On read, if parsing fails, fall back to "no active profile" with a clear error rather than panicking.

**Warning signs:**
- Sporadic "failed to parse state.toml" errors
- Users report "my active profile keeps resetting"
- CI/automation running parallel clmux commands sees intermittent failures

**Phase to address:**
Phase 1 (Core profile management). Atomic writes should be the default from day one. This is a 10-line pattern that prevents a class of hard-to-reproduce bugs.

---

### Pitfall 3: Child Process Signal Handling -- Orphaned Claude Code Processes

**What goes wrong:**
When `clmux run` spawns Claude Code as a child process, pressing Ctrl+C sends SIGINT to the entire process group. If clmux installs its own signal handler (e.g., for cleanup), the default SIGINT behavior is replaced, and Claude Code may not receive the signal. Conversely, if clmux does NOT handle signals, it may exit while Claude Code continues running as an orphan process, still holding auth state and potentially conflicting with subsequent profile switches.

**Why it happens:**
Rust's `std::process::Command` creates a child in the same process group by default. SIGINT is sent to the entire foreground process group, so typically both parent and child get it. Problems arise when:
- clmux sets a custom SIGINT handler (e.g., via `ctrlc` crate) that doesn't propagate the signal
- Claude Code is spawned as a background process (different process group)
- clmux exits before Claude Code finishes, losing the ability to clean up

**How to avoid:**
1. For `clmux run`: exec into Claude Code (replace the clmux process) rather than spawning a child. Use `std::os::unix::process::CommandExt::exec()`. This is the simplest and most correct approach -- clmux sets up env vars, then becomes Claude Code.
2. If exec is not viable (e.g., need post-run cleanup): use `Command::spawn()` + `child.wait()`, and install a signal handler that forwards signals to the child before exiting.
3. Never use `child.kill()` (SIGKILL) as a first response -- send SIGTERM first, wait briefly, then escalate.

**Warning signs:**
- `ps aux | grep claude` shows processes from a previous profile after switching
- Users report "Claude Code seems to be using the wrong account"
- Resource leaks (open file handles, socket connections) after clmux exits

**Phase to address:**
Phase 2 (Process launching / `clmux run`). The exec approach should be the default. If clmux ever needs to wrap (not replace) the child, signal forwarding must be implemented at that point.

---

### Pitfall 4: Environment Variable Inheritance Leaking Between Profiles

**What goes wrong:**
When `clmux run` launches Claude Code, it sets `CLAUDE_CONFIG_DIR` to the profile directory. But the child process inherits ALL other environment variables from the parent shell. If the user previously set `CLAUDE_CONFIG_DIR` manually, or has other Claude-related env vars (`ANTHROPIC_API_KEY`, `CLAUDE_CODE_USE_BEDROCK`, etc.), these leak into the clmux-managed session and override profile-specific settings.

**Why it happens:**
Rust's `std::process::Command` inherits the parent environment by default. The `Command::env()` method adds/overrides specific variables but does not clear others. Developers often set only the variables they know about, missing new ones added in future Claude Code releases.

**How to avoid:**
1. When building the child command, explicitly remove known Claude-related env vars using `Command::env_remove()` for: `CLAUDE_CONFIG_DIR`, `ANTHROPIC_API_KEY`, `CLAUDE_API_KEY`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX`, `AWS_PROFILE`, `AWS_REGION` (when profile-specific).
2. Consider a "strict mode" that uses `env_clear()` and only passes through a known-safe allowlist (`PATH`, `HOME`, `TERM`, `SHELL`, `USER`, `LANG`, `TZ`, `DISPLAY`, `SSH_AUTH_SOCK`).
3. Document the env var sanitization behavior so users understand what clmux strips.
4. Do NOT clear all env vars by default -- tools depend on `PATH`, `HOME`, editor vars, etc. Clearing too aggressively breaks the user's shell.

**Warning signs:**
- `clmux run work` uses credentials from the personal profile (because `ANTHROPIC_API_KEY` was set globally)
- Users with `CLAUDE_CODE_USE_BEDROCK=1` in their shell can't use a direct-API profile
- Env vars set in `.zshrc` override profile-specific values

**Phase to address:**
Phase 2 (Process launching). Must be designed alongside `clmux run` and `clmux env`. Consider letting profiles specify their own env var overrides in profile config.

---

### Pitfall 5: Hardcoded Path Assumptions -- `~` vs `$HOME` vs Actual Paths

**What goes wrong:**
The tool stores paths like `~/.clmux/profiles/work/` in config files. On macOS, `$HOME` is `/Users/username`. On Linux, it's `/home/username`. The `~` character is a shell expansion, not a filesystem path -- Rust's `std::fs` functions do not expand `~`. If the user's `$HOME` changes (container, sudo, CI), stored absolute paths point to nonexistent directories.

**Why it happens:**
Developers test on their own machine where `~` and `$HOME` and the actual path all agree. Edge cases appear in: Docker containers with different `$HOME`, `sudo` invocations where `$HOME` is `/root`, NFS-mounted homes, CI systems with synthetic users.

**How to avoid:**
1. Always resolve `~` to `$HOME` at runtime using `dirs::home_dir()` or `std::env::var("HOME")`, never store `~` in config files.
2. Store paths relative to a known base (e.g., profiles are always at `$CLMUX_HOME/profiles/<name>/`) rather than absolute paths. Resolve at runtime.
3. Define `CLMUX_HOME` env var (defaulting to `$HOME/.clmux`) so advanced users can relocate the entire clmux state directory.
4. Validate that profile directories exist on every operation, with clear errors when they don't.

**Warning signs:**
- "directory not found" errors that work fine when the user runs the exact same command manually
- Tests pass locally but fail in CI
- sudo clmux behaves differently from regular clmux

**Phase to address:**
Phase 1 (Core profile management). Path resolution is foundational -- get it right in the first data structure that stores profile locations.

---

### Pitfall 6: Breaking Changes in Claude Code's Directory Layout

**What goes wrong:**
Claude Code is actively developed. The files inside `~/.claude/` (now controlled by `CLAUDE_CONFIG_DIR`) can change between versions: new files, renamed files, changed formats. If clmux makes assumptions about the internal structure of the Claude config directory (e.g., "credentials are always in `.credentials.json`"), a Claude Code update can break clmux's profile inspection or health-check features.

**Why it happens:**
Claude Code's internal directory structure is not a stable API. The `CLAUDE_CONFIG_DIR` env var is documented, but the contents of that directory are implementation details. Tools that depend on those details are fragile.

**How to avoid:**
1. clmux should treat profile directories as opaque. It creates them, sets `CLAUDE_CONFIG_DIR` to point at them, and never reads or interprets their contents.
2. For `clmux status` / `clmux inspect`: show directory existence and size, not parsed file contents.
3. If profile health checks are needed, run `claude --version` or a similar probe rather than parsing config files.
4. Pin the minimum supported Claude Code version and document it.

**Warning signs:**
- clmux breaks after a Claude Code update without any clmux changes
- `clmux inspect` shows stale or incorrect information
- Users report "it worked yesterday" after auto-updating Claude Code

**Phase to address:**
Phase 1 (Core profile management). The opaque-directory principle must be a design decision from day one, not bolted on later.

---

## Technical Debt Patterns

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Using `fs::write()` instead of atomic writes | Simpler code, fewer deps | Corrupted state on crash or concurrent access | Never -- atomic writes are trivial with `tempfile` crate |
| Storing absolute paths in config | Works on dev machine | Breaks in containers, CI, sudo, relocated HOME | Never -- always resolve at runtime from relative base |
| Parsing Claude Code's internal files | Richer `clmux inspect` output | Breaks on every Claude Code update | Never in v1 -- treat profile dirs as opaque |
| `unwrap()` on file operations | Faster prototyping | Panic on missing dir, permission denied, disk full | MVP only, replace with proper errors before release |
| Skipping `--` separator in clap subcommands | Fewer edge cases to handle | Users can't pass flags that collide with clmux's | MVP only -- add passthrough support before v1 |

## Integration Gotchas

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Claude Code `CLAUDE_CONFIG_DIR` | Assuming it isolates everything | It only isolates global state; project-local `.claude/` dirs are per-workspace, not per-profile |
| Claude Code auth | Trying to manage credentials directly | Let Claude Code handle its own auth; clmux just points it at a directory |
| Shell integration (`eval $(clmux env)`) | Setting env vars that persist beyond the intended scope | Use subshell patterns or document that users should use `clmux run` for clean isolation |
| `CLAUDE_CODE_PLUGIN_CACHE_DIR` | Not setting it alongside `CLAUDE_CONFIG_DIR` | Each profile may need its own plugin cache if plugins differ between accounts |
| macOS Keychain | Assuming credentials are in the config dir | Claude Code may store OAuth tokens in macOS Keychain, bypassing directory isolation entirely |

## Performance Traps

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Scanning profile dirs for status info | Slow `clmux list` with many profiles | Only read state.toml for active profile; lazy-load details | 20+ profiles with large session histories |
| Synchronous file I/O for every command | Noticeable latency on NFS/slow disks | Read only what's needed; state.toml should be < 1KB | Network-mounted home directories |
| Recursive directory size calculation | `clmux inspect` hangs on large profiles | Show file count, not total size; or make it async/optional | Profiles with months of session transcripts |

## Security Mistakes

| Mistake | Risk | Prevention |
|---------|------|------------|
| Profile directories with world-readable permissions | Other users on shared machine can read credentials | Create profile dirs with `0700` permissions; verify on every access |
| Symlink following in profile paths | Symlink attack could redirect clmux to read/write arbitrary directories | Use `canonicalize()` on paths before operations; reject symlinks in profile base dir |
| Storing API keys in clmux's own config | Credential duplication and exposure | Never store credentials -- clmux manages directories, Claude Code manages auth |
| Not sanitizing profile names | Path traversal via `../` in profile names | Validate names: alphanumeric + hyphens + underscores only; reject `/`, `..`, NUL |
| Leaking env vars in error messages | API keys or tokens appear in logs/output | Redact env vars in error output; only show variable names, not values |

## UX Pitfalls

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Silent profile switch with no confirmation | User runs commands against wrong account without realizing | Print active profile name on switch; consider showing it in `clmux run` prompt |
| No indication of active profile in shell | User forgets which profile is active | `clmux env` should export `CLMUX_PROFILE` for shell prompt integration; document PS1 customization |
| Destructive `clmux remove` without confirmation | Accidental deletion of profile with auth state | Require `--force` flag or interactive confirmation; consider `clmux archive` as soft-delete |
| Cryptic errors when Claude Code is not installed | User installs clmux first, then can't figure out what's wrong | Check for `claude` binary on first run and in `clmux doctor`; provide install instructions |
| `clmux use` changes global state, affects other terminals | User switches profile in one terminal, breaks running session in another | Document this clearly; consider per-terminal env var approach vs global state file |

## "Looks Done But Isn't" Checklist

- [ ] **Profile creation:** Often missing directory permission setting -- verify dirs are created with `0700`
- [ ] **Profile switching:** Often missing validation that target profile dir exists -- verify graceful error on deleted/moved profile dir
- [ ] **`clmux run`:** Often missing env var sanitization -- verify Claude-related env vars from parent shell are stripped
- [ ] **`clmux remove`:** Often missing check for active profile -- verify you can't delete the currently active profile without switching first
- [ ] **Config parsing:** Often missing graceful handling of corrupt TOML -- verify clmux recovers from truncated/empty state.toml
- [ ] **Cross-platform:** Often missing Linux path differences -- verify `~/.clmux` resolution works when `$HOME` is not `/Users/...`
- [ ] **Signal handling:** Often missing in `clmux run` -- verify Ctrl+C propagates cleanly to Claude Code child process
- [ ] **Profile names:** Often missing input validation -- verify names like `../../../etc/passwd`, empty string, and very long names are rejected

## Recovery Strategies

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Corrupted state.toml | LOW | Delete state.toml; clmux starts with no active profile; user runs `clmux use <name>` |
| Deleted profile directory | MEDIUM | Re-run `clmux add <name>`; user must re-authenticate in Claude Code |
| Orphaned Claude Code process | LOW | `pkill -f "claude.*CLAUDE_CONFIG_DIR"` or just `killall claude`; no data loss |
| Wrong profile used for sensitive operation | HIGH | Cannot undo -- audit what was done; this is why active-profile visibility matters |
| Permission escalation via symlink | HIGH | Audit affected directories; re-create profile dirs; review what was exposed |

## Pitfall-to-Phase Mapping

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| CLAUDE_CONFIG_DIR partial isolation | Phase 1 (Core) | Integration test: create profile, run Claude Code, verify no cross-contamination |
| State file corruption | Phase 1 (Core) | Unit test: concurrent writes to state.toml; verify atomic write pattern used |
| Orphaned child processes | Phase 2 (Run/Exec) | Manual test: Ctrl+C during `clmux run`; verify no orphan processes remain |
| Env var leakage | Phase 2 (Run/Exec) | Integration test: set `ANTHROPIC_API_KEY` in shell, verify `clmux run` strips it |
| Hardcoded path assumptions | Phase 1 (Core) | CI test: run with custom `$HOME`; verify all paths resolve correctly |
| Claude Code breaking changes | Phase 1 (Core) | Design review: verify no code reads contents of profile directories |
| Profile name path traversal | Phase 1 (Core) | Unit test: attempt `clmux add "../../../tmp/evil"`; verify rejection |
| Global state affects other terminals | Phase 2 (Run/Exec) | Document behavior; consider env-var-only mode as alternative to state file |

## Sources

- [Claude Code Settings Documentation](https://code.claude.com/docs/en/settings) -- official config scopes and file locations
- [Claude Code Environment Variables](https://code.claude.com/docs/en/env-vars) -- `CLAUDE_CONFIG_DIR` behavior and related vars
- [GitHub Issue #3833: CLAUDE_CONFIG_DIR behavior unclear](https://github.com/anthropics/claude-code/issues/3833) -- known limitations of directory isolation
- [Command in std::process - Rust](https://doc.rust-lang.org/std/process/struct.Command.html) -- env inheritance behavior
- [Command/environment variable interaction is confusing (Issue #28975)](https://github.com/rust-lang/rust/issues/28975) -- lazy env capture pitfall
- [Signal handling - Command Line Applications in Rust](https://rust-cli.github.io/book/in-depth/signals.html) -- signal propagation patterns
- [tempfile crate - NamedTempFile](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html) -- atomic write pattern
- [atomic_write_file crate](https://docs.rs/atomic-write-file) -- alternative atomic write approach
- [CVE-2022-21658: Rust std::fs::remove_dir_all symlink following](https://groups.google.com/g/rustlang-security-announcements/c/R1fZFDhnJVQ) -- TOCTOU in Rust stdlib
- [Rust cross-compilation with musl](https://blog.crafteo.io/2024/02/29/my-rust-cross-compilation-journey/) -- static binary distribution challenges
- [musl malloc performance](https://raniz.blog/2025-02-06_rust-musl-malloc/) -- allocator performance with musl

---
*Pitfalls research for: Rust CLI profile manager for Claude Code*
*Researched: 2026-04-02*
