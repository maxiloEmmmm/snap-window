//! Windows platform-specific window enumeration
//!
//! This module uses the Windows API to enumerate windows.
//! Future implementation will use windows-rs crate with HWND handles.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on Windows
///
/// Currently returns mock data for foundation phase testing.
/// Future implementation will use EnumWindows API.
#[cfg(target_os = "windows")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    // Mock implementation for foundation phase
    // TODO: Implement using windows-rs crate with EnumWindows
    // - Use HWND for window handles
    // - Filter for visible windows only
    // - Get window title with GetWindowText
    // - Get process ID with GetWindowThreadProcessId

    let mock_windows = vec![
        WindowInfo::new(
            0,
            1001,
            "Command Prompt",
            1234,
            "cmd.exe",
            100,
            100,
            800,
            600,
        ),
        WindowInfo::new(
            1,
            1002,
            "Microsoft Edge",
            5678,
            "msedge.exe",
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
            "Code.exe",
            50,
            50,
            1400,
            900,
        ),
        WindowInfo::new(
            3,
            1004,
            "File Explorer",
            3456,
            "explorer.exe",
            300,
            200,
            1000,
            700,
        ),
    ];

    Ok(mock_windows)
}

/// Stub for non-Windows platforms (prevents compilation errors during development)
#[cfg(not(target_os = "windows"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Windows platform module is not available on this platform")
}
