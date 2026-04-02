# Project Research Summary

**Project:** clmux
**Domain:** Rust CLI tool for Claude Code profile/account isolation
**Researched:** 2026-04-02
**Confidence:** HIGH

## Executive Summary

clmux is a profile manager for Claude Code, following the same patterns as `aws configure`, `kubectl config use-context`, and `gcloud config configurations`. The core mechanism is simple: each profile gets its own directory under `~/.clmux/profiles/<name>/`, and clmux sets `CLAUDE_CONFIG_DIR` to point Claude Code at the right one. The Rust CLI ecosystem is mature and well-suited for this -- clap for arg parsing, serde+toml for config, anyhow for errors, and dirs for path resolution. The entire dependency tree is small, well-maintained, and produces a single static binary with no runtime dependencies.

The recommended approach is a layered build: foundation (paths, config, state I/O) first, then CRUD commands (add, list, use, remove), then process launching (`clmux run` with env injection), then polish (completions, JSON output, prompt helpers). This order respects component dependencies and lets each phase produce a testable increment. The architecture is intentionally flat -- one file per concern, no trait hierarchies, no async runtime.

The primary risk is the `CLAUDE_CONFIG_DIR` isolation boundary. It redirects global state (auth, settings, history) but NOT project-local `.claude/` directories, which are always per-workspace. This is correct behavior but will confuse users who expect total isolation. Secondary risks include state file corruption from concurrent writes (solved with atomic writes from day one), orphaned child processes when `clmux run` is interrupted (solved by using Unix `exec()` instead of spawn), and environment variable leakage from the parent shell into launched Claude Code sessions. All of these are well-understood problems with known solutions that must be implemented in the right phase.

## Key Findings

### Recommended Stack

The stack is Rust-native, minimal, and battle-tested. Every dependency is actively maintained and widely used in the Rust CLI ecosystem. The effective MSRV is Rust 1.81+ (bottleneck: owo-colors). No async runtime is needed -- all operations are synchronous filesystem I/O.

**Core technologies:**
- **clap 4.6** (derive): CLI parsing and subcommand dispatch -- industry standard, free completions and man pages via companion crates
- **serde 1.0 + toml 1.1**: Config serialization -- Rust convention, human-readable, no YAML footguns
- **anyhow 1.0**: Error handling -- context chaining for user-facing messages, no need for typed error enums in v1
- **dirs 6.0**: Home directory resolution -- correct on macOS and Linux, tiny footprint
- **owo-colors 4.2**: Terminal coloring -- zero-allocation, respects `NO_COLOR`
- **tempfile** (add to deps): Atomic file writes via `NamedTempFile::persist()` -- critical for state.toml integrity

**Dev/test:** assert_cmd + assert_fs + predicates for CLI integration tests. cargo-dist for release automation.

### Expected Features

**Must have (table stakes):**
- `clmux add <name>` -- create profile with isolated directory
- `clmux list` -- show all profiles with active marker
- `clmux use <name>` -- switch active profile (pointer in state.toml)
- `clmux current` -- print active profile name (scriptable)
- `clmux remove <name>` -- delete profile with confirmation
- `clmux run [-- args]` -- launch Claude Code with `CLAUDE_CONFIG_DIR` set
- `clmux env` -- emit shell exports for `eval` integration
- `clmux status` -- verbose active profile info
- Clear error messages on unknown profiles, missing state

**Should have (differentiators over shell aliases):**
- `clmux show <name>` -- inspect any profile's paths and state
- `CLMUX_PROFILE` env var override -- per-terminal profile without global switch
- `clmux which` -- debug profile resolution order
- `clmux prompt` -- shell prompt helper for PS1 integration
- `clmux completions` -- bash/zsh/fish (nearly free from clap)
- `--json` flag on list/status/show

**Defer (v2+):**
- `clmux clone` -- copy profile (nice-to-have, not essential)
- Shell completions can ship late since clap makes them trivial
- Windows support

### Architecture Approach

Flat, layered architecture with clear component boundaries. CLI layer (clap derive) dispatches to thin command handlers, which call into a Profile Manager for business logic. Config and state are separate files (config.toml for profile registry, state.toml for active pointer) because they change at different frequencies. All path resolution is centralized in a dedicated module to prevent path construction from leaking everywhere. Profile directories are treated as opaque -- clmux creates them and points Claude Code at them but never reads their contents.

**Major components:**
1. **CLI Layer** (cli.rs, main.rs) -- clap parsing, dispatch, output formatting
2. **Command Handlers** (commands/*.rs) -- one file per subcommand, thin orchestration
3. **Profile Manager** (profile.rs) -- CRUD logic, validation, active profile resolution
4. **Config/State Stores** (config.rs, state.rs) -- separate TOML files for separate concerns
5. **Path Resolution** (paths.rs) -- centralized `CLMUX_HOME` resolution, profile dir paths
6. **Shell Integration** (shell.rs) -- env var generation, process exec for `clmux run`

### Critical Pitfalls

1. **CLAUDE_CONFIG_DIR partial isolation** -- It only redirects global state, not project-local `.claude/` dirs. Treat profile dirs as opaque; document the boundary clearly. Must verify with real Claude Code in Phase 1.
2. **State file corruption** -- Concurrent `clmux use` from multiple terminals can corrupt state.toml. Use atomic writes (tempfile crate, write-then-rename) from day one. Non-negotiable.
3. **Orphaned child processes** -- `clmux run` must use Unix `exec()` to replace the process, not spawn+wait. This eliminates signal handling complexity entirely.
4. **Environment variable leakage** -- Parent shell env vars (`ANTHROPIC_API_KEY`, `CLAUDE_CODE_USE_BEDROCK`) leak into launched sessions. Strip known Claude-related vars in `clmux run`.
5. **Path resolution assumptions** -- Never store `~` in config files, never hardcode paths. Resolve from `CLMUX_HOME` (default `$HOME/.clmux`) at runtime. Validate profile dirs exist on every operation.
6. **Profile name injection** -- Names like `../../../tmp/evil` enable path traversal. Validate: alphanumeric + hyphens + underscores only.

## Implications for Roadmap

Based on research, suggested phase structure:

### Phase 1: Foundation and Profile CRUD

**Rationale:** Everything depends on path resolution, config/state I/O, and the ability to create profiles. This phase establishes the data model and proves the directory isolation approach works. All three research streams (architecture, features, pitfalls) agree this comes first.
**Delivers:** Working `clmux add`, `clmux list`, `clmux use`, `clmux current`, `clmux remove`, `clmux status`
**Addresses:** All table-stakes CRUD features, state management via state.toml
**Avoids:** State file corruption (atomic writes), path traversal (name validation), hardcoded paths (runtime resolution), opaque profile dirs (no internal file parsing)
**Key implementation details:**
- paths.rs, error.rs, config.rs, state.rs as foundation modules
- profile.rs for CRUD logic
- Atomic writes for state.toml from the start (add `tempfile` to deps)
- Profile name validation (reject special chars, `..`, empty)
- Directory permissions set to 0700 on creation

### Phase 2: Process Launching and Shell Integration

**Rationale:** This phase delivers the core value proposition -- actually launching Claude Code with profile isolation. It depends on Phase 1's profile resolution being solid. The `clmux run` command is the feature that makes clmux worth using over manual aliases.
**Delivers:** `clmux run [-- args]`, `clmux env`, env var sanitization
**Addresses:** Launch and integration features from FEATURES.md
**Avoids:** Orphaned processes (use exec()), env var leakage (strip Claude-related vars), global state affecting other terminals (document behavior)
**Key implementation details:**
- shell.rs module for env construction and process exec
- Use `CommandExt::exec()` on Unix to replace process (no child management needed)
- Strip `CLAUDE_CONFIG_DIR`, `ANTHROPIC_API_KEY`, `CLAUDE_API_KEY`, `CLAUDE_CODE_USE_BEDROCK`, `CLAUDE_CODE_USE_VERTEX` from inherited env
- `clmux env` outputs `export CLAUDE_CONFIG_DIR=...` for eval pattern

### Phase 3: Developer Experience Polish

**Rationale:** With core functionality working, this phase adds the features that make clmux feel like a polished tool rather than a prototype. These features are low-complexity and high-value for daily use.
**Delivers:** `clmux show <name>`, `clmux completions`, `CLMUX_PROFILE` env override, `clmux which`, `clmux prompt`, `--json` flag
**Addresses:** All differentiator features from FEATURES.md
**Key implementation details:**
- `CLMUX_PROFILE` env var modifies profile resolution in all commands
- `clmux which` shows resolution chain (env var -> state.toml -> none)
- `clmux prompt --format "[%s]"` for PS1 integration
- `--json` flag using serde_json (add to deps in this phase)
- clap_complete for shell completions

### Phase 4: Distribution and Release

**Rationale:** Binary distribution is the last mile. cargo-dist handles the GitHub Actions CI pipeline for cross-platform builds. This phase also includes documentation and install instructions.
**Delivers:** GitHub Releases with static binaries (macOS x86_64 + aarch64, Linux x86_64-musl), install docs, README
**Key implementation details:**
- `cargo dist init` to scaffold release CI
- Linux builds use musl target for static linking
- macOS builds are portable by default (system dylibs always available)

### Phase Ordering Rationale

- Phase 1 before Phase 2: Cannot launch with a profile if profiles do not exist. Foundation modules (paths, config, state) are imported by every subsequent component.
- Phase 2 before Phase 3: The `run` command is the primary user interaction. Polish features enhance it but are not prerequisites.
- Phase 3 before Phase 4: Ship a complete feature set before worrying about distribution packaging.
- Pitfall prevention is front-loaded: atomic writes, path validation, and opaque-directory principles are all Phase 1 concerns. Signal handling and env sanitization are Phase 2. This avoids retrofitting safety into existing code.

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 2:** Process launching and signal handling on macOS vs Linux may have subtle differences. The `exec()` approach is well-documented but the env var sanitization list may need updating as Claude Code adds new env vars. Test against real Claude Code.
- **Phase 2:** The `eval $(clmux env)` pattern has shell-specific edge cases (fish uses `set -x`, not `export`). Research shell-specific output formats.

Phases with standard patterns (skip research-phase):
- **Phase 1:** CRUD operations, TOML config I/O, clap subcommands -- all thoroughly documented with established Rust patterns.
- **Phase 3:** Shell completions, JSON output, env var overrides -- all well-supported by clap ecosystem and serde.
- **Phase 4:** cargo-dist handles release automation with minimal configuration.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crate versions verified on docs.rs. Dependencies are mature and widely used. No experimental or niche choices. |
| Features | HIGH | Feature set derived from established CLI tools (aws, kubectl, gh, gcloud). Dependency graph is clear. |
| Architecture | HIGH | Flat layered architecture matches patterns used by major Rust CLI tools. Component boundaries are well-defined. |
| Pitfalls | HIGH | Pitfalls are domain-specific and verified against official Claude Code docs and known GitHub issues. Prevention strategies are concrete. |

**Overall confidence:** HIGH

### Gaps to Address

- **macOS Keychain interaction:** Claude Code may store OAuth tokens in macOS Keychain, bypassing directory isolation entirely. This needs testing with real Claude Code to determine if it is an issue. If so, it may require documentation ("re-authenticate after switching profiles on macOS") rather than a code fix.
- **`CLAUDE_CONFIG_DIR` exact file list:** The precise set of files that move with this env var is not fully documented. Phase 1 should include an integration test that creates a profile, runs Claude Code, and inventories what ends up in the profile directory vs the workspace `.claude/` directory.
- **`CLAUDE_CODE_PLUGIN_CACHE_DIR`:** May need to be set alongside `CLAUDE_CONFIG_DIR` if profiles use different plugins. Low priority but worth noting.
- **Fish shell compatibility:** `clmux env` needs to detect or be told the shell type to output the correct syntax (`export` vs `set -x`). Address during Phase 3 implementation.

## Sources

### Primary (HIGH confidence)
- [clap 4.6 docs](https://docs.rs/crate/clap/latest) -- CLI framework API and derive patterns
- [serde 1.0 docs](https://docs.rs/crate/serde/latest) -- serialization framework
- [toml 1.1 docs](https://docs.rs/crate/toml/latest) -- TOML parsing and writing
- [Claude Code environment variables](https://code.claude.com/docs/en/env-vars) -- CLAUDE_CONFIG_DIR behavior
- [Claude Code settings docs](https://code.claude.com/docs/en/settings) -- config scopes and file locations
- [AWS CLI configuration](https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-files.html) -- profile management patterns
- [kubectl config contexts](https://kubernetes.io/docs/reference/kubectl/generated/kubectl_config/kubectl_config_use-context/) -- context switching patterns
- [Rust CLI testing guide](https://rust-cli.github.io/book/tutorial/testing.html) -- assert_cmd/assert_fs patterns

### Secondary (MEDIUM confidence)
- [GitHub Issue #3833](https://github.com/anthropics/claude-code/issues/3833) -- CLAUDE_CONFIG_DIR limitations and known bugs
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/) -- owo-colors, configuration patterns
- [cargo-dist releases](https://github.com/axodotdev/cargo-dist/releases) -- binary distribution automation
- [tempfile crate docs](https://docs.rs/tempfile/latest/tempfile/struct.NamedTempFile.html) -- atomic write pattern

### Tertiary (LOW confidence)
- macOS Keychain interaction with CLAUDE_CONFIG_DIR -- inferred risk, needs empirical validation
- Fish shell `set -x` syntax for env output -- known but not tested in context of clmux

---
*Research completed: 2026-04-02*
*Ready for roadmap: yes*
