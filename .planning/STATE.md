# Project State: snap-window

**Project:** snap-window - Cross-platform CLI window screenshot tool
**Core Value:** Users can reliably capture any visible window as a PNG image using simple CLI commands, regardless of operating system.
**Current Phase:** 01-foundation
**Current Plan:** 01-03
**Status:** Plan 01-02 complete - Error handling with anyhow/thiserror and dynamic defaults
**Last Updated:** 2026-03-10

---

## Current Position

```
[░░░░░░░░░░] 5% - Phase 1 in progress

Phase 1: Foundation         [████░░░░░░] 67% - 2/3 plans complete
Phase 2: Window Discovery   [░░░░░░░░░░] 0% - Not started
Phase 3: Window Targeting   [░░░░░░░░░░] 0% - Not started
Phase 4: Screenshot Capture [░░░░░░░░░░] 0% - Not started
Phase 5: Highlight Mode     [░░░░░░░░░░] 0% - Not started
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
- Image: image v0.25.5
- Errors: anyhow/thiserror

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

---

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

### Open Questions

- Which platform to implement first? (Research suggests macOS for simpler permissions)
- Windows layered window capture fallback strategy
- Wayland portal behavior validation needed during Phase 5

### Known Blockers

None currently.

### Technical Debt

None currently.

---

## Session Continuity

**Last Action:** Completed plan 01-02 - Error handling with anyhow/thiserror and dynamic defaults
**Next Action:** Execute plan 01-03
**Context Valid Until:** 2026-03-11 (assumed)

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

---

*State file: Initialize at roadmap completion*
*Update after each plan/execution cycle*
