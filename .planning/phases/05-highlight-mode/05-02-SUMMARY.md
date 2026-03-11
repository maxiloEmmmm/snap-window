---
phase: 05-highlight-mode
plan: 02
subsystem: ui

# Dependency graph
requires:
  - phase: 05-01
    provides: WindowInfo struct and platform enumeration
provides:
  - show_highlight_border() for macOS, Windows, Linux
  - highlight_service orchestration module
  - 4-overlay red border highlight system
affects:
  - 05-03 (CLI wiring for --highlight flag)

# Tech tracking
tech-stack:
  added:
    - objc2 (v0.6) for macOS MainThreadOnly trait
    - objc2-app-kit NSWindow overlay API
    - Win32_Graphics_Gdi for Windows red brush
  patterns:
    - Platform-specific overlay windows (not drawing on target)
    - Coordinate system conversion (Quartz to Cocoa)
    - MainThreadMarker for macOS UI thread safety

key-files:
  created:
    - src/highlight_service.rs - Orchestration layer
  modified:
    - src/platform/macos.rs - NSWindow overlay implementation
    - src/platform/windows.rs - CreateWindowExW overlay
    - src/platform/linux.rs - X11 window overlay
    - src/platform/mod.rs - Re-export show_highlight_border
    - src/lib.rs - Register highlight_service module
    - Cargo.toml - Add objc2 and Win32_Graphics_Gdi

key-decisions:
  - "Added direct objc2 dependency (v0.6) to access MainThreadOnly trait"
  - "Used 4 separate overlay windows to ensure highlight never appears in screenshots (HIL-02)"
  - "Applied Quartz to Cocoa coordinate conversion for correct macOS positioning"

requirements-completed: [HIL-01, HIL-02]

# Metrics
duration: 25min
completed: 2026-03-11
---

# Phase 5 Plan 2: Window Highlight Overlay Summary

**Platform-specific red border highlight using 4 separate overlay windows per platform, with Quartz-to-Cocoa coordinate conversion and thread-safe macOS implementation**

## Performance

- **Duration:** 25 min
- **Started:** 2026-03-11T13:24:07Z
- **Completed:** 2026-03-11T13:49:00Z
- **Tasks:** 2
- **Files modified:** 7

## Accomplishments

- Implemented `show_highlight_border()` for all 3 platforms (macOS, Windows, Linux)
- Created `highlight_service` orchestration module with index validation
- 4-overlay window system ensures highlight border never appears in screenshots
- macOS: NSWindow with Quartz-to-Cocoa coordinate conversion
- Windows: CreateWindowExW with WS_EX_TOPMOST and red GDI brush
- Linux: X11 windows with _NET_WM_STATE_ABOVE and _NET_WM_WINDOW_TYPE_NOTIFICATION

## Task Commits

Each task was committed atomically:

1. **Task 1: Add platform overlay dependencies and implement show_highlight_border** - `27d5a03` (feat)
2. **Task 2: Create highlight_service orchestration module** - `cb025d0` (feat)

## Files Created/Modified

- `src/highlight_service.rs` - Orchestration layer with highlight_window() function
- `src/platform/macos.rs` - NSWindow overlay with coordinate conversion
- `src/platform/windows.rs` - CreateWindowExW overlay implementation
- `src/platform/linux.rs` - X11 window overlay implementation
- `src/platform/mod.rs` - Re-export show_highlight_border for all platforms
- `src/lib.rs` - Register highlight_service module
- `Cargo.toml` - Add objc2 v0.6 and Win32_Graphics_Gdi feature

## Decisions Made

- **Added objc2 v0.6 dependency directly**: Required to access MainThreadOnly trait for NSWindow::alloc(). The objc2-app-kit 0.3 depends on objc2 but doesn't re-export this trait.
- **Used 4 separate overlay windows**: Top, bottom, left, right rectangles forming a border frame. This ensures the highlight is never drawn on the target window surface, satisfying HIL-02 requirement.
- **Quartz to Cocoa coordinate conversion**: macOS uses two coordinate systems - Core Graphics (top-left origin) vs AppKit (bottom-left origin). Implemented proper conversion using screen height.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added direct objc2 dependency for MainThreadOnly trait**
- **Found during:** Task 1 (macOS implementation)
- **Issue:** NSWindow::alloc() requires MainThreadOnly trait in scope, but objc2-app-kit 0.3 doesn't re-export it from its objc2 dependency
- **Fix:** Added `objc2 = "0.6"` to Cargo.toml macOS dependencies and imported `MainThreadOnly` trait
- **Files modified:** Cargo.toml, src/platform/macos.rs
- **Verification:** Build succeeds, all 55 tests pass
- **Committed in:** 27d5a03 (Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary dependency addition for macOS compilation. No scope creep.

## Issues Encountered

- **objc2 version mismatch**: The project had objc2 0.6.4 in Cargo.lock but objc2-app-kit 0.3.2 expects objc2 0.3.x APIs. Resolved by using the 0.6 API (MainThreadOnly at crate root) which is what was actually available.
- **MainThreadMarker requirement**: NSScreen::mainScreen() and NSWindow::alloc() require MainThreadMarker in objc2 0.6. Added proper marker creation and propagation.

## Next Phase Readiness

- highlight_service is ready for CLI wiring
- Next: Implement --highlight flag in CLI and wire to highlight_service
- No blockers

---
*Phase: 05-highlight-mode*
*Completed: 2026-03-11*
