use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on the system
///
/// Returns a vector of WindowInfo structs sorted by index.
/// On unsupported platforms, returns an error.
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    // Mock implementation for foundation phase
    // Returns sample windows for testing the CLI
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
            "Google Chrome",
            5678,
            "Google Chrome",
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
            "Safari",
            3456,
            "Safari",
            300,
            200,
            1000,
            700,
        ),
    ];
    
    Ok(mock_windows)
}
