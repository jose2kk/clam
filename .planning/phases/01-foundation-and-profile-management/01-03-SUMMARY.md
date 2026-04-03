---
phase: 01-foundation-and-profile-management
plan: 03
subsystem: cli
tags: [rust, profile-switching, profile-removal, confirmation-prompt, tty-detection]

requires:
  - phase: 01-00
    provides: "Project initialization and planning artifacts"
  - phase: 01-01
    provides: "Core modules (config, state, paths, profile, output) and add command"
provides:
  - "Profile switching via `clmux use <name>` with atomic state update"
  - "Profile removal via `clmux remove <name>` with confirmation prompt and safety checks"
  - "Active profile protection (cannot remove active profile)"
  - "Non-TTY stdin detection with --force flag guidance"
affects: [01-04]

tech-stack:
  added: []
  patterns: [confirmation-prompt-with-default-no, active-profile-protection, non-tty-stdin-detection, force-flag-bypass]

key-files:
  created: []
  modified:
    - src/commands/use_cmd.rs
    - src/commands/remove.rs
    - tests/cli_tests.rs

key-decisions:
  - "Tests verify state via state.toml file reads rather than `clmux current` (which is implemented in parallel plan 01-02)"
  - "Non-TTY stdin defaults to rejection with guidance to use --force, rather than silently aborting"

patterns-established:
  - "Confirmation prompt pattern: y/N default with is_terminal() check and --force bypass"
  - "Active profile protection: check state before destructive operations"
  - "Validation-first flow: validate name -> check registry -> check constraints -> execute"

requirements-completed: [PROF-03, PROF-05]

duration: 2min
completed: 2026-04-03
---

# Phase 1 Plan 03: Use and Remove Commands Summary

**Profile switching with atomic state writes and profile removal with y/N confirmation prompt, active-profile protection, and --force bypass for non-TTY environments**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-03T18:24:02Z
- **Completed:** 2026-04-03T18:26:11Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- `clmux use <name>` switches active profile with validation, registry check, and atomic state write
- `clmux remove <name>` with confirmation prompt (y/N default), --force flag, active profile protection
- Non-TTY stdin detection prevents hanging on missing input, with actionable --force guidance
- 9 new integration tests (4 for use, 5 for remove), all 19 total tests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement use command** - `f64c83a` (test: RED), `1cf67d5` (feat: GREEN)
2. **Task 2: Implement remove command** - `d4a45d4` (test: RED), `28ed1be` (feat: GREEN)

_Note: TDD tasks have two commits each (failing test then implementation)_

## Files Created/Modified
- `src/commands/use_cmd.rs` - Profile switching: validate name, check registry, update state atomically
- `src/commands/remove.rs` - Profile removal: validate, check active, confirm (or --force), delete dir + config entry
- `tests/cli_tests.rs` - 9 new integration tests for use and remove commands

## Decisions Made
- Tests verify state.toml directly rather than depending on `clmux current` (implemented in parallel plan 01-02)
- Non-TTY stdin gets explicit "Cannot prompt" error with --force guidance rather than silent abort

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

None - all commands in this plan are fully implemented.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Profile lifecycle complete: add, switch, remove all functional
- Ready for status/env/run commands in plan 01-04
- Confirmation prompt pattern established and reusable

## Self-Check: PASSED

All 3 key files verified present. All 4 task commits (f64c83a, 1cf67d5, d4a45d4, 28ed1be) verified in git history.

---
*Phase: 01-foundation-and-profile-management*
*Completed: 2026-04-03*
