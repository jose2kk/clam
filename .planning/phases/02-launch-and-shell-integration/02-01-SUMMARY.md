---
phase: 02-launch-and-shell-integration
plan: 01
subsystem: cli
tags: [rust, clap, exec, env-isolation, unix-process]

requires:
  - phase: 01-foundation-and-profile-management
    provides: "Profile CRUD, state.rs, config.rs, paths.rs, profile.rs"
provides:
  - "clmux run command with exec()-based process launching"
  - "resolve_profile() pub(crate) helper reusable by env command"
  - "CLAUDE_*/ANTHROPIC_* env var sanitization"
  - "CLI routing for Run and Env subcommands"
affects: [02-launch-and-shell-integration]

tech-stack:
  added: []
  patterns: ["unix exec() for process replacement", "env var stripping for isolation"]

key-files:
  created: [src/commands/run.rs, src/commands/env.rs]
  modified: [src/cli.rs, src/main.rs, src/commands/mod.rs]

key-decisions:
  - "exec() replaces clmux process (no orphaned children)"
  - "Strip CLAUDE_*/ANTHROPIC_* but not CLMUX_* env vars"
  - "Default binary is 'claude' when no args provided"

patterns-established:
  - "resolve_profile pattern: optional override vs active state fallback"
  - "exec() for process launch (no spawn/wait)"

requirements-completed: [LNCH-01, LNCH-02, LNCH-03, LNCH-04]

duration: 2min
completed: 2026-04-03
---

# Phase 2 Plan 1: Run Command Summary

**exec()-based process launcher with profile-scoped env isolation, CLAUDE_*/ANTHROPIC_* stripping, and resolve_profile helper**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-03T19:24:00Z
- **Completed:** 2026-04-03T19:25:28Z
- **Tasks:** 2
- **Files modified:** 5

## Accomplishments
- Extended CLI with Run (--profile flag, trailing_var_arg args) and Env subcommands
- Implemented run command using unix exec() to replace process with no orphaned children
- Built resolve_profile() pub(crate) helper for reuse by env command in plan 02-02
- Environment sanitization strips all CLAUDE_* and ANTHROPIC_* vars before setting CLAUDE_CONFIG_DIR

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend CLI with Run and Env subcommands** - `bfaf97c` (feat)
2. **Task 2: Implement run command with resolve_profile, env sanitization, and exec** - `2957a6c` (feat)

## Files Created/Modified
- `src/commands/run.rs` - Run command with resolve_profile, env sanitization, exec()
- `src/commands/env.rs` - Placeholder for plan 02-02
- `src/cli.rs` - Added Run and Env variants to Commands enum
- `src/main.rs` - Added routing for Run and Env commands
- `src/commands/mod.rs` - Added pub mod run and pub mod env

## Decisions Made
- Used exec() (CommandExt) to replace process rather than spawn/wait -- no orphaned children
- Strip CLAUDE_* and ANTHROPIC_* env vars but preserve CLMUX_* for tooling visibility
- Default to "claude" binary when no args provided (most common use case)
- resolve_profile is pub(crate) so env.rs can reuse it in plan 02-02

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

- `src/commands/env.rs:4` - `todo!("Implemented in plan 02")` - Intentional placeholder; env command implemented in plan 02-02.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- resolve_profile() ready for env.rs to reuse in plan 02-02
- Run subcommand wired and compiles; needs real Claude Code binary to test end-to-end
- All 32 existing Phase 1 tests pass unchanged

---
*Phase: 02-launch-and-shell-integration*
*Completed: 2026-04-03*
