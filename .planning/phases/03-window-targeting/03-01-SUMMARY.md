---
phase: 03-window-targeting
plan: 01
subsystem: targeting
tags: [rust, window-service, case-insensitive, tdd, unit-tests, refactoring]

# Dependency graph
requires:
  - phase: 02-window-discovery
    provides: platform::list_windows(), WindowInfo struct, AppError types
  - phase: 01-foundation
    provides: CLI structure (Cli, Mode), error handling (anyhow/thiserror), platform stubs
provides:
  - window_service module with find_by_name (case-insensitive), find_by_pid, find_by_index, print_available_windows
  - 10 unit tests covering WIN-02/03/04/05 and ERR-02 behaviors
  - main.rs targeting arms delegating to window_service
  - Integration tests asserting auto-list on all not-found error paths
affects: [04-screenshot-capture, 05-highlight-mode]

# Tech tracking
tech-stack:
  added: []
  patterns: [service module pattern, TDD with inline unit tests, delegation over inline logic]

key-files:
  created:
    - src/window_service.rs
  modified:
    - src/lib.rs
    - src/main.rs
    - tests/cli_tests.rs

key-decisions:
  - "window_service module as pure function library — no state, all functions take &[WindowInfo] slices"
  - "find_by_name checks both title and app_name for maximum match coverage"
  - "find_by_index guards empty list with is_empty() before computing max index to prevent underflow"
  - "print_available_windows uses eprintln! exclusively to keep auto-list on stderr"

patterns-established:
  - "Targeting functions: fn find_by_*(windows: &[WindowInfo], key: T) -> Result<&WindowInfo, AppError>"
  - "Error display: print_available_windows() called before returning Err from targeting arms"
  - "Case-insensitive matching: needle.to_lowercase().contains(&hayfield.to_lowercase())"

requirements-completed: [WIN-02, WIN-03, WIN-04, WIN-05, ERR-02]

# Metrics
duration: 4min
completed: 2026-03-10
---

# Phase 3 Plan 01: Window Targeting Summary

**Extracted window-targeting logic into window_service module with case-insensitive name matching, empty-list underflow guard, and unified auto-list error display — all three targeting arms in main.rs now delegate to tested service functions.**

## Performance

- **Duration:** ~4 min
- **Started:** 2026-03-10T13:49:51Z
- **Completed:** 2026-03-10T13:52:58Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Created `src/window_service.rs` with four public functions and 10 unit tests (TDD)
- Fixed case-sensitive name match bug (WIN-02): `w.title.contains(&name)` replaced with `.to_lowercase().contains()`
- Eliminated three duplicated `eprintln!` auto-list blocks — replaced by single `print_available_windows()` call
- Fixed integer underflow panic risk on empty window lists in `find_by_index`
- All 13 integration tests pass including 4 new/strengthened "Available windows" stderr assertions

## Task Commits

Each task was committed atomically:

1. **Task 1: Create window_service module with unit tests** - `07d1307` (test)
2. **Task 2: Refactor main.rs to delegate targeting and update integration tests** - `070baf6` (feat)

**Plan metadata:** (docs commit, see below)

_Note: TDD task committed as single commit since tests and implementation are co-located in window_service.rs_

## Files Created/Modified
- `src/window_service.rs` - New module: find_by_name, find_by_pid, find_by_index, print_available_windows + 10 unit tests
- `src/lib.rs` - Added `pub mod window_service` declaration
- `src/main.rs` - All three targeting arms refactored to delegate to window_service functions
- `tests/cli_tests.rs` - test_invalid_window, test_invalid_pid, test_invalid_index strengthened with "Available windows" stderr assertions; test_window_flag_case_insensitive_not_found added

## Decisions Made
- `window_service` functions accept `&[WindowInfo]` slices rather than owning the list, keeping them pure and testable
- `find_by_name` checks both `title` and `app_name` to maximize match coverage (matching plan spec)
- `print_available_windows` uses `eprintln!` (not `println!`) so auto-list always goes to stderr, never stdout

## Deviations from Plan

None - plan executed exactly as written. The plan specified 10 test cases; the initial implementation had 9 (missing `find_by_pid(1002)` for "Code - main.rs"). The 10th test was added before the Task 1 commit to match the behavior spec.

## Issues Encountered

None — all compilation and tests passed on first run.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- window_service module provides the targeting foundation needed by Phase 4 (screenshot capture)
- All targeting arms in main.rs are clean delegation points ready for screenshot integration
- No blockers for Phase 4 development
## Self-Check: PASSED

- src/window_service.rs: FOUND
- 03-01-SUMMARY.md: FOUND
- Commit 07d1307 (Task 1): FOUND
- Commit 070baf6 (Task 2): FOUND

---
*Phase: 03-window-targeting*
*Completed: 2026-03-10*
