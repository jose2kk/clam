---
phase: 03-developer-experience-polish
plan: 02
subsystem: cli
tags: [json, serde_json, scripting, automation]

requires:
  - phase: 03-01
    provides: "--json flag plumbing in CLI arg parser"
provides:
  - "JSON output for list, status, and env commands via --json flag"
  - "Machine-parseable output for scripting and automation"
affects: []

tech-stack:
  added: []
  patterns: [json-output-branch-pattern]

key-files:
  created: []
  modified:
    - src/commands/list.rs
    - src/commands/status.rs
    - src/commands/env.rs

key-decisions:
  - "JSON output uses serde_json::to_string_pretty for human-readable formatted JSON"
  - "env --json omits unset commands -- JSON consumers manage their own env"

patterns-established:
  - "JSON output pattern: check json bool param early, serialize with serde_json::json! macro, return early before colored output path"

requirements-completed: [DEVX-02]

duration: 1min
completed: 2026-04-03
---

# Phase 3 Plan 2: JSON Output for Commands Summary

**Machine-parseable JSON output for list/status/env commands using serde_json with --json flag**

## Performance

- **Duration:** 1 min
- **Started:** 2026-04-03T19:49:43Z
- **Completed:** 2026-04-03T19:51:07Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- list --json outputs JSON array with name/active/path per profile
- status --json outputs JSON object with profile/path/exists/config_dir/items fields
- env --json outputs JSON object with CLAUDE_CONFIG_DIR and CLMUX_PROFILE
- All 43 existing tests pass, no build warnings

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement --json for list and status commands** - `d7f56fb` (feat)
2. **Task 2: Implement --json for env command** - `2f79814` (feat)

## Files Created/Modified
- `src/commands/list.rs` - Added JSON output branch with serde_json::json! macro for profile array
- `src/commands/status.rs` - Added JSON output branch for profile status object
- `src/commands/env.rs` - Added JSON output branch for environment variable object

## Decisions Made
- Used serde_json::to_string_pretty for human-readable JSON output (easier to debug when piped)
- env --json omits the unset commands since JSON consumers handle their own environment

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All three read commands support --json flag
- JSON schemas match CONTEXT.md specifications
- Ready for any downstream scripting/automation work

## Self-Check: PASSED

- All 3 modified files exist on disk
- Both task commits found in git log (d7f56fb, 2f79814)
- All 3 files contain serde_json imports
