# Phase 1: Foundation and Profile Management - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver the core Rust project scaffolding, profile CRUD commands (`add`, `list`, `use`, `current`, `remove`), data integrity foundations (atomic writes, permissions, name validation), and baseline CLI output behavior (color, errors). This phase produces a working binary that manages profiles but does NOT launch Claude Code or integrate with the shell.

</domain>

<decisions>
## Implementation Decisions

### Output Formatting
- **D-01:** `clmux list` uses simple list format — one profile per line, asterisk prefix for active: `* work\n  personal`
- **D-02:** `clmux status` is compact — active profile name, path, and directory-exists check (3 lines max)
- **D-03:** `clmux current` with no active profile prints nothing to stdout and exits with code 1 (scriptable, like `git branch --show-current`)

### Profile Creation
- **D-04:** First `clmux add` auto-activates the new profile; subsequent adds do not (user must `clmux use` explicitly)
- **D-05:** Profiles are bare directories — no metadata, no description field, no config.toml per profile. Just name → directory.

### Removal Safety
- **D-06:** `clmux remove <name>` prompts `Remove profile <name>? [y/N]` with `--force` flag to skip confirmation
- **D-07:** Active profile cannot be removed (must switch first — clear error message)
- **D-08:** Last remaining profile CAN be removed — user manages their own state, empty is valid

### CLI Conventions
- **D-09:** Data stored at `~/.clmux/` (not XDG) — simple, discoverable, matches `~/.aws/` pattern
- **D-10:** No global flags for v1 — no `--quiet`, `--verbose`, `--no-color`. Rely on `NO_COLOR` env var for color control.
- **D-11:** Exit codes are simple: 0 = success, 1 = any error

### Claude's Discretion
- Status output detail level beyond the 3-line minimum — Claude can include additional useful info if it's concise
- Error message wording and suggestion phrasing
- Internal module organization within the constraints (flat, minimal abstraction)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Research
- `.planning/research/STACK.md` — Recommended crate versions and rationale (clap 4.6, serde, toml 1.1, anyhow, dirs 6.0, owo-colors 4.2, tempfile)
- `.planning/research/FEATURES.md` — Feature landscape with table stakes, differentiators, anti-features, and dependency graph
- `.planning/research/ARCHITECTURE.md` — Component boundaries, data flow, build order
- `.planning/research/PITFALLS.md` — Domain pitfalls: atomic writes, name validation, opaque dirs, path traversal prevention
- `.planning/research/SUMMARY.md` — Synthesized findings and phase implications

### Project
- `.planning/PROJECT.md` — Core value, constraints, key decisions
- `.planning/REQUIREMENTS.md` — v1 requirements with REQ-IDs and phase mapping

No external specs — requirements fully captured in decisions above.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None — greenfield project, no existing code

### Established Patterns
- None — patterns will be established in this phase

### Integration Points
- `~/.clmux/config.toml` — profile registry (list of profile names)
- `~/.clmux/state.toml` — active profile pointer
- `~/.clmux/profiles/<name>/` — isolated profile directories

</code_context>

<specifics>
## Specific Ideas

- Active profile marker in list output should be a simple `*` prefix (not arrow, not color-only)
- `clmux current` should behave like `git branch --show-current` — pure stdout, no decoration, scriptable
- Confirmation prompt follows the `[y/N]` convention (default is No)
- Profile names: alphanumeric + hyphens + underscores only (research flagged path traversal as critical pitfall)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-foundation-and-profile-management*
*Context gathered: 2026-04-03*
