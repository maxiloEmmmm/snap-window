//! X11 backend implementation for Linux
//!
//! Provides window enumeration and highlight functionality using the X11 protocol
//! via the x11rb crate. This is the primary backend for X11 display servers.

use anyhow::Result;
use crate::window::WindowInfo;
use super::LinuxBackend;

/// X11 backend implementation
///
/// Holds the X11 connection and screen information needed for all operations.
pub struct X11Backend {
    // Connection will be added in Task 2
    _placeholder: (),
}

impl X11Backend {
    /// Create a new X11 backend by connecting to the X server
    ///
    /// Attempts to connect using the DISPLAY environment variable.
    /// Returns an error if the connection fails.
    pub fn new() -> Result<Self> {
        // Placeholder implementation - will be fully implemented in Task 2
        Err(crate::error::AppError::platform_error(
            "X11Backend::new() not yet fully implemented"
        ).into())
    }
}

impl LinuxBackend for X11Backend {
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        // Placeholder - will be implemented in Task 2
        Err(crate::error::AppError::enumeration_failed(
            "X11Backend::list_windows() not yet implemented"
        ).into())
    }

    fn show_highlight_border(&self, _info: &WindowInfo) -> Result<()> {
        // Placeholder - will be implemented in Task 2
        Err(crate::error::AppError::platform_error(
            "X11Backend::show_highlight_border() not yet implemented"
        ).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: X11Backend implements LinuxBackend trait
    #[test]
    fn test_x11_backend_implements_trait() {
        // This test verifies that X11Backend implements LinuxBackend
        // If this compiles, the trait implementation is correct
        fn _assert_implements_trait<T: LinuxBackend>() {}
        _assert_implements_trait::<X11Backend>();
    }
}
