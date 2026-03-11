---
phase: 07-support-wayland
plan: 03
type: summary
subsystem: platform
tags: [wayland, capture, portal, ashpd, xdg-desktop-portal]
dependencies:
  requires: [07-02]
  provides: []
  affects: [capture_service, error_handling]
tech-stack:
  added: [ashpd 0.13, tokio 1.40]
  patterns: [async-runtime-block_on, portal-error-mapping]
key-files:
  created: []
  modified:
    - Cargo.toml
    - src/error.rs
    - src/platform/linux/mod.rs
    - src/platform/linux/x11.rs
    - src/platform/linux/wayland.rs
    - src/capture_service.rs
decisions:
  - Use ashpd 0.13 for XDG Desktop Portal access (industry standard)
  - Use tokio::runtime::block_on for isolated async portal operations
  - Portal Screenshot API captures full screen (specific window selection not available)
  - LinuxBackend trait extended with capture_window method for unified interface
  - X11Backend implements capture_window using xcap (existing logic)
  - WaylandBackend implements capture_window using portal Screenshot API
  - capture_service delegates to backend abstraction on Linux
metrics:
  duration: 15
  completed_date: 2026-03-11
---

# Phase 07 Plan 03: Wayland Screenshot Capture Summary

## One-Liner

Implemented Wayland screenshot capture using XDG Desktop Portal (ashpd), extended LinuxBackend trait with capture_window method, and refactored capture_service to use backend abstraction on Linux.

## What Was Built

### Core Functionality

- **Wayland Portal Capture**: WaylandBackend now implements `capture_window()` using the XDG Desktop Portal Screenshot API via ashpd
- **Async Runtime Integration**: Uses tokio runtime with `block_on()` for isolated async portal operations within synchronous codebase
- **Unified Backend Interface**: Extended `LinuxBackend` trait with `capture_window()` method, implemented by both X11Backend and WaylandBackend
- **Refactored Capture Service**: capture_service now delegates to platform backend on Linux, maintaining xcap for other platforms

### Error Handling

- **Portal-Specific Errors**: Added `PortalNotAvailable` and `PortalPermissionDenied` error variants
- **Error Mapping**: `map_portal_error()` converts ashpd errors to user-friendly AppError variants
- **Actionable Messages**: Portal errors include clear instructions for installing xdg-desktop-portal or granting permissions

### Dependencies Added

- `ashpd = { version = "0.13", features = ["tokio"] }` - XDG Desktop Portal D-Bus wrapper
- `tokio = { version = "1.40", features = ["rt-multi-thread"] }` - Async runtime for portal operations

## Files Modified

| File | Changes |
|------|---------|
| `Cargo.toml` | Added ashpd and tokio dependencies for Linux |
| `src/error.rs` | Added PortalNotAvailable and PortalPermissionDenied variants; added tests |
| `src/platform/linux/mod.rs` | Extended LinuxBackend trait with capture_window method |
| `src/platform/linux/x11.rs` | Implemented capture_window using xcap; added Path import |
| `src/platform/linux/wayland.rs` | Implemented capture_window using portal; added error mapping |
| `src/capture_service.rs` | Refactored to use backend abstraction on Linux; extracted capture_with_xcap |

## Commits

| Hash | Message |
|------|---------|
| `8f467d3` | feat(07-03): add ashpd dependency and Wayland portal capture |
| `95c1363` | feat(07-03): integrate Wayland capture into capture_service |
| `2c7daae` | test(07-03): add portal error variant tests |

## Test Results

- **Unit tests**: 53 passed
- **Integration tests**: 29 passed (10 regex + 19 CLI)
- **Platform tests**: 9 passed
- **Doc tests**: 2 passed
- **Total**: 93 tests passing

## Technical Details

### Portal Capture Flow

1. Create tokio runtime for async operation
2. Connect to XDG Desktop Portal via ashpd
3. Request screenshot with non-interactive options
4. Portal saves image to temp location (returns URI)
5. Copy from temp location to user-requested output path
6. Clean up portal's temp file

### Backend Abstraction

```rust
pub trait LinuxBackend {
    fn list_windows(&self) -> Result<Vec<WindowInfo>>;
    fn show_highlight_border(&self, info: &WindowInfo) -> Result<()>;
    fn capture_window(&self, info: &WindowInfo, output_path: &Path) -> Result<()>;
}
```

### Platform Detection in capture_service

```rust
pub fn capture_window(info: &WindowInfo, output: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        use crate::platform::linux::{create_backend, LinuxBackend};
        let backend = create_backend()?;
        return backend.capture_window(info, output);
    }

    #[cfg(not(target_os = "linux"))]
    {
        capture_with_xcap(info, output)
    }
}
```

## Limitations

- **Screenshot API Limitation**: XDG Desktop Portal Screenshot API captures the entire screen, not a specific window by ID. This is a security design decision in Wayland. For specific window capture, the ScreenCast API with PipeWire would be required (more complex implementation).
- **Portal Requirement**: Wayland capture requires xdg-desktop-portal and a backend implementation (xdg-desktop-portal-gtk, xdg-desktop-portal-kde, etc.) to be installed and running.
- **User Interaction**: First screenshot may trigger a permission dialog depending on portal configuration.

## Deviation from Plan

None - plan executed exactly as written.

## Verification

- [x] ashpd dependency added and compiles
- [x] WaylandBackend::capture_window() uses portal API
- [x] LinuxBackend trait includes capture_window method
- [x] X11Backend implements capture_window using xcap
- [x] capture_service uses backend abstraction for Linux
- [x] Portal-specific error handling with clear messages
- [x] Full test suite passes (93 tests)

## Self-Check: PASSED

All modified files verified:
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/Cargo.toml` - ashpd and tokio added
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/src/error.rs` - portal errors added
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/src/platform/linux/mod.rs` - trait updated
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/src/platform/linux/x11.rs` - capture_window implemented
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/src/platform/linux/wayland.rs` - portal capture implemented
- `/Users/maxilo202/Downloads/code/_TEST/snap-windows/src/capture_service.rs` - backend abstraction

All commits verified:
- `8f467d3` - feat(07-03): add ashpd dependency and Wayland portal capture
- `95c1363` - feat(07-03): integrate Wayland capture into capture_service
- `2c7daae` - test(07-03): add portal error variant tests
