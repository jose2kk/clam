# Phase 2: Launch and Shell Integration - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver process launching (`clmux run`) with environment isolation, shell env exports (`clmux env`), and verbose profile status inspection (`clmux status` enhancement). After this phase, users can launch Claude Code scoped to any profile with clean env isolation, or set up their shell session via `eval "$(clmux env)"`.

</domain>

<decisions>
## Implementation Decisions

### Env Sanitization
- **D-01:** Broad denylist — strip ALL `CLAUDE_*` and `ANTHROPIC_*` env vars before launching Claude Code to prevent leakage from parent shell
- **D-02:** After stripping, inject `CLAUDE_CONFIG_DIR` pointing to the active profile's directory AND `CLMUX_PROFILE` with the profile name

### Run Behavior
- **D-03:** Bare `clmux run` (no `--` args) defaults to launching the `claude` binary with the active profile's environment
- **D-04:** `clmux run -- <cmd> [args]` passes through to an arbitrary command (not just `claude`)
- **D-05:** `clmux run --profile <name>` uses a specific profile without switching the global active state
- **D-06:** Use Unix `exec()` to replace the clmux process — no orphaned children, correct signal forwarding. Standard for wrapper tools.

### Shell Env Output
- **D-07:** `clmux env` outputs bash/POSIX `export` syntax only — `export CLAUDE_CONFIG_DIR=... ; export CLMUX_PROFILE=...`
- **D-08:** No shell auto-detection. Fish/PowerShell users use `clmux run` instead.
- **D-09:** `clmux env` exports both `CLAUDE_CONFIG_DIR` and `CLMUX_PROFILE` (matches what `clmux run` injects)

### Status (enhancement from Phase 1)
- Phase 1's `clmux status` already shows name, path, and dir-exists. Phase 2 adds directory health context for LNCH-06.

### Claude's Discretion
- Exact list of `CLAUDE_*` and `ANTHROPIC_*` env var patterns to strip (research should identify the full set)
- Whether `clmux env` also emits `unset` statements for stripped vars
- Error message wording when `claude` binary is not found in PATH
- Status directory health detail beyond exists/missing

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Research
- `.planning/research/STACK.md` — Crate versions and Rust exec patterns
- `.planning/research/ARCHITECTURE.md` — Shell integration patterns (eval, exec, env injection)
- `.planning/research/PITFALLS.md` — exec() vs spawn, env var sanitization, signal handling
- `.planning/research/FEATURES.md` — Feature dependencies and `clmux run` / `clmux env` patterns

### Project
- `.planning/PROJECT.md` — Core value, constraints
- `.planning/REQUIREMENTS.md` — LNCH-01 through LNCH-06

### Phase 1 Context
- `.planning/phases/01-foundation-and-profile-management/01-CONTEXT.md` — Prior decisions (D-09: ~/.clmux/, D-11: 0/1 exit codes)

### Existing Code
- `src/paths.rs` — `base_dir()`, `profile_dir()`, `atomic_write()` — reuse for path resolution
- `src/state.rs` — `load()` — read active profile
- `src/config.rs` — `load()` — read profile registry (validate profile exists)
- `src/cli.rs` — CLI structure to extend with `Run` and `Env` subcommands
- `src/main.rs` — Command routing to extend
- `src/profile.rs` — `validate_name()` — reuse for `--profile` flag validation

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `paths::base_dir()` / `paths::profile_dir()` — resolves profile directory paths, respects `CLMUX_HOME`
- `state::load()` — reads active profile from `state.toml`
- `config::load()` — reads profile registry from `config.toml`
- `profile::validate_name()` — validates profile names (reuse for `--profile` flag)
- `output::error()` / `output::success()` — consistent output helpers

### Established Patterns
- Subcommand per file in `src/commands/` with `pub fn execute(...)` signature
- Error handling via `anyhow::Result` with context chaining
- `CLMUX_HOME` env var override already in `paths::base_dir()`

### Integration Points
- `src/cli.rs` — add `Run` and `Env` variants to `Commands` enum
- `src/main.rs` — add routing for new commands
- `src/commands/mod.rs` — add new module declarations

</code_context>

<specifics>
## Specific Ideas

- `exec()` means clmux disappears after launch — `clmux run` PID becomes Claude Code's PID
- The `claude` binary should be found via PATH, not hardcoded path
- `clmux env` should be safe to run multiple times (idempotent in shell RC files)
- `clmux run --profile work` should NOT modify state.toml — it's a one-shot override

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-launch-and-shell-integration*
*Context gathered: 2026-04-03*
