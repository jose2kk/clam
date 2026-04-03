---
phase: 01-foundation-and-profile-management
plan: 01
subsystem: cli
tags: [rust, clap, toml, serde, anyhow, owo-colors, tempfile, atomic-writes]

requires:
  - phase: 01-00
    provides: "Project initialization and planning artifacts"
provides:
  - "Compilable clmux binary with add command"
  - "Core modules: paths, config, state, profile, output, CLI definition"
  - "Atomic write pattern via tempfile+rename"
  - "Profile name validation with path traversal prevention"
  - "TTY-aware colored output helpers"
  - "Integration test harness using assert_cmd + assert_fs"
affects: [01-02, 01-03, 01-04]

tech-stack:
  added: [clap 4.6, serde, toml 1.1, anyhow, dirs 6.0, owo-colors 4.2, tempfile 3, assert_cmd 2, assert_fs 1, predicates 3]
  patterns: [atomic-write-via-tempfile, clmux-home-env-override, profile-name-validation-regex, first-add-auto-activates]

key-files:
  created:
    - Cargo.toml
    - src/main.rs
    - src/cli.rs
    - src/paths.rs
    - src/config.rs
    - src/state.rs
    - src/profile.rs
    - src/output.rs
    - src/commands/mod.rs
    - src/commands/add.rs
    - src/commands/list.rs
    - src/commands/use_cmd.rs
    - src/commands/current.rs
    - src/commands/remove.rs
    - src/commands/status.rs
    - tests/cli_tests.rs
  modified: []

key-decisions:
  - "Extracted atomic_write helper into paths.rs for reuse by both config.rs and state.rs"
  - "Profile name regex allows [a-zA-Z0-9_-]+ with explicit . and .. rejection"

patterns-established:
  - "Atomic writes: all TOML files written via tempfile::NamedTempFile::persist()"
  - "Path resolution: CLMUX_HOME env var overrides ~/.clmux/ base directory"
  - "Profile validation: validate_profile_name() called before any path construction"
  - "Error handling: anyhow::Result with context() for user-facing messages"
  - "CLI structure: clap derive with subcommands, exit 0/1 only"
  - "Test isolation: CLMUX_HOME pointed at TempDir for integration tests"

requirements-completed: [PROF-01, PROF-06, DATA-01, DATA-02, DATA-03, DATA-04]

duration: 2min
completed: 2026-04-03
---

# Phase 1 Plan 01: Cargo Scaffold and Add Command Summary

**Rust CLI scaffold with clap-derived subcommands, atomic TOML I/O, profile directory creation with 0700 permissions, and 10 passing integration tests**

## Performance

- **Duration:** 2 min
- **Started:** 2026-04-03T18:19:11Z
- **Completed:** 2026-04-03T18:21:18Z
- **Tasks:** 2
- **Files modified:** 17

## Accomplishments
- Complete Rust project scaffold with all core modules compiling
- Add command creates isolated profile directories with 0700 permissions
- First profile auto-activates, subsequent profiles do not
- Atomic writes for config.toml and state.toml via tempfile+rename
- Profile name validation rejects path traversal, empty, dots, spaces
- 10 integration tests covering all add command behaviors

## Task Commits

Each task was committed atomically:

1. **Task 1: Cargo project scaffold and core modules** - `a6f7068` (feat)
2. **Task 2: Add command integration tests** - `1c9aa60` (test)

## Files Created/Modified
- `Cargo.toml` - Project manifest with all dependencies
- `src/main.rs` - Entry point with error handling and command dispatch
- `src/cli.rs` - Clap derive CLI definition with all subcommands
- `src/paths.rs` - Path resolution with CLMUX_HOME support and atomic_write helper
- `src/config.rs` - Config file I/O with atomic writes
- `src/state.rs` - State file I/O with atomic writes
- `src/profile.rs` - Profile name validation and directory creation
- `src/output.rs` - TTY-aware colored output helpers
- `src/commands/mod.rs` - Command module re-exports
- `src/commands/add.rs` - Add command implementation
- `src/commands/list.rs` - Stub (not yet implemented)
- `src/commands/use_cmd.rs` - Stub (not yet implemented)
- `src/commands/current.rs` - Stub (not yet implemented)
- `src/commands/remove.rs` - Stub (not yet implemented)
- `src/commands/status.rs` - Stub (not yet implemented)
- `tests/cli_tests.rs` - 10 integration tests for add command

## Decisions Made
- Extracted `atomic_write` into `paths.rs` as shared helper rather than duplicating in config.rs and state.rs
- Profile name validation uses char-by-char check rather than regex crate (no additional dependency needed)

## Deviations from Plan

None - plan executed exactly as written.

## Known Stubs

The following stub commands exist and are intentional -- they will be implemented in subsequent plans (01-02, 01-03):

- `src/commands/list.rs` - Returns "Not yet implemented" error
- `src/commands/use_cmd.rs` - Returns "Not yet implemented" error
- `src/commands/current.rs` - Returns "Not yet implemented" error
- `src/commands/remove.rs` - Returns "Not yet implemented" error
- `src/commands/status.rs` - Returns "Not yet implemented" error

These stubs do not prevent the plan's goal (add command) from being achieved. They exist so the binary compiles with all subcommands registered.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- All core modules established and ready for list/use/current/remove/status implementations
- Atomic write pattern proven and reusable
- Test harness with CLMUX_HOME isolation ready for additional test files

## Self-Check: PASSED

All 11 key files verified present. Both task commits (a6f7068, 1c9aa60) verified in git history.

---
*Phase: 01-foundation-and-profile-management*
*Completed: 2026-04-03*
