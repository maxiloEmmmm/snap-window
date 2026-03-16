//! Runtime display server detection for Linux
//!
//! Detects whether the system is running X11, Wayland, or an unknown display server
//! by checking environment variables and fallback socket paths.

use std::path::Path;

/// Represents the detected display server type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    /// X11 display server (DISPLAY environment variable set)
    X11,
    /// Wayland display server (WAYLAND_DISPLAY environment variable set)
    Wayland,
    /// Unknown or unsupported display server
    Unknown,
}

/// Detect the display server type at runtime
///
/// Detection logic (per RESEARCH.md Pattern 1):
/// 1. Check WAYLAND_DISPLAY env var - if set, return Wayland
/// 2. Check DISPLAY env var - if set, return X11
/// 3. If neither, check for /run/user/{uid}/wayland-0 socket existence as fallback
/// 4. Return Unknown if all checks fail
///
/// # Examples
///
/// ```
/// use snap_window::platform::linux::detector::{DisplayServer, detect_display_server};
///
/// let server = detect_display_server();
/// match server {
///     DisplayServer::X11 => println!("Running on X11"),
///     DisplayServer::Wayland => println!("Running on Wayland"),
///     DisplayServer::Unknown => println!("No display server detected"),
/// }
/// ```
pub fn detect_display_server() -> DisplayServer {
    let wayland_display = std::env::var("WAYLAND_DISPLAY");
    let x11_display = std::env::var("DISPLAY");

    match (wayland_display, x11_display) {
        // Both set - could be XWayland; prefer native Wayland
        (Ok(_), Ok(_)) => DisplayServer::Wayland,
        // Only Wayland
        (Ok(_), Err(_)) => DisplayServer::Wayland,
        // Only X11
        (Err(_), Ok(_)) => DisplayServer::X11,
        // Neither - check for Wayland socket fallback
        (Err(_), Err(_)) => {
            // Try to find wayland socket in standard location
            if let Some(runtime_dir) = std::env::var_os("XDG_RUNTIME_DIR") {
                let wayland_socket = Path::new(&runtime_dir).join("wayland-0");
                if wayland_socket.exists() {
                    return DisplayServer::Wayland;
                }
            }

            // Fallback to hardcoded /run/user/{uid}/wayland-0
            if let Ok(uid) = std::process::id().to_string().parse::<u32>() {
                let fallback_socket = format!("/run/user/{}/wayland-0", uid);
                if Path::new(&fallback_socket).exists() {
                    return DisplayServer::Wayland;
                }
            }

            DisplayServer::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to clear display server environment variables
    fn clear_env() {
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::remove_var("DISPLAY");
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    // Test 1: detect_display_server returns X11 when only DISPLAY is set
    #[test]
    fn test_detect_x11_only() {
        clear_env();
        std::env::set_var("DISPLAY", ":0");

        let result = detect_display_server();
        assert_eq!(result, DisplayServer::X11);

        clear_env();
    }

    // Test 2: detect_display_server returns Wayland when only WAYLAND_DISPLAY is set
    #[test]
    fn test_detect_wayland_only() {
        clear_env();
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");

        let result = detect_display_server();
        assert_eq!(result, DisplayServer::Wayland);

        clear_env();
    }

    // Test 3: detect_display_server prefers Wayland when both are set (XWayland case)
    #[test]
    fn test_detect_prefers_wayland_when_both_set() {
        clear_env();
        std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
        std::env::set_var("DISPLAY", ":0");

        let result = detect_display_server();
        assert_eq!(result, DisplayServer::Wayland);

        clear_env();
    }

    // Test 4: detect_display_server returns Unknown when neither is set
    #[test]
    fn test_detect_unknown_when_neither_set() {
        clear_env();

        let result = detect_display_server();
        assert_eq!(result, DisplayServer::Unknown);

        clear_env();
    }

    // Test 5: DisplayServer enum implements Debug
    #[test]
    fn test_display_server_debug() {
        assert_eq!(format!("{:?}", DisplayServer::X11), "X11");
        assert_eq!(format!("{:?}", DisplayServer::Wayland), "Wayland");
        assert_eq!(format!("{:?}", DisplayServer::Unknown), "Unknown");
    }

    // Test 6: DisplayServer enum implements Clone and Copy
    #[test]
    fn test_display_server_clone_copy() {
        let x11 = DisplayServer::X11;
        let x11_copy = x11;
        let x11_clone = x11.clone();

        assert_eq!(x11, x11_copy);
        assert_eq!(x11, x11_clone);
    }

    // Test 7: DisplayServer enum implements PartialEq
    #[test]
    fn test_display_server_equality() {
        assert_eq!(DisplayServer::X11, DisplayServer::X11);
        assert_eq!(DisplayServer::Wayland, DisplayServer::Wayland);
        assert_eq!(DisplayServer::Unknown, DisplayServer::Unknown);

        assert_ne!(DisplayServer::X11, DisplayServer::Wayland);
        assert_ne!(DisplayServer::X11, DisplayServer::Unknown);
        assert_ne!(DisplayServer::Wayland, DisplayServer::Unknown);
    }
}
