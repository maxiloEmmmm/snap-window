//! Linux platform-specific window enumeration
//!
//! This module uses X11 to enumerate windows.
//! Future implementation will use x11rb crate for X11 access.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on Linux (X11)
///
/// Currently returns mock data for foundation phase testing.
/// Future implementation will use x11rb to query X11.
#[cfg(target_os = "linux")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    // Mock implementation for foundation phase
    // TODO: Implement using x11rb crate
    // - Connect to X11 display with x11rb::connect()
    // - Query root window with setup.roots[screen].root
    // - Use XQueryTree to enumerate child windows
    // - Filter for visible windows with XGetWindowAttributes
    // - Get window title with _NET_WM_NAME or WM_NAME
    // - Get PID with _NET_WM_PID
    // - Get window geometry with XGetGeometry
    // - Wayland fallback: use xdg-desktop-portal or wlr-screencopy

    let mock_windows = vec![
        WindowInfo::new(
            0,
            1001,
            "GNOME Terminal",
            1234,
            "gnome-terminal",
            100,
            100,
            800,
            600,
        ),
        WindowInfo::new(
            1,
            1002,
            "Firefox",
            5678,
            "firefox",
            200,
            150,
            1200,
            800,
        ),
        WindowInfo::new(
            2,
            1003,
            "Visual Studio Code",
            9012,
            "code",
            50,
            50,
            1400,
            900,
        ),
        WindowInfo::new(
            3,
            1004,
            "Files",
            3456,
            "nautilus",
            300,
            200,
            1000,
            700,
        ),
    ];

    Ok(mock_windows)
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Linux platform module is not available on this platform")
}
