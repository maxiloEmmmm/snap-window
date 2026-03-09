# snap-window

## What This Is

A Rust CLI tool for capturing screenshots of specific application windows across Windows, macOS, and Linux (X11/Wayland). Users can target windows by name, PID, or index from a list. Includes a highlight mode for visual window identification.

## Core Value

Users can reliably capture any visible window as a PNG image using simple CLI commands, regardless of operating system.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Cross-platform window enumeration (list all windows with indices)
- [ ] Window targeting by name substring match
- [ ] Window targeting by PID
- [ ] Window targeting by index from list
- [ ] Screenshot capture and save as PNG
- [ ] Configurable output path for screenshots
- [ ] Default output path with timestamp
- [ ] Visual highlight mode (red border) for window identification
- [ ] Window info export as JSON (highlight mode only)
- [ ] Platform-specific window attributes in JSON
- [ ] Graceful handling when window not found (auto-list)
- [ ] Graceful handling on unsupported platforms

### Out of Scope

- Multiple image formats (PNG only) — keep implementation focused
- Screenshots without a target window (full screen capture) — out of scope for v1
- Real-time video capture — different tool category
- GUI interface — CLI-only by design
- Image editing/annotation — border is temporary visual aid only

## Context

**Cross-platform challenges:**
- Windows: Win32 API (EnumWindows, PrintWindow, GDI+)
- macOS: CoreGraphics/Quartz (CGWindowList, CGWindowID)
- Linux X11: Xlib/xcb with composite extension
- Linux Wayland: Limited support via pipewire/dbus portals or fallback to XWayland

**Platform-specific window attributes to capture:**
- Common: window ID, title, PID, position (x, y), dimensions (width, height)
- Windows: HWND, process name, window class, style flags, thread ID
- macOS: window number, owner name, owner PID, sharing state, alpha, memory usage
- Linux X11: XID, WM_CLASS, WM_WINDOW_ROLE, visual info, map state
- Linux Wayland: app_id, title (limited attrs via portals)

## Constraints

- **Tech stack**: Rust (cross-platform, no runtime dependencies)
- **Output format**: PNG only
- **Info format**: JSON only
- **CLI style**: Long flags with optional short flags

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Highlight border excluded from screenshot | Border is identification aid, not part of captured content | — Pending |
| JSON saves to same path as --output with .json extension | Consistent naming, predictable location | — Pending |
| --highlight is standalone mode | Clear separation between identification and capture workflows | — Pending |
| PNG only | Simpler implementation, PNG is lossless standard for screenshots | — Pending |

---
*Last updated: 2026-03-09 after initialization*
