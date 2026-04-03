# Roadmap: clmux

## Overview

clmux delivers profile isolation for Claude Code in three phases: first establish the foundation with profile CRUD, data integrity, and core output behavior; then add process launching and shell integration (the primary value prop); finally polish with completions and machine-parseable output. Each phase produces a testable increment that builds on the previous one.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Foundation and Profile Management** - Project scaffolding, profile CRUD, data integrity, and core CLI output behavior
- [ ] **Phase 2: Launch and Shell Integration** - Process launching with env isolation, shell env exports, and profile status inspection
- [ ] **Phase 3: Developer Experience Polish** - Shell completions and JSON output for scripting

## Phase Details

### Phase 1: Foundation and Profile Management
**Goal**: Users can create, list, switch, inspect, and remove isolated Claude Code profiles with safe data handling
**Depends on**: Nothing (first phase)
**Requirements**: PROF-01, PROF-02, PROF-03, PROF-04, PROF-05, PROF-06, DATA-01, DATA-02, DATA-03, DATA-04, DEVX-03, DEVX-04
**Success Criteria** (what must be TRUE):
  1. User can run `clmux add work` and a directory is created at `~/.clmux/profiles/work/` with 0700 permissions
  2. User can run `clmux list` and see all profiles with the active one visually marked, with colored output to TTY and plain when piped
  3. User can run `clmux use work` to switch profiles and `clmux current` to confirm the switch, with state persisted across shell sessions
  4. User can run `clmux remove work` and get a confirmation prompt that refuses to remove the active profile
  5. User sees clear, actionable error messages when referencing nonexistent profiles or using invalid names (e.g., `../evil`)
**Plans:** 4 plans

Plans:
- [ ] 01-01-PLAN.md -- Cargo scaffold, core modules, and `add` command
- [ ] 01-02-PLAN.md -- Read commands: `list`, `current`, `status`
- [ ] 01-03-PLAN.md -- Mutating commands: `use`, `remove`
- [ ] 01-04-PLAN.md -- End-to-end integration tests and error message audit

### Phase 2: Launch and Shell Integration
**Goal**: Users can launch Claude Code scoped to the active profile with clean environment isolation
**Depends on**: Phase 1
**Requirements**: LNCH-01, LNCH-02, LNCH-03, LNCH-04, LNCH-05, LNCH-06
**Success Criteria** (what must be TRUE):
  1. User can run `clmux run` and Claude Code launches with `CLAUDE_CONFIG_DIR` pointing to the active profile's directory, replacing the clmux process (no orphans)
  2. User can run `clmux run --profile work -- --help` to launch Claude Code with a specific profile and pass-through arguments
  3. User can run `eval "$(clmux env)"` and have `CLAUDE_CONFIG_DIR` set correctly in their shell, with known Claude-related env vars stripped
  4. User can run `clmux status` and see the active profile name, directory path, and directory health
**Plans**: TBD

### Phase 3: Developer Experience Polish
**Goal**: Users get shell completions and machine-parseable output for scripting and automation
**Depends on**: Phase 2
**Requirements**: DEVX-01, DEVX-02
**Success Criteria** (what must be TRUE):
  1. User can run `clmux completions zsh` (or bash/fish) and get valid shell completion scripts
  2. User can run `clmux list --json`, `clmux status --json` to get structured JSON output suitable for piping to jq
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Foundation and Profile Management | 0/4 | Planned | - |
| 2. Launch and Shell Integration | 0/0 | Not started | - |
| 3. Developer Experience Polish | 0/0 | Not started | - |
