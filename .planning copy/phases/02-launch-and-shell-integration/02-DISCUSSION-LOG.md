# Phase 2: Launch and Shell Integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 02-launch-and-shell-integration
**Areas discussed:** Env sanitization, Run behavior, Shell env output

---

## Env Sanitization

### Strip Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal denylist | Strip only CLAUDE_CONFIG_DIR and ANTHROPIC_API_KEY | |
| Broad Claude denylist | Strip all CLAUDE_* and ANTHROPIC_* env vars | ✓ |
| You decide | Claude picks appropriate sanitization scope | |

**User's choice:** Broad Claude denylist
**Notes:** Ensures no leakage from parent shell

### Inject Vars

| Option | Description | Selected |
|--------|-------------|----------|
| Just CLAUDE_CONFIG_DIR | Single var injection — minimal, predictable | |
| CLAUDE_CONFIG_DIR + CLMUX_PROFILE | Also set CLMUX_PROFILE for scripts/prompts | ✓ |

**User's choice:** CLAUDE_CONFIG_DIR + CLMUX_PROFILE
**Notes:** None

---

## Run Behavior

### Default Command

| Option | Description | Selected |
|--------|-------------|----------|
| Launch `claude` | Default to running the `claude` binary with active profile's env | ✓ |
| Require command | `clmux run` alone errors | |
| You decide | Claude picks based on UX research | |

**User's choice:** Launch `claude`
**Notes:** None

### Exec Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| exec() always | Replace clmux process with child — no orphans | ✓ |
| spawn + wait | Keep clmux alive as parent process | |

**User's choice:** exec() always
**Notes:** Standard for wrapper tools

---

## Shell Env Output

### Shell Detection

| Option | Description | Selected |
|--------|-------------|----------|
| Bash/POSIX only | Always output `export VAR=value` | ✓ |
| Auto-detect shell | Check SHELL env var | |
| Flag-based | `clmux env --shell bash/fish` | |

**User's choice:** Bash/POSIX only
**Notes:** Fish/PowerShell users use `clmux run` instead

### Export Vars

| Option | Description | Selected |
|--------|-------------|----------|
| CLAUDE_CONFIG_DIR only | Minimal — just the one var that matters | |
| CLAUDE_CONFIG_DIR + CLMUX_PROFILE | Both — matches what `clmux run` injects | ✓ |
| Full unset + set | Unset CLAUDE_*/ANTHROPIC_* vars too | |

**User's choice:** CLAUDE_CONFIG_DIR + CLMUX_PROFILE
**Notes:** None

---

## Claude's Discretion

- Exact CLAUDE_*/ANTHROPIC_* env var patterns to strip
- Whether `clmux env` also emits `unset` statements
- Error message when `claude` not in PATH
- Status directory health detail

## Deferred Ideas

None — discussion stayed within phase scope
