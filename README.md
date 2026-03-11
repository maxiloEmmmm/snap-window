# snap-window

Cross-platform CLI tool for capturing screenshots of specific application windows.

## Features

- **Target windows by**: name substring, regex pattern, PID, or list index
- **Visual highlight mode**: Red border overlay for window identification
- **JSON export**: Window metadata with platform-specific attributes
- **Cross-platform**: Windows, macOS, Linux (X11)
- **Zero configuration**: Works out of the box

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Usage

### List all windows

```bash
snap-window --list
```

Output:
```
[0] Claude Code (PID: 705, App: Alacritty)
[1] snap-windows (PID: 391, App: Finder)
[2] Terminal (PID: 5472, App: Terminal)
```

### Capture by window name (substring match)

```bash
snap-window --window "Terminal"
snap-window -w "Firefox"
```

### Capture by regex pattern

```bash
snap-window --regexp "Terminal.*"
snap-window -r "(?i)firefox"  # case-insensitive
```

When multiple windows match, the first match is automatically selected.

### Capture by PID

```bash
snap-window --pid 1234
snap-window -p 5678
```

### Capture by index from list

```bash
snap-window --index 0
snap-window -i 3
```

### Specify output path

```bash
snap-window --window "Terminal" --output ~/screenshots/term.png
snap-window -w "Firefox" -o /tmp/firefox.png
```

Default output uses timestamped filename: `screenshot_YYYYMMDD_HHMMSS.png`

### Highlight window (visual identification)

```bash
snap-window --highlight 0
```

Shows a red border around the specified window for ~3 seconds. Also exports window info as JSON.

## Examples

```bash
# Quick screenshot of your terminal
snap-window -w "Terminal"

# Capture specific Firefox window using regex
snap-window -r "^Firefox - "

# Screenshot with custom filename
snap-window -i 2 -o ~/Desktop/window.png

# Find and capture in one go
snap-window -l  # see indices
snap-window -i 5 -o capture.png
```

## Platform Notes

### macOS
- Requires **Screen Recording** permission in System Settings > Privacy & Security
- First run will prompt for permission

### Linux
- Requires X11 (XWayland support pending)
- No additional dependencies needed

### Windows
- No additional setup required

## JSON Output

Highlight mode exports window metadata:

```json
{
  "index": 0,
  "window_id": "12345",
  "title": "Terminal",
  "pid": 5472,
  "app_name": "Terminal",
  "x": 100,
  "y": 100,
  "width": 800,
  "height": 600,
  "platform_attributes": {
    "macos": {
      "window_number": 12345,
      "owner_name": "Terminal",
      "owner_pid": 5472
    }
  }
}
```

## Development

```bash
# Run tests
cargo test

# Build release binary
cargo build --release

# Install locally
cargo install --path .
```

## License

MIT
