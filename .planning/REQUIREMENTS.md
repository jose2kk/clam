# Requirements: clmux

**Defined:** 2026-04-02
**Core Value:** Profile isolation — each Claude Code account's local state lives in its own directory, switching is explicit and safe.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Profile Management

- [ ] **PROF-01**: User can create a named profile that provisions an isolated directory at `~/.clmux/profiles/<name>/`
- [ ] **PROF-02**: User can list all configured profiles with the active profile visually marked
- [ ] **PROF-03**: User can switch the active profile by name, updating `~/.clmux/state.toml`
- [ ] **PROF-04**: User can print the active profile name to stdout for scripting (`clmux current`)
- [ ] **PROF-05**: User can remove a profile by name with confirmation prompt (refuses to remove active profile)
- [ ] **PROF-06**: Profile names are validated (alphanumeric, hyphens, underscores only) to prevent path traversal

### Launch & Integration

- [ ] **LNCH-01**: User can launch Claude Code scoped to the active profile via `clmux run [-- args]`, setting `CLAUDE_CONFIG_DIR`
- [ ] **LNCH-02**: User can launch Claude Code with a specific profile via `clmux run --profile <name> [-- args]`
- [ ] **LNCH-03**: `clmux run` uses Unix `exec()` to replace the process (no orphaned children, correct signal handling)
- [ ] **LNCH-04**: `clmux run` sanitizes environment variables (strips `ANTHROPIC_API_KEY`, `CLAUDE_CONFIG_DIR`, etc.) before launching
- [ ] **LNCH-05**: User can print shell environment exports via `clmux env` for `eval "$(clmux env)"` integration
- [ ] **LNCH-06**: User can view verbose active profile info via `clmux status` (name, path, directory health)

### Data Integrity

- [ ] **DATA-01**: State file (`~/.clmux/state.toml`) is written atomically using tempfile + rename to prevent corruption
- [ ] **DATA-02**: Config file (`~/.clmux/config.toml`) is written atomically
- [ ] **DATA-03**: Profile directories are created with `0700` permissions (user-only access)
- [ ] **DATA-04**: Profile directories are treated as opaque — clmux never parses Claude Code's internal files

### Developer Experience

- [ ] **DEVX-01**: User can generate shell completions for bash, zsh, and fish via `clmux completions <shell>`
- [ ] **DEVX-02**: User can get machine-parseable JSON output via `--json` flag on `list`, `status`, and `show`
- [ ] **DEVX-03**: Colored output to TTY, plain output when piped (respects `NO_COLOR`)
- [ ] **DEVX-04**: Clear error messages with actionable suggestions (e.g., "Profile 'foo' not found. Run `clmux list` to see available profiles.")

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
| PROF-01 | Pending | Pending |
| PROF-02 | Pending | Pending |
| PROF-03 | Pending | Pending |
| PROF-04 | Pending | Pending |
| PROF-05 | Pending | Pending |
| PROF-06 | Pending | Pending |
| LNCH-01 | Pending | Pending |
| LNCH-02 | Pending | Pending |
| LNCH-03 | Pending | Pending |
| LNCH-04 | Pending | Pending |
| LNCH-05 | Pending | Pending |
| LNCH-06 | Pending | Pending |
| DATA-01 | Pending | Pending |
| DATA-02 | Pending | Pending |
| DATA-03 | Pending | Pending |
| DATA-04 | Pending | Pending |
| DEVX-01 | Pending | Pending |
| DEVX-02 | Pending | Pending |
| DEVX-03 | Pending | Pending |
| DEVX-04 | Pending | Pending |

**Coverage:**
- v1 requirements: 20 total
- Mapped to phases: 0
- Unmapped: 20

---
*Requirements defined: 2026-04-02*
*Last updated: 2026-04-02 after initial definition*
