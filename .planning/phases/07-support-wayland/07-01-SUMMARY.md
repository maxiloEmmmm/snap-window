---
phase: 07-support-wayland
plan: 01
type: execute
subsystem: platform
tags: [linux, wayland, x11, backend-trait, runtime-detection]
dependency_graph:
  requires: []
  provides: [LIN-02]
  affects: [src/platform/linux/*]
tech_stack:
  added: []
  patterns: [backend-trait, factory-pattern, runtime-detection]
key_files:
  created:
    - src/platform/linux/mod.rs
    - src/platform/linux/detector.rs
    - src/platform/linux/x11.rs
  modified:
    - src/platform/linux.rs
    - src/platform/linux/mod.rs (from linux.rs)
decisions:
  - "Use trait-based backend pattern (LinuxBackend) for extensibility"
  - "Prefer Wayland when both DISPLAY and WAYLAND_DISPLAY are set (XWayland case)"
  - "Maintain backward compatibility via facade pattern in linux.rs"
  - "XWayland fallback when native Wayland not yet implemented"
metrics:
  duration_minutes: 18
  completed_at: "2026-03-11T15:51:00Z"
  tasks_total: 3
  tasks_completed: 3
---

# Phase 07 Plan 01: Linux Backend Refactor for Wayland Support

**One-liner:** Refactored Linux platform module with backend trait pattern and runtime X11/Wayland detection, maintaining full backward compatibility.

---

## What Was Built

### Architecture Overview

Created a modular Linux platform structure supporting runtime detection between X11 and Wayland display servers:

```
src/platform/linux/
├── mod.rs       # LinuxBackend trait, create_backend() factory
├── detector.rs  # DisplayServer enum, detect_display_server()
└── x11.rs       # X11Backend implementation
```

The existing `src/platform/linux.rs` was converted to a backward-compatible facade that delegates to the new backend system.

### Key Components

1. **DisplayServer Detection** (`detector.rs`)
   - `DisplayServer` enum: X11, Wayland, Unknown
   - `detect_display_server()`: Checks WAYLAND_DISPLAY, DISPLAY, and Wayland socket fallback
   - Prefers Wayland when both env vars are set (handles XWayland case)

2. **Backend Trait** (`mod.rs`)
   - `LinuxBackend` trait with `list_windows()` and `show_highlight_border()`
   - `create_backend()`: Factory function returning `Box<dyn LinuxBackend>`
   - Object-safe trait design for runtime dispatch

3. **X11 Backend** (`x11.rs`)
   - `X11Backend` struct holding `RustConnection` and `screen_num`
   - Full implementation of `LinuxBackend` trait
   - All original X11 logic preserved (window enumeration, highlight borders)

4. **Backward Compatibility** (`linux.rs` facade)
   - `list_windows()` and `show_highlight_border()` delegate to `create_backend()`
   - Existing `platform/mod.rs` imports work unchanged
   - No breaking changes to public API

---

## Deviations from Plan

None - plan executed exactly as written.

---

## Implementation Details

### Detection Logic (per RESEARCH.md Pattern 1)

```rust
match (wayland_display, x11_display) {
    (Ok(_), Ok(_)) => DisplayServer::Wayland,  // Prefer Wayland in XWayland case
    (Ok(_), Err(_)) => DisplayServer::Wayland,
    (Err(_), Ok(_)) => DisplayServer::X11,
    (Err(_), Err(_)) => check_wayland_socket_fallback(),
}
```

### Backend Factory

```rust
pub fn create_backend() -> Result<Box<dyn LinuxBackend>> {
    match detect_display_server() {
        DisplayServer::X11 => Ok(Box::new(X11Backend::new()?)),
        DisplayServer::Wayland => {
            // Try XWayland fallback until native Wayland is implemented
            match X11Backend::new() {
                Ok(backend) => Ok(Box::new(backend)),
                Err(_) => Err("Wayland native backend not yet implemented".into())
            }
        }
        DisplayServer::Unknown => Err("No supported display server found".into()),
    }
}
```

---

## Test Coverage

### Unit Tests (detector.rs)
- `test_detect_x11_only`: Returns X11 when only DISPLAY is set
- `test_detect_wayland_only`: Returns Wayland when only WAYLAND_DISPLAY is set
- `test_detect_prefers_wayland_when_both_set`: Prefers Wayland in XWayland case
- `test_detect_unknown_when_neither_set`: Returns Unknown when no display server detected
- `test_display_server_debug`: Debug formatting for all variants
- `test_display_server_clone_copy`: Clone and Copy trait implementations
- `test_display_server_equality`: PartialEq implementation

### Unit Tests (mod.rs)
- `test_linux_backend_object_safe`: Trait object safety verification
- `test_create_backend_unknown_display_server`: Error handling when no display server
- `test_display_server_variants`: Enum variant accessibility

### Unit Tests (x11.rs)
- `test_x11_backend_implements_trait`: LinuxBackend trait implementation
- `test_x11_backend_new_fails_when_unavailable`: Graceful failure when X11 unavailable
- `test_x11_backend_as_trait_object`: Trait object compatibility

### Integration
- All 79 existing tests pass without modification
- Backward compatibility confirmed via `platform/mod.rs` re-exports
- Cross-platform compilation verified (cfg guards work correctly)

---

## Files Changed

| File | Lines | Change |
|------|-------|--------|
| `src/platform/linux/detector.rs` | +180 | New file - DisplayServer detection |
| `src/platform/linux/mod.rs` | +142 | New file - LinuxBackend trait and factory |
| `src/platform/linux/x11.rs` | +385 | New file - X11Backend implementation |
| `src/platform/linux.rs` | +81/-376 | Refactored to facade |

**Net change:** +412 lines across 4 files

---

## Verification

- [x] Runtime detection works for X11, Wayland, and Unknown cases
- [x] X11 backend functions identically to pre-refactor implementation
- [x] LinuxBackend trait properly defined and implemented
- [x] All existing tests pass (79 tests)
- [x] Module structure follows RESEARCH.md recommended architecture
- [x] No breaking changes to platform API
- [x] Backward compatibility maintained

---

## Auth Gates

None encountered.

---

## Commits

| Hash | Message |
|------|---------|
| 32e4cc6 | feat(07-01): create Linux backend module structure and detector |
| 6051123 | feat(07-01): extract X11 implementation into backend module |

---

## Next Steps

Plan 07-02 will implement the native Wayland backend using the foreign-toplevel protocol for window enumeration and XDG Desktop Portal for screenshot capture. The trait structure is now in place to add `WaylandBackend` as a third variant in `create_backend()`.

---

## Self-Check: PASSED

- [x] All created files exist: detector.rs, mod.rs, x11.rs
- [x] Modified file updated: linux.rs (facade)
- [x] Commits exist: 32e4cc6, 6051123
- [x] All tests pass: 79/79
- [x] Code compiles without errors
- [x] Backward compatibility verified
