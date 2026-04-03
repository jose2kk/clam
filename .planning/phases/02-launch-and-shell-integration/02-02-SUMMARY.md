---
phase: 02-launch-and-shell-integration
plan: 02
subsystem: cli
tags: [shell-integration, env-vars, posix, integration-tests]

# Dependency graph
requires:
  - phase: 02-launch-and-shell-integration/01
    provides: "run command with resolve_profile, env sanitization, exec()"
provides:
  - "clmux env command outputting POSIX shell export/unset syntax"
  - "enhanced clmux status with CLAUDE_CONFIG_DIR and item count"
  - "11 integration tests covering run, env, and enhanced status"
affects: [03-developer-experience-polish]

# Tech tracking
tech-stack:
  added: []
  patterns: ["POSIX shell output for eval integration", "directory item count as health indicator"]

key-files:
  created: []
  modified:
    - src/commands/env.rs
    - src/commands/status.rs
    - tests/cli_tests.rs

key-decisions:
  - "env command uses resolve_profile(None) to always target active profile (no --profile flag per D-08)"
  - "status item count uses read_dir().count() respecting DATA-04 opaque directory contract"

patterns-established:
  - "Shell integration output: every line is valid POSIX (export or unset with semicolons)"
  - "Health indicators: count items without parsing directory contents"

requirements-completed: [LNCH-05, LNCH-06]

# Metrics
duration: 2min
completed: 2026-04-03
---

# Phase 2 Plan 2: Env Command and Enhanced Status Summary

**Shell integration via `clmux env` with POSIX export/unset output, enhanced status with CLAUDE_CONFIG_DIR and item count, plus 11 integration tests for run/env/status**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-03T19:27:02Z
- **Completed:** 2026-04-03T19:28:48Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Implemented `clmux env` command that outputs valid POSIX shell syntax for `eval "$(clmux env)"` usage
- Enhanced `clmux status` with Config line (CLAUDE_CONFIG_DIR) and Items line (file count)
- Added 11 integration tests: 6 for run command, 3 for env command, 2 for enhanced status

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement env command and enhance status with directory health** - `77c2945` (feat)
2. **Task 2: Add integration tests for run, env, and enhanced status commands** - `f21a14c` (test)

## Files Created/Modified
- `src/commands/env.rs` - Full env command: resolve_profile, unset CLAUDE_*/ANTHROPIC_*, export CLAUDE_CONFIG_DIR and CLMUX_PROFILE
- `src/commands/status.rs` - Added Config line with CLAUDE_CONFIG_DIR value and Items line with file count
- `tests/cli_tests.rs` - 11 new integration tests (43 total) covering run, env, and enhanced status

## Decisions Made
- env command uses resolve_profile(None) to always target active profile (no --profile flag per D-08)
- status item count uses read_dir().count() respecting DATA-04 opaque directory contract

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All Phase 2 commands (run, env, status) are implemented and tested
- 43 integration tests pass covering Phase 1 + Phase 2 functionality
- Ready for Phase 3 developer experience polish

---
*Phase: 02-launch-and-shell-integration*
*Completed: 2026-04-03*
