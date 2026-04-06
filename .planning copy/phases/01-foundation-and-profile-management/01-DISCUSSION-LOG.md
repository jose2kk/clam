# Phase 1: Foundation and Profile Management - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 01-foundation-and-profile-management
**Areas discussed:** Output formatting, Profile creation, Removal safety, CLI conventions

---

## Output Formatting

### List Format

| Option | Description | Selected |
|--------|-------------|----------|
| Simple list | One profile per line, asterisk for active: `* work\n  personal` | ✓ |
| Table | Columns with headers: NAME \| ACTIVE \| PATH | |
| kubectl-style | Table with `*` marker column | |

**User's choice:** Simple list
**Notes:** None

### Status Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Compact | Active profile name + path + directory exists (3 lines) | ✓ |
| Detailed | Name, path, dir exists, dir size, file count, last modified timestamp | |
| You decide | Claude picks appropriate detail level | |

**User's choice:** Compact
**Notes:** None

### Current No-Profile Behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Empty + exit 1 | Print nothing, non-zero exit code (scriptable) | ✓ |
| Message + exit 1 | Print 'No active profile' to stderr, exit 1 | |
| You decide | Claude picks | |

**User's choice:** Empty + exit 1
**Notes:** Modeled after `git branch --show-current`

---

## Profile Creation

### Auto-activate

| Option | Description | Selected |
|--------|-------------|----------|
| First only | Auto-activate the first profile added, subsequent ones just get created | ✓ |
| Always | Every `add` switches active to the new profile | |
| Never | Always require explicit `clmux use work` after adding | |

**User's choice:** First only
**Notes:** None

### Metadata

| Option | Description | Selected |
|--------|-------------|----------|
| Bare directory only | No metadata — just the name and directory | ✓ |
| Optional description | `clmux add work --description 'Work account'` stored in config.toml | |

**User's choice:** Bare directory only
**Notes:** None

---

## Removal Safety

### Confirmation Style

| Option | Description | Selected |
|--------|-------------|----------|
| y/N prompt | Simple 'Remove profile work? [y/N]' with --force to skip | ✓ |
| Type name | Type the profile name to confirm (like `gh repo delete`) | |
| No confirm + --force | Just delete, but require --force flag (no interactive prompt) | |

**User's choice:** y/N prompt
**Notes:** None

### Last Profile

| Option | Description | Selected |
|--------|-------------|----------|
| Allow it | User manages their own state — empty is valid | ✓ |
| Warn but allow | Print warning, still proceed | |
| Block it | Refuse removal of the last profile | |

**User's choice:** Allow it
**Notes:** None

---

## CLI Conventions

### Home Directory

| Option | Description | Selected |
|--------|-------------|----------|
| ~/.clmux/ | Simple, discoverable, matches ~/.aws/ pattern | ✓ |
| XDG dirs | ~/.config/clmux/ for config, ~/.local/share/clmux/ for data | |
| You decide | Claude picks based on target platforms | |

**User's choice:** ~/.clmux/
**Notes:** None

### Global Flags

| Option | Description | Selected |
|--------|-------------|----------|
| None for v1 | Keep it minimal, add flags when needed | ✓ |

**User's choice:** None for v1
**Notes:** NO_COLOR env var still respected for color control

### Exit Codes

| Option | Description | Selected |
|--------|-------------|----------|
| Simple (0/1) | 0 = success, 1 = any error | ✓ |
| Differentiated | 0 = success, 1 = user error, 2 = system error | |

**User's choice:** Simple (0/1)
**Notes:** None

---

## Claude's Discretion

- Status output detail beyond 3-line minimum
- Error message wording and suggestion phrasing
- Internal module organization

## Deferred Ideas

None — discussion stayed within phase scope
