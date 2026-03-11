//! Linux platform module with backend trait pattern
//!
//! This module provides runtime detection between X11 and Wayland display servers,
//! using a trait-based backend pattern for extensibility.
//!
//! # Architecture
//!
//! - `detector`: Runtime display server detection (X11 vs Wayland)
//! - `x11`: X11 backend implementation using x11rb
//! - `wayland`: Wayland backend implementation using foreign-toplevel protocol
//!
//! # Usage
//!
//! ```
//! use snap_window::platform::linux::{create_backend, LinuxBackend};
//!
//! let backend = create_backend()?;
//! let windows = backend.list_windows()?;
//! ```

use anyhow::Result;
use crate::window::WindowInfo;

pub mod detector;
pub mod x11;
pub mod wayland;

use detector::{detect_display_server, DisplayServer};
use wayland::WaylandBackend;

/// Trait for Linux platform backends
///
/// Implementations provide window enumeration and highlight functionality
/// for specific display servers (X11, Wayland, etc.).
pub trait LinuxBackend {
    /// List all visible windows
    ///
    /// Returns a vector of `WindowInfo` structs containing window metadata
    /// such as title, PID, app name, and geometry.
    fn list_windows(&self) -> Result<Vec<WindowInfo>>;

    /// Show a highlight border around the specified window
    ///
    /// Creates a temporary visual indicator (typically a colored border)
    /// around the target window to help users identify it.
    fn show_highlight_border(&self, info: &WindowInfo) -> Result<()>;
}

/// Create the appropriate backend for the current display server
///
/// Detects the display server at runtime and returns a boxed backend
/// implementation. Supports both Wayland (with foreign-toplevel protocol)
/// and X11 backends, with automatic fallback from Wayland to X11.
///
/// # Errors
///
/// Returns an error if:
/// - No supported display server is detected
/// - Connection to the display server fails
///
/// # Examples
///
/// ```
/// use snap_window::platform::linux::create_backend;
///
/// match create_backend() {
///     Ok(backend) => println!("Backend created successfully"),
///     Err(e) => eprintln!("Failed to create backend: {}", e),
/// }
/// ```
pub fn create_backend() -> Result<Box<dyn LinuxBackend>> {
    match detect_display_server() {
        DisplayServer::Wayland => {
            // Try Wayland native backend first
            match WaylandBackend::new() {
                Ok(backend) => return Ok(Box::new(backend)),
                Err(e) => {
                    log::debug!("Wayland backend failed: {}, trying X11 fallback", e);
                    // Fall through to X11 fallback
                }
            }

            // X11 fallback for XWayland
            match x11::X11Backend::new() {
                Ok(backend) => Ok(Box::new(backend)),
                Err(_) => Err(crate::error::AppError::enumeration_failed(
                    "Wayland backend failed and X11 fallback unavailable"
                ).into())
            }
        }
        DisplayServer::X11 => {
            let backend = x11::X11Backend::new()?;
            Ok(Box::new(backend))
        }
        DisplayServer::Unknown => {
            Err(crate::error::AppError::enumeration_failed(
                "No supported display server found (tried Wayland, X11)"
            ).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: LinuxBackend trait is object-safe
    #[test]
    fn test_linux_backend_object_safe() {
        // This test verifies that LinuxBackend can be used as a trait object
        // If this compiles, the trait is object-safe
        fn _assert_object_safe(_: &dyn LinuxBackend) {}
    }

    // Test: create_backend returns an error when no display server is available
    #[test]
    fn test_create_backend_unknown_display_server() {
        // Clear environment variables
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::remove_var("DISPLAY");

        let result = create_backend();
        // Should fail since we have no display server in test environment
        assert!(result.is_err());
    }

    // Test: DisplayServer enum variants are accessible
    #[test]
    fn test_display_server_variants() {
        let x11 = DisplayServer::X11;
        let wayland = DisplayServer::Wayland;
        let unknown = DisplayServer::Unknown;

        assert!(matches!(x11, DisplayServer::X11));
        assert!(matches!(wayland, DisplayServer::Wayland));
        assert!(matches!(unknown, DisplayServer::Unknown));
    }
}
