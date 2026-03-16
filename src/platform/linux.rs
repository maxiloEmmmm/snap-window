//! Linux platform-specific window enumeration (backward-compatible facade)
//!
//! This module provides backward-compatible re-exports of the Linux platform
//! functionality. The actual implementation has been moved to the `linux/`
//! subdirectory with a backend trait pattern for X11/Wayland support.
//!
//! # Migration Note
//!
//! New code should use the backend trait directly:
//! ```rust
//! use snap_window::platform::linux::{create_backend, LinuxBackend};
//!
//! let backend = create_backend()?;
//! let windows = backend.list_windows()?;
//! ```

use anyhow::Result;
use crate::window::WindowInfo;

// Re-export the modular backend system
pub mod linux {
    //! Linux platform backend modules
    pub use super::super::linux::*;
}

// Re-export detector types
pub use self::linux::{detect_display_server, DisplayServer};

// Re-export backend trait and factory
pub use self::linux::{LinuxBackend, create_backend};

// Re-export X11 backend for direct access if needed
pub use self::linux::x11::X11Backend;

/// List all visible windows on Linux
///
/// This is a backward-compatible facade that delegates to the appropriate
/// backend based on runtime display server detection.
///
/// # Errors
///
/// Returns an error if no supported display server is detected or if
/// connection to the display server fails.
#[cfg(target_os = "linux")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    let backend = create_backend()?;
    backend.list_windows()
}

/// Show a highlight border around a window
///
/// This is a backward-compatible facade that delegates to the appropriate
/// backend based on runtime display server detection.
///
/// # Errors
///
/// Returns an error if no supported display server is detected or if
/// the highlight operation fails.
#[cfg(target_os = "linux")]
pub fn show_highlight_border(info: &WindowInfo) -> Result<()> {
    let backend = create_backend()?;
    backend.show_highlight_border(info)
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Linux platform module is not available on this platform")
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn show_highlight_border(_info: &WindowInfo) -> Result<()> {
    anyhow::bail!("Linux highlight border is not available on this platform")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: list_windows returns error on non-Linux platforms
    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_list_windows_stub_on_non_linux() {
        let result = list_windows();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not available"));
    }

    // Test: show_highlight_border returns error on non-Linux platforms
    #[test]
    #[cfg(not(target_os = "linux"))]
    fn test_show_highlight_border_stub_on_non_linux() {
        let info = WindowInfo::new(0, 1, "Test", 123, "TestApp", 0, 0, 100, 100);
        let result = show_highlight_border(&info);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("not available"));
    }

    // Test: Re-exports are accessible
    #[test]
    fn test_reexports_exist() {
        // These should compile if re-exports are correct
        let _: DisplayServer = DisplayServer::Unknown;
    }
}
