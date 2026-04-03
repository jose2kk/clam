# Requirements: clmux

**Defined:** 2026-04-02
**Core Value:** Profile isolation — each Claude Code account's local state lives in its own directory, switching is explicit and safe.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Profile Management

- [x] **PROF-01**: User can create a named profile that provisions an isolated directory at `~/.clmux/profiles/<name>/`
- [x] **PROF-02**: User can list all configured profiles with the active profile visually marked
- [x] **PROF-03**: User can switch the active profile by name, updating `~/.clmux/state.toml`
- [x] **PROF-04**: User can print the active profile name to stdout for scripting (`clmux current`)
- [x] **PROF-05**: User can remove a profile by name with confirmation prompt (refuses to remove active profile)
- [x] **PROF-06**: Profile names are validated (alphanumeric, hyphens, underscores only) to prevent path traversal

### Launch & Integration

- [x] **LNCH-01**: User can launch Claude Code scoped to the active profile via `clmux run [-- args]`, setting `CLAUDE_CONFIG_DIR`
- [x] **LNCH-02**: User can launch Claude Code with a specific profile via `clmux run --profile <name> [-- args]`
- [x] **LNCH-03**: `clmux run` uses Unix `exec()` to replace the process (no orphaned children, correct signal handling)
- [x] **LNCH-04**: `clmux run` sanitizes environment variables (strips `ANTHROPIC_API_KEY`, `CLAUDE_CONFIG_DIR`, etc.) before launching
- [x] **LNCH-05**: User can print shell environment exports via `clmux env` for `eval "$(clmux env)"` integration
- [x] **LNCH-06**: User can view verbose active profile info via `clmux status` (name, path, directory health)

### Data Integrity

- [x] **DATA-01**: State file (`~/.clmux/state.toml`) is written atomically using tempfile + rename to prevent corruption
- [x] **DATA-02**: Config file (`~/.clmux/config.toml`) is written atomically
- [x] **DATA-03**: Profile directories are created with `0700` permissions (user-only access)
- [x] **DATA-04**: Profile directories are treated as opaque — clmux never parses Claude Code's internal files

### Developer Experience

- [x] **DEVX-01**: User can generate shell completions for bash, zsh, and fish via `clmux completions <shell>`
- [x] **DEVX-02**: User can get machine-parseable JSON output via `--json` flag on `list`, `status`, and `show`
- [x] **DEVX-03**: Colored output to TTY, plain output when piped (respects `NO_COLOR`)
- [x] **DEVX-04**: Clear error messages with actionable suggestions (e.g., "Profile 'foo' not found. Run `clmux list` to see available profiles.")

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Profile Enhancements

- **PROF-10**: User can inspect any profile's paths and state via `clmux show <name>`
- **PROF-11**: User can clone an existing profile to a new name via `clmux clone <source> <target>`

### Shell Integration

- **SHEL-01**: `CLMUX_PROFILE` env var overrides active profile for per-terminal isolation
- **SHEL-02**: User can debug profile resolution order via `clmux which`
- **SHEL-03**: User can get active profile name for shell prompt via `clmux prompt [--format]`

### Security

- **SECR-01**: Optional keychain integration for credential storage
- **SECR-02**: Profile directory encryption at rest

## Out of Scope

Explicitly excluded. Documented to prevent scope creep.

| Feature | Reason |
|---------|--------|
| API request proxying | clmux is a profile manager, not a middleware |
| Cloud sync / cross-machine state | Local-only tool; users can tar/rsync `~/.clmux/` |
| TUI / interactive picker | Target audience types commands; pipe to fzf if needed |
| Plugin system | Premature abstraction for a tool this focused |
| Config file editing within profiles | Claude Code's responsibility, not ours |
| Windows support | macOS + Linux first; Rust makes eventual support feasible |
| Automatic shell mutation (direnv-style) | Explicit > magic; user opts in via `eval` |
| Profile import/export | Directories are just directories; users can cp/tar |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| PROF-01 | Phase 1 | Complete |
| PROF-02 | Phase 1 | Complete |
| PROF-03 | Phase 1 | Complete |
| PROF-04 | Phase 1 | Complete |
| PROF-05 | Phase 1 | Complete |
| PROF-06 | Phase 1 | Complete |
| LNCH-01 | Phase 2 | Complete |
| LNCH-02 | Phase 2 | Complete |
| LNCH-03 | Phase 2 | Complete |
| LNCH-04 | Phase 2 | Complete |
| LNCH-05 | Phase 2 | Complete |
| LNCH-06 | Phase 2 | Complete |
| DATA-01 | Phase 1 | Complete |
| DATA-02 | Phase 1 | Complete |
| DATA-03 | Phase 1 | Complete |
| DATA-04 | Phase 1 | Complete |
| DEVX-01 | Phase 3 | Complete |
| DEVX-02 | Phase 3 | Complete |
| DEVX-03 | Phase 1 | Complete |
| DEVX-04 | Phase 1 | Complete |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 20
- Unmapped: 0

---
*Requirements defined: 2026-04-02*
*Last updated: 2026-04-02 after roadmap creation*
