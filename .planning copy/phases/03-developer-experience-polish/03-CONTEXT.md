# Phase 3: Developer Experience Polish - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Add shell completions generation (`clmux completions <shell>`) and machine-parseable JSON output (`--json` flag) to read commands. This is additive polish on top of a working tool — no behavioral changes to existing commands.

</domain>

<decisions>
## Implementation Decisions

### Claude's Discretion (user deferred all decisions)

**Shell Completions:**
- `clmux completions <shell>` subcommand outputting completion scripts to stdout (bash, zsh, fish)
- Use `clap_complete` crate — near-free with clap derive
- Output to stdout for user to redirect (e.g., `clmux completions zsh > _clmux`)
- No auto-install — user manages their completion setup

**JSON Output:**
- `--json` flag on `list`, `status`, and `env` commands
- Structured JSON to stdout (pipe to `jq`)
- `list --json`: array of `{"name": "...", "active": true/false, "path": "..."}`
- `status --json`: object with `{"profile": "...", "path": "...", "exists": true/false, "config_dir": "...", "items": N}`
- `env --json`: object with `{"CLAUDE_CONFIG_DIR": "...", "CLMUX_PROFILE": "..."}`
- `current` and `add`/`use`/`remove`/`run` do NOT get `--json` — they're either already scriptable or mutating

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Existing Code
- `src/cli.rs` — CLI structure to extend with `Completions` subcommand and `--json` flags
- `src/main.rs` — Command routing
- `src/commands/list.rs` — Add `--json` output path
- `src/commands/status.rs` — Add `--json` output path
- `src/commands/env.rs` — Add `--json` output path
- `src/commands/mod.rs` — Add completions module

### Research
- `.planning/research/STACK.md` — clap_complete crate info
- `.planning/research/FEATURES.md` — Completions and --json as differentiators

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `clap` derive already in use — `clap_complete` integrates directly
- `serde` already in Cargo.toml — add `serde_json` for JSON serialization
- Existing command structs can derive `Serialize` for JSON output

### Established Patterns
- One file per command in `src/commands/`
- `pub fn execute(...)` signature
- `anyhow::Result` error handling

### Integration Points
- `src/cli.rs` — add `Completions` variant and `--json` flags to `List`, `Status`, `Env`
- `Cargo.toml` — add `clap_complete` and `serde_json` dependencies

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-developer-experience-polish*
*Context gathered: 2026-04-03*
