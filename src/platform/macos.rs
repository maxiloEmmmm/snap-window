//! macOS platform-specific window enumeration
//!
//! This module uses the Core Graphics API to enumerate windows.
//! Future implementation will use objc2-core-graphics crate with CGWindowID.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on macOS
///
/// Currently returns mock data for foundation phase testing.
/// Future implementation will use CGWindowListCopyWindowInfo.
#[cfg(target_os = "macos")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    // Mock implementation for foundation phase
    // TODO: Implement using objc2-core-graphics crate
    // - Use CGWindowListCopyWindowInfo with kCGWindowListOptionOnScreenOnly
    // - Filter for normal windows (kCGWindowLayer == 0)
    // - Extract window ID from kCGWindowNumber
    // - Extract title from kCGWindowName
    // - Extract PID from kCGWindowOwnerPID
    // - Extract bounds from kCGWindowBounds

    let mock_windows = vec![
        WindowInfo::new(
            0,
            1001,
            "Terminal",
            1234,
            "Terminal.app",
            100,
            100,
            800,
            600,
        ),
        WindowInfo::new(
            1,
            1002,
            "Safari",
            5678,
            "Safari.app",
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
            "Code",
            50,
            50,
            1400,
            900,
        ),
        WindowInfo::new(
            3,
            1004,
            "Finder",
            3456,
            "Finder.app",
            300,
            200,
            1000,
            700,
        ),
    ];

    Ok(mock_windows)
}

/// Stub for non-macOS platforms (prevents compilation errors during development)
#[cfg(not(target_os = "macos"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("macOS platform module is not available on this platform")
}
