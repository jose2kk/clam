# clmux

## What This Is

A Rust CLI tool that manages multiple Claude Code accounts on a single machine. Each account (profile) gets its own isolated directory for auth state, config, and session data, so two Claude Code accounts never interfere with each other. Think `aws configure` meets `kubectl config use-context`, but for Claude Code.

## Core Value

Profile isolation — each Claude Code account's local state lives in its own directory, and switching between them is explicit, predictable, and safe.

## Requirements

### Validated

- User can add a named profile with isolated directory — Phase 1
- User can list all configured profiles — Phase 1
- User can see which profile is currently active — Phase 1
- User can switch the active profile — Phase 1
- User can remove a profile — Phase 1
- Each profile maps to its own `~/.clmux/profiles/<name>/` directory — Phase 1
- Active profile tracked in `~/.clmux/state.toml` — Phase 1
- User can launch Claude Code scoped to a specific profile's environment — Phase 2
- User can inspect a profile's paths and state — Phase 2
- Profiles store Claude Code auth state, config, and session data in isolation — Phase 2

### Active

- [ ] Tool distributed as a single static binary (Mac + Linux)

### Out of Scope

- API request proxying — clmux is a profile manager, not a middleware
- Cloud sync / cross-machine session syncing — local-only tool
- Profile import/export — can be added later if needed
- TUI interface — CLI-only for v1
- Plugin system — no extensibility framework
- Credential encryption / keychain integration — v1 uses filesystem isolation; encrypt later
- Shell auto-injection — no automatic env mutation; user opts in via `eval` or `clmux run`

## Context

- Claude Code stores local state in `~/.claude/` by default
- The `CLAUDE_CONFIG_DIR` environment variable (or equivalent) can redirect Claude Code's config home — this is the primary isolation mechanism
- Developers using multiple Claude Code accounts (personal + work, multiple orgs) currently have no ergonomic way to switch between them without manually moving directories
- The tool name "clmux" = Claude multiplexer
- Target users: developers already comfortable with CLI tools like aws, gh, kubectl

## Constraints

- **Language**: Rust — performance, single binary distribution, no runtime deps
- **CLI framework**: clap (latest) — industry standard for Rust CLIs
- **Config format**: TOML — Rust ecosystem default (serde_toml, Cargo.toml precedent), human-readable, simpler than YAML for flat config
- **OS support**: macOS + Linux first; Windows deferred
- **Architecture**: Minimal abstraction, no trait-heavy design, no speculative extensibility
- **Distribution**: Single statically-linked binary

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| TOML over YAML for config | Rust ecosystem convention (Cargo.toml), simpler spec, no gotchas like YAML's Norway problem, serde support is first-class | — Pending |
| Directory-based isolation over env-var-only | Predictable, debuggable, survives shell session boundaries | — Pending |
| No keychain/encryption in v1 | Claude Code manages its own auth; clmux just points it at a directory. No secrets to encrypt yet. | — Pending |
| No automatic shell mutation | Explicit > magic. User runs `eval $(clmux env)` or `clmux run` intentionally. | — Pending |
| Flat module structure over deep trait hierarchies | v1 MVP — fewer files, less indirection, easier to ship and iterate | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-03 after Phase 2 completion*
