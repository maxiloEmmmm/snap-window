//! Capture service — takes a WindowInfo and output path, produces a PNG screenshot.
//!
//! Uses xcap to enumerate system windows, correlate by window ID (with title+pid fallback),
//! and capture the window contents to a PNG file.

use std::path::Path;

use anyhow::{Context, Result};
use xcap::Window as XCapWindow;

use crate::error::AppError;
use crate::window::WindowInfo;

/// Capture a screenshot of the window described by `info` and save as PNG to `output`.
///
/// Steps:
/// 1. Enumerate all xcap windows
/// 2. Find matching window by ID (fallback: title+pid)
/// 3. Check if minimized
/// 4. Capture image, detecting permission errors
/// 5. Create parent directories and save PNG
pub fn capture_window(_info: &WindowInfo, _output: &Path) -> Result<()> {
    todo!("capture_window not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_capture_fails_when_window_not_in_xcap_list() {
        // Create a WindowInfo with window_id=999999 (guaranteed not to match any real window)
        let info = WindowInfo::new(
            0,
            999999,
            "NonExistentWindow_Test_12345".to_string(),
            99999,
            "test_app".to_string(),
            0,
            0,
            100,
            100,
        );
        let tmp_path = PathBuf::from("/tmp/snap_window_test_capture_nonexistent.png");
        let result = capture_window(&info, &tmp_path);
        assert!(result.is_err(), "capture_window should fail for non-existent window");
        let err_msg = result.unwrap_err().to_string().to_lowercase();
        assert!(
            err_msg.contains("not found") || err_msg.contains("capture"),
            "Error should mention 'not found' or 'capture', got: {}",
            err_msg
        );
    }

    #[test]
    fn test_capture_error_variants_display() {
        let err = AppError::capture_failed("lens error");
        assert!(
            err.to_string().contains("lens error"),
            "CaptureFailed should contain the original message"
        );

        let err = AppError::permission_denied("denied");
        let msg = err.to_string().to_lowercase();
        assert!(
            msg.contains("screen recording") || msg.contains("system preferences") || msg.contains("permission"),
            "PermissionDenied should mention Screen Recording, System Preferences, or permission"
        );
    }

    #[test]
    fn test_permission_denied_message_actionable() {
        let err = AppError::permission_denied("test");
        let msg = err.to_string();
        assert!(
            msg.contains("System Preferences") || msg.contains("Privacy"),
            "PermissionDenied message should direct user to System Preferences or Privacy settings, got: {}",
            msg
        );
    }

    #[test]
    fn test_capture_service_module_compiles() {
        // Verify that the module can be imported and types resolve
        fn _assert_fn_signature(_f: fn(&WindowInfo, &Path) -> Result<()>) {}
        _assert_fn_signature(capture_window);
    }
}
