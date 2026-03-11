---
phase: 04-screenshot-capture
plan: 01
subsystem: capture
tags: [xcap, png, screenshot, image-capture, macos-permissions]

# Dependency graph
requires:
  - phase: 03-window-targeting
    provides: WindowInfo struct with window_id for xcap correlation
  - phase: 01-foundation
    provides: AppError with thiserror, anyhow error propagation
provides:
  - capture_window() function — WindowInfo + Path -> Result<()>
  - AppError::CaptureFailed variant for capture errors
  - AppError::PermissionDenied variant for macOS Screen Recording
affects: [04-02-main-wiring, 05-highlight-mode]

# Tech tracking
tech-stack:
  added: [xcap 0.9]
  patterns: [xcap-window-correlation, permission-error-detection]

key-files:
  created: [src/capture_service.rs]
  modified: [src/error.rs, src/lib.rs, Cargo.toml]

key-decisions:
  - "xcap 0.9 builds successfully on macOS — no fallback to 0.8 needed"
  - "ID correlation with u32->u64 cast plus title+pid fallback for robustness"
  - "Permission error detection via keyword matching in xcap error messages"

patterns-established:
  - "capture_service as pure function library: capture_window(&WindowInfo, &Path) -> Result<()>"
  - "Permission error detection: lowercase error string keyword matching for macOS Screen Recording"

requirements-completed: [CAP-01, CAP-02, CAP-04, ERR-04]

# Metrics
duration: 5min
completed: 2026-03-11
---

# Phase 4 Plan 01: Capture Service Summary

**capture_window() pure function with xcap 0.9 integration, ID+title+pid correlation, permission error detection, and 4 unit tests**

## Performance

- **Duration:** 5 min
- **Started:** 2026-03-11T11:34:08Z
- **Completed:** 2026-03-11T11:39:08Z
- **Tasks:** 2 (TDD RED + GREEN)
- **Files modified:** 4

## Accomplishments
- Added xcap 0.9 dependency with successful build on macOS
- Extended AppError with CaptureFailed and PermissionDenied variants with actionable messages
- Implemented capture_window() with xcap Window::all() enumeration, ID correlation (u32->u64 cast), title+pid fallback
- Minimized window detection, permission error keyword matching, parent directory creation
- 4 new unit tests, all 29 lib tests pass

## Task Commits

Each task was committed atomically:

1. **Task 1: RED - failing tests + error variants + xcap dep** - `b550143` (test)
2. **Task 2: GREEN - implement capture_window** - `1218181` (feat)

_TDD plan: RED wrote failing tests with todo!() stub, GREEN implemented the function._

## Files Created/Modified
- `src/capture_service.rs` - New module: capture_window() with xcap integration and unit tests
- `src/error.rs` - Added CaptureFailed and PermissionDenied AppError variants with constructors
- `src/lib.rs` - Registered pub mod capture_service
- `Cargo.toml` - Added xcap = "0.9" dependency

## Decisions Made
- xcap 0.9 builds cleanly on macOS, no fallback needed
- ID correlation uses u32->u64 cast with title+pid fallback for cases where xcap IDs differ
- Permission detection via lowercase keyword matching ("permission", "screen recording", "not permitted", "access denied")

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- capture_service module ready for main.rs wiring in Plan 02
- All existing tests remain green (29/29)
- capture_window() exported and available via snap_window::capture_service::capture_window

---
*Phase: 04-screenshot-capture*
*Completed: 2026-03-11*
