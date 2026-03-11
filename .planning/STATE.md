---
gsd_state_version: 1.0
milestone: v1.5
milestone_name: milestone
current_phase: 08
current_plan: 03
status: in-progress
last_updated: "2026-03-11T16:30:00Z"
progress:
  total_phases: 8
  completed_phases: 7
  total_plans: 17
  completed_plans: 18
---

# Project State: snap-window

**Project:** snap-window - Cross-platform CLI window screenshot tool
**Core Value:** Users can reliably capture any visible window as a PNG image using simple CLI commands, regardless of operating system.
**Current Phase:** 08
**Current Plan:** 02
**Status:** In Progress
**Last Updated:** 2026-03-11

---

## Current Position

```
[██████████] 100% - Phase 4 complete (8/8 plans complete)

Phase 1: Foundation              [██████████] 100% - 3/3 plans complete
Phase 2: Window Discovery        [██████████] 100% - 2/2 plans complete
Phase 3: Window Targeting        [██████████] 100% - 1/1 plans complete
Phase 4: Screenshot Capture      [██████████] 100% - 2/2 plans complete
Phase 5: Highlight Mode          [██████████] 100% - 3/3 plans complete
Phase 6: Support Regexp          [██████████] 100% - 2/2 plans complete
Phase 7: Support Wayland         [██████████] 100% - 3/3 plans complete
Phase 8: Wayland Highlight Cleanup [██████░░░░] 67% - 2/3 plans complete
```

---

## Project Reference

**Tech Stack:**
- Language: Rust
- CLI: clap (derive macros)
- Cross-platform capture: xcap crate
- Windows: windows-rs v0.62+
- macOS: objc2-core-graphics v0.3.2
- Linux X11: x11rb v0.12+
- Linux Wayland: wayland-client v0.31, wayland-protocols-wlr v0.3
- Image: image v0.25.5
- Errors: anyhow/thiserror
- Testing: assert_cmd, predicates

**Key Decisions:**
- PNG only (no other image formats)
- CLI-only (no GUI)
- Highlight border excluded from screenshot
- JSON saves to same path as --output with .json extension
- --highlight is standalone mode

**Constraints:**
- Cross-platform: Windows, macOS, Linux (X11 primary, Wayland fallback)
- No runtime dependencies
- Long flags with optional short flags

---

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Window list latency | < 500ms | - |
| Screenshot capture | < 1s | - |
| Binary size | < 10MB | - |
| Platforms passing tests | 3/3 | - |
| Phase 05-highlight-mode P01 | 5 min | 1 tasks | 4 files |
| Phase 05-highlight-mode P02 | 25 min | 2 tasks | 7 files |
| Phase 05-highlight-mode P03 | 5 min | 2 tasks | 2 files |
| Phase 06-support-regexp-title P01 | 8 min | 3 tasks | 4 files |
| Phase 06-support-regexp-title P02 | 7 min | 2 tasks | 2 files |

### Execution Metrics

| Phase/Plan | Duration | Tasks | Files |
|------------|----------|-------|-------|
| Phase 02-window-discovery P02 | 3 min | 3 tasks | 4 files |
| Phase 03-window-targeting P01 | 4 min | 2 tasks | 4 files |
| Phase 04-screenshot-capture P01 | 5 min | 2 tasks | 4 files |
| Phase 04-screenshot-capture P02 | 5 min | 2 tasks | 2 files |

## Accumulated Context

### Decisions Made

| Date | Decision | Rationale |
|------|----------|-----------|
| 2026-03-09 | PNG only | Simpler implementation, PNG is lossless standard |
| 2026-03-09 | Highlight border excluded | Border is identification aid, not captured content |
| 2026-03-09 | --highlight standalone | Clear separation between identification and capture |
| 2026-03-10 | 5-phase roadmap | Fine granularity, natural delivery boundaries |
| 2026-03-10 | Clap derive with argument groups | Clean CLI with mutually exclusive modes enforced at parse time |
| 2026-03-10 | Mock window data for foundation | Enables CLI testing without platform-specific APIs |
| 2026-03-10 | Hybrid error handling (anyhow + thiserror) | anyhow for propagation, thiserror for domain errors |
| 2026-03-10 | Auto-list on lookup failures | Better UX - show available windows when target not found |
| 2026-03-10 | Timestamped default output paths | Compact YYYYMMDD_HHMMSS format for sortable filenames |
| 2026-03-10 | Platform stubs with cfg attributes | Conditional compilation for cross-platform support |
| 2026-03-10 | assert_cmd for CLI testing | Integration tests invoke actual binary for end-to-end verification |
| 2026-03-10 | platform_error() as semantic alias | Separate semantic constructor for platform API failures vs generic enumeration failures |
| 2026-03-10 | Determinism contract via test | Mock implementations validated as deterministic to establish contract for real platform code |
| 2026-03-10 | window_service as pure function library | Service functions accept &[WindowInfo] slices — testable without binary, no state |
| 2026-03-10 | find_by_index empty list guard | is_empty() check before computing max index prevents integer underflow panic |
| 2026-03-11 | xcap 0.9 builds on macOS | No fallback to 0.8 needed, 0.9 compiles and links cleanly |
| 2026-03-11 | ID correlation u32->u64 cast with title+pid fallback | Robust window matching across xcap and platform APIs |
| 2026-03-11 | Permission error keyword detection | Lowercase string matching for macOS Screen Recording errors |
| 2026-03-11 | Unified success message format | "Saved screenshot to: {path}" across all targeting arms |
| 2026-03-11 | Dual-outcome integration tests | Tests accept both success and graceful failure for headless CI |
| 2026-03-11 | Added objc2 v0.6 dependency | Required for MainThreadOnly trait access in highlight overlay |
| 2026-03-11 | Quartz to Cocoa coordinate conversion | Critical for correct NSWindow positioning on macOS |
| 2026-03-11 | Regex crate 1.12 for pattern matching | Industry standard, excellent performance |
| 2026-03-11 | Return Vec from find_by_regexp | Multiple matches possible; caller handles disambiguation |
| 2026-03-11 | User-controlled case sensitivity | (?i) flag gives users full control vs forced case-insensitive |
| 2026-03-11 | Separate empty match arm | Consistent UX - always show window list on "not found" |
| 2026-03-11 | Disambiguation to stderr | User-facing info belongs on stderr, not stdout |
| 2026-03-11 | Dual-outcome test pattern | Tests work in both headless CI and desktop environments |
| 2026-03-11 | Backend trait pattern for Linux | Enables runtime X11/Wayland selection; X11Backend implements LinuxBackend trait |
| 2026-03-11 | Prefer Wayland over X11 when both available | Handles XWayland case correctly; native Wayland preferred |
| 2026-03-11 | Foreign-toplevel protocol for Wayland enumeration | Silent window discovery without portal dialogs; works on wlroots compositors |
| 2026-03-11 | Geometry/PID unavailable in foreign-toplevel | Set to 0 - will be populated during capture phase if needed |
| 2026-03-11 | Wayland capture via XDG Desktop Portal | ashpd crate provides standard portal access; tokio runtime for async operations |
| 2026-03-11 | Portal Screenshot API for Wayland capture | Simpler than ScreenCast API; captures full screen (specific window requires PipeWire) |
| 2026-03-11 | LinuxBackend trait extended with capture_window | Unified interface for X11 and Wayland capture implementations |

### Open Questions

- Which platform to implement first? (Research suggests macOS for simpler permissions)
- Windows layered window capture fallback strategy
- Wayland portal behavior validation needed during Phase 5

### Known Blockers

None currently.

### Technical Debt

None currently.

### Roadmap Evolution

- Phase 6 added: support regexp title
- Phase 7 added: support wayland

---

## Session Continuity

**Last Action:** Completed plan 08-02 - implemented Wayland highlight using layer-shell protocol
**Next Action:** Continue with plan 08-03
**Context Valid Until:** 2026-03-12 (assumed)

### Key Files

- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/.planning/PROJECT.md` - Project context
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/.planning/REQUIREMENTS.md` - v1 requirements
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/.planning/ROADMAP.md` - Phase structure
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/.planning/research/SUMMARY.md` - Research findings

---

## Phase History

| Phase | Started | Completed | Notes |
|-------|---------|-----------|-------|
| 01-foundation | 2026-03-10 | - | Plan 01 complete: CLI foundation with mock window enumeration |
| 01-foundation | 2026-03-10 | - | Plan 02 complete: Error handling with anyhow/thiserror and dynamic defaults |
| 01-foundation | 2026-03-10 | - | Plan 03 complete: Cross-platform compilation with conditional compilation and integration tests |
| 03-window-targeting | 2026-03-10 | 2026-03-10 | Plan 01 complete: window_service module with case-insensitive matching, underflow guard, auto-list delegation |
| 04-screenshot-capture | 2026-03-11 | - | Plan 01 complete: capture_service with xcap 0.9, ID correlation, permission detection, 4 unit tests |
| 04-screenshot-capture | 2026-03-11 | 2026-03-11 | Plan 02 complete: wired capture_service into main.rs, 3 new CLI tests, 55 total tests |
| 05-highlight-mode | 2026-03-11 | 2026-03-11 | Plan 01 complete: json_export module with serde serialization |
| 05-highlight-mode | 2026-03-11 | 2026-03-11 | Plan 02 complete: highlight overlay for all platforms with 4-window border system |
| 05-highlight-mode | 2026-03-11 | 2026-03-11 | Plan 03 complete: highlight mode wired into main.rs with JSON export, 30 tests green |
| 06-support-regexp-title | 2026-03-11 | 2026-03-11 | Plan 01 complete: core regex support with regex crate, InvalidRegexPattern error, find_by_regexp function, --regexp CLI flag, 11 new tests |
| 06-support-regexp-title | 2026-03-11 | 2026-03-11 | Plan 02 complete: wired --regexp into main.rs with single/multiple match handling, disambiguation UI, 10 integration tests |
| 07-support-wayland | 2026-03-11 | - | Plan 01 complete: refactored Linux platform with backend trait pattern, runtime X11/Wayland detection, backward-compatible facade |
| 07-support-wayland | 2026-03-11 | - | Plan 02 complete: WaylandBackend with foreign-toplevel protocol, window enumeration on wlroots compositors |
| 07-support-wayland | 2026-03-11 | - | Plan 03 complete: Wayland screenshot capture via XDG Desktop Portal, LinuxBackend trait extended with capture_window |
| 08-wayland-highlight-cleanup | 2026-03-11 | - | Plan 01 complete: removed dead code (UnsupportedPlatform variant), fixed unnecessary unsafe blocks in macos.rs, zero targeted warnings |
| 08-wayland-highlight-cleanup | 2026-03-11 | - | Plan 02 complete: Wayland highlight using layer-shell protocol, red border overlay on wlroots compositors, click-through behavior |

---

*State file: Initialize at roadmap completion*
*Update after each plan/execution cycle*
