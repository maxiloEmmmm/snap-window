//! Window highlight service
//!
//! Provides functionality to visually highlight a window by drawing a red border
//! overlay around it. This is useful for identifying which window will be
//! captured before taking a screenshot.
//!
//! The highlight consists of 4 separate overlay windows (top, bottom, left, right)
//! that form a border frame around the target window. Using separate windows
//! ensures the highlight never appears in screenshots (HIL-02 requirement).

use anyhow::Result;
use crate::error::AppError;
use crate::platform;
use crate::window::WindowInfo;

/// Highlight a window by drawing a red border overlay around it.
///
/// This function validates the index, retrieves the window information,
/// and delegates to the platform-specific overlay implementation.
///
/// # Arguments
///
/// * `windows` - Slice of window information from window enumeration
/// * `index` - The index of the window to highlight
///
/// # Errors
///
/// Returns `AppError::InvalidIndex` if the index is out of bounds.
/// Returns platform-specific errors if overlay creation fails.
///
/// # Example
///
/// ```
/// use snap_window::highlight_service::highlight_window;
/// use snap_window::window::WindowInfo;
///
/// # fn example(windows: &[WindowInfo]) -> anyhow::Result<()> {
/// highlight_window(windows, 0)?;
/// # Ok(())
/// # }
/// ```
pub fn highlight_window(windows: &[WindowInfo], index: usize) -> Result<()> {
    // Validate index is in bounds
    if index >= windows.len() {
        return Err(AppError::invalid_index(index, windows.len().saturating_sub(1)).into());
    }

    let window_info = &windows[index];

    // Delegate to platform-specific overlay implementation
    platform::show_highlight_border(window_info)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlight_service_module_compiles() {
        // This test verifies the module and function signatures are correct
        // Actual highlighting requires a running window system
        fn _assert_fn_type() {
            let _: fn(&[WindowInfo], usize) -> Result<()> = highlight_window;
        }
    }

    #[test]
    fn test_highlight_window_invalid_index() {
        let windows = vec![];
        let result = highlight_window(&windows, 0);
        assert!(result.is_err());

        let windows = vec![
            WindowInfo::new(0, 1, "Test", 123, "TestApp", 0, 0, 100, 100),
        ];
        let result = highlight_window(&windows, 5);
        assert!(result.is_err());
    }
}
