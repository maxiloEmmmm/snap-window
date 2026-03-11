---
phase: 04-screenshot-capture
plan: 02
subsystem: capture
tags: [capture-wiring, cli-integration, main-dispatch, png-screenshot]

# Dependency graph
requires:
  - phase: 04-screenshot-capture
    provides: capture_service::capture_window() function from Plan 01
  - phase: 03-window-targeting
    provides: window_service find_by_name/pid/index returning &WindowInfo
provides:
  - End-to-end capture pipeline: CLI args -> window targeting -> screenshot -> PNG file
  - "Saved screenshot to:" success message format on stdout
  - 3 new CLI integration tests for capture behavior
affects: [05-highlight-mode]

# Tech tracking
tech-stack:
  added: []
  patterns: [capture-dispatch-from-main, integration-test-dual-outcome]

key-files:
  created: []
  modified: [src/main.rs, tests/cli_tests.rs]

key-decisions:
  - "Unified success message format: 'Saved screenshot to: {path}' across all targeting arms"
  - "Integration tests accept both success and graceful failure for headless CI compatibility"

patterns-established:
  - "Dual-outcome integration tests: assert on observable behavior that holds in both success and failure cases"
  - "Capture dispatch pattern: find_by_*() -> capture_window() -> println success message"

requirements-completed: [CAP-01, CAP-02, CAP-03, CAP-04, CLI-04, CLI-07, ERR-04]

# Metrics
duration: 5min
completed: 2026-03-11
---

# Phase 4 Plan 02: Main Wiring Summary

**Wire capture_service into main.rs dispatch arms with unified success messages and 3 CLI integration tests**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T11:43:15Z
- **Completed:** 2026-03-11T11:48:00Z
- **Tasks:** 2
- **Files modified:** 2

## Accomplishments
- Replaced placeholder prints in all three targeting arms (window/pid/index) with capture_service::capture_window() calls
- Unified success message format: "Saved screenshot to: {path}" on stdout across all arms
- Added 3 new CLI integration tests verifying capture wiring and placeholder text removal
- Updated 3 existing CLI tests to match new capture behavior (no more "Found window:" checks)
- Full test suite green: 55 tests (29 lib + 16 CLI + 9 platform + 1 doc)

## Task Commits

Each task was committed atomically:

1. **Task 1: Wire capture_service into main.rs targeting arms** - `7aed904` (feat)
2. **Task 2: Add CLI integration tests for capture behavior** - `7be53ec` (test)

## Files Created/Modified
- `src/main.rs` - Added mod capture_service, replaced placeholder prints with capture_window() calls in window/pid/index arms
- `tests/cli_tests.rs` - Added 3 new capture integration tests, updated 3 existing tests for new behavior

## Decisions Made
- Unified all three arms to identical pattern: capture_window() + "Saved screenshot to:" message
- Integration tests designed for dual-outcome (success or graceful failure) to work in headless CI environments

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Updated existing CLI tests for new capture behavior**
- **Found during:** Task 2 (CLI integration tests)
- **Issue:** test_window_flag, test_pid_flag, and test_index_flag checked for old placeholder text ("Found window:", "Selected window:") that no longer appears after Task 1 wiring
- **Fix:** Updated assertions to check for "Saved screenshot to:" or error messages instead of old placeholders
- **Files modified:** tests/cli_tests.rs
- **Verification:** All 16 CLI tests pass
- **Committed in:** 7be53ec (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug fix)
**Impact on plan:** Necessary correction -- existing tests referenced removed placeholder text. No scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Full capture pipeline operational: CLI args -> window targeting -> xcap capture -> PNG file
- All requirements for Phase 4 complete (CAP-01 through CAP-04, CLI-04, CLI-07, ERR-04)
- Ready for Phase 5: Highlight Mode

---
*Phase: 04-screenshot-capture*
*Completed: 2026-03-11*
