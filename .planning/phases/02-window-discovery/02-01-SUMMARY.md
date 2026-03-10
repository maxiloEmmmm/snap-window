---
phase: 02-window-discovery
plan: 01
subsystem: platform
tags: [rust, windows-rs, objc2-core-graphics, x11rb, platform, window-enumeration]

# Dependency graph
requires:
  - phase: 01-foundation
    provides: Platform module stubs, WindowInfo struct, AppError types
provides:
  - Real window enumeration on macOS via CGWindowListCopyWindowInfo
  - Real window enumeration on Windows via EnumWindows + Win32 API
  - Real window enumeration on Linux via x11rb _NET_CLIENT_LIST
  - snap-window --list shows actual system windows (not mock data)
affects: [03-window-targeting, 04-screenshot-capture, 05-highlight-mode]

# Tech tracking
tech-stack:
  added: [windows-rs 0.62, objc2-core-graphics 0.3.2, objc2-core-foundation 0.3.2, x11rb 0.13]
  patterns: [cfg-based platform dispatch, unsafe extern static access, EWMH atom interning]

key-files:
  created: []
  modified:
    - src/platform/windows.rs
    - src/platform/macos.rs
    - src/platform/linux.rs
    - Cargo.toml

key-decisions:
  - "macOS extern statics (kCGWindowLayer etc.) require unsafe {} wrapper — wrapped at call site, not in helper signature"
  - "Linux uses _NET_CLIENT_LIST (EWMH) — requires compliant WM; windows without titles are skipped"
  - "Windows EnumWindows callback uses Arc<Mutex<Vec>> to safely collect across callback boundary"

patterns-established:
  - "macOS: take &extern_static inside unsafe block at each call site"
  - "Linux: intern atoms once before loop, reuse atom IDs for all windows"
  - "All platforms: sort by app_name then title, assign sequential indices after sort"

requirements-completed: [WIN-01]

# Metrics
duration: ~15min
completed: "2026-03-10"
---

# Phase 02-01: Window Discovery Summary

**Native window enumeration on macOS (CGWindowListCopyWindowInfo), Windows (EnumWindows), and Linux (x11rb _NET_CLIENT_LIST) — `snap-window --list` now shows real system windows**

## Performance

- **Duration:** ~15 min
- **Completed:** 2026-03-10
- **Tasks:** 4
- **Files modified:** 4

## Accomplishments
- macOS: CGWindowListCopyWindowInfo with layer-0 filtering, extracts title/PID/app/bounds from CFDictionary
- Windows: EnumWindows with IsWindowVisible + non-empty title filter, QueryFullProcessImageNameW for app name
- Linux: x11rb via _NET_CLIENT_LIST, _NET_WM_NAME/_NET_WM_PID/WM_CLASS/translate_coordinates
- All platforms sort consistently (app_name → title) with sequential 0-based indices

## Task Commits

1. **Task 1: Platform dependencies** - `db2bf03` (chore: add platform-specific deps to Cargo.toml)
2. **Task 2: Windows implementation** - `f5b2916` (feat: implement Windows EnumWindows enumeration)
3. **Task 3: macOS implementation** - `562fbf0` (fix: wrap extern static accesses in unsafe blocks)
4. **Task 4: Linux X11 implementation** - `9ff9be4` (feat: implement Linux X11 via x11rb)

## Files Created/Modified
- `src/platform/windows.rs` - EnumWindows-based enumeration with Win32 process name lookup
- `src/platform/macos.rs` - CGWindowListCopyWindowInfo with CFDictionary key extraction
- `src/platform/linux.rs` - x11rb EWMH enumeration with atom caching and geometry translation
- `Cargo.toml` - Platform-conditional dependency sections

## Decisions Made
- Wrapped extern static accesses (`kCGWindowLayer` etc.) in `unsafe {}` at call sites — Rust requires this for foreign statics
- Linux uses `_NET_CLIENT_LIST` from root window (EWMH standard) rather than XQueryTree — simpler and returns only managed client windows

## Deviations from Plan

### Auto-fixed Issues

**1. macOS unsafe extern static access**
- **Found during:** Task 3 (macOS implementation)
- **Issue:** `kCGWindowLayer`, `kCGWindowName` etc. are `extern static` values requiring `unsafe` to reference — missing from research pattern
- **Fix:** Wrapped all `&kCGWindowStatic` references in `unsafe {}` blocks at call sites; removed now-unused imports
- **Files modified:** src/platform/macos.rs
- **Verification:** `cargo build` passes with no errors
- **Committed in:** `562fbf0`

---

**Total deviations:** 1 auto-fixed (unsafe extern static wrapping)
**Impact on plan:** Required for correctness on newer Rust editions. No scope creep.

## Issues Encountered
- Agent hit permission restrictions before completing Task 4 (Linux) and creating SUMMARY.md — orchestrator completed both

## Next Phase Readiness
- Window enumeration functional on all three platforms
- `snap-window --list` returns real system windows with index, title, PID, app name
- Phase 3 (Window Targeting) can use list_windows() output directly for lookup by name/PID/index

---
*Phase: 02-window-discovery*
*Completed: 2026-03-10*
