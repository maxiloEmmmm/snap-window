# Requirements: snap-window

**Defined:** 2026-03-09
**Core Value:** Users can reliably capture any visible window as a PNG image using simple CLI commands, regardless of operating system.

## v1 Requirements

### Core CLI

- [x] **CLI-01**: CLI accepts `--window <name>` to target window by substring match
- [x] **CLI-02**: CLI accepts `--pid <pid>` to target window by process ID
- [x] **CLI-03**: CLI accepts `--index <n>` to target window by list index
- [x] **CLI-04**: CLI accepts `--output <path>` for configurable screenshot path
- [x] **CLI-05**: CLI accepts `--list` to enumerate all windows with indices
- [x] **CLI-06**: CLI accepts `--highlight <index>` to add red border to window (no screenshot)
- [x] **CLI-07**: Default output path uses timestamped filename in current directory
- [x] **CLI-08**: Window list displays: index, window title, PID, application name

### Window Operations

- [ ] **WIN-01**: Cross-platform window enumeration (Windows, macOS, Linux X11)
- [x] **WIN-02**: Target window by substring match on title (case-insensitive)
- [x] **WIN-03**: Target window by exact PID match
- [x] **WIN-04**: Target window by index from list
- [x] **WIN-05**: Graceful error when window not found — auto-list all windows
- [x] **WIN-06**: Graceful error on unsupported platform — list all windows if possible

### Screenshot Capture

- [x] **CAP-01**: Capture target window content as PNG
- [x] **CAP-02**: PNG encoding with standard compression
- [x] **CAP-03**: Screenshot excludes highlight border (if highlight was shown)
- [x] **CAP-04**: Capture respects window bounds (position and dimensions)

### Highlight Mode

- [x] **HIL-01**: Red border overlay around specified window
- [x] **HIL-02**: Border is visual only — not included in saved screenshot
- [x] **HIL-03**: Highlight mode saves window info as JSON only (no screenshot)
- [x] **HIL-04**: JSON output path follows same logic as `--output` (same name, .json extension)

### Window Info JSON

- [x] **JSON-01**: JSON contains platform-specific window attributes
- [x] **JSON-02**: Common fields: window_id, title, pid, app_name, x, y, width, height
- [x] **JSON-03**: Windows fields: hwnd, window_class, thread_id
- [x] **JSON-04**: macOS fields: window_number, owner_name, owner_pid, sharing_state
- [x] **JSON-05**: Linux X11 fields: xid, wm_class, wm_window_role, visual_info

### Error Handling

- [x] **ERR-01**: Clear error message when target window not found
- [x] **ERR-02**: Auto-list all available windows on error
- [x] **ERR-03**: Clear error when platform unsupported
- [x] **ERR-04**: Clear error when required permissions missing (macOS Screen Recording)

## v2 Requirements

### Platform Support

- [x] **LIN-01**: Native Wayland support (via XDG Desktop Portal)
- [x] **LIN-02**: Runtime detection between X11 and Wayland

### Additional Features

- **FEAT-01`: Clipboard output option (--clipboard)
- **FEAT-02`: Delay/timer option (--delay <ms>)
- **FEAT-03`: Silent/quiet mode (--quiet)
- **FEAT-04`: Window class/role targeting (Linux)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Multiple image formats (JPEG, BMP, WebP) | PNG is standard for screenshots; keep implementation focused |
| Full-screen capture without window target | Out of scope for v1; tool is window-specific |
| Video capture / screen recording | Different tool category; requires different architecture |
| GUI interface | CLI-only by design |
| Image editing/annotation | Border is temporary visual aid only; no post-processing |
| Real-time streaming | Out of scope; this is a capture tool, not streaming tool |
| Cloud upload | Requires auth, network, privacy considerations; defer |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| CLI-01 | Phase 1 | Complete |
| CLI-02 | Phase 1 | Complete |
| CLI-03 | Phase 1 | Complete |
| CLI-04 | Phase 1 | Complete |
| CLI-05 | Phase 1 | Complete |
| CLI-06 | Phase 1 | Complete |
| CLI-07 | Phase 1 | Complete |
| CLI-08 | Phase 1 | Complete |
| WIN-01 | Phase 2 | Pending |
| WIN-02 | Phase 3 | Complete |
| WIN-03 | Phase 3 | Complete |
| WIN-04 | Phase 3 | Complete |
| WIN-05 | Phase 3 | Complete |
| WIN-06 | Phase 2 | Complete |
| CAP-01 | Phase 4 | Complete |
| CAP-02 | Phase 4 | Complete |
| CAP-03 | Phase 4 | Complete |
| CAP-04 | Phase 4 | Complete |
| HIL-01 | Phase 5 | Complete |
| HIL-02 | Phase 5 | Complete |
| HIL-03 | Phase 5 | Complete |
| HIL-04 | Phase 8 | Complete |
| JSON-01 | Phase 5 | Complete |
| JSON-02 | Phase 5 | Complete |
| JSON-03 | Phase 5 | Complete |
| JSON-04 | Phase 5 | Complete |
| JSON-05 | Phase 5 | Complete |
| ERR-01 | Phase 1 | Complete |
| ERR-02 | Phase 1 | Complete |
| ERR-03 | Phase 1 | Complete |
| ERR-04 | Phase 4 | Complete |

**Coverage:**
- v1 requirements: 30 total
- Mapped to phases: 30
- Unmapped: 0

---

*Requirements defined: 2026-03-09*
*Last updated: 2026-03-12 (Phase 8 assigned for HIL-04 gap closure)*
