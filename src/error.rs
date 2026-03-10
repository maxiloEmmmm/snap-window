use thiserror::Error;

/// Application-specific error types
///
/// These are domain errors that will be converted to anyhow::Error for propagation.
/// Each variant provides a clear, actionable error message for the user.
#[derive(Error, Debug)]
pub enum AppError {
    /// Window not found when searching by name/substring
    #[error("Window not found matching: {0}")]
    WindowNotFound(String),

    /// Invalid window index provided
    #[error("No window at index {index}. Use --list to see valid indices (0-{max})")]
    InvalidIndex { index: usize, max: usize },

    /// Platform is not supported
    #[error("Platform '{0}' is not supported")]
    UnsupportedPlatform(String),

    /// Failed to enumerate windows
    #[error("Failed to enumerate windows: {0}")]
    EnumerationFailed(String),
}

impl AppError {
    /// Create a WindowNotFound error with the given search criteria
    pub fn window_not_found(name: impl Into<String>) -> Self {
        Self::WindowNotFound(name.into())
    }

    /// Create an InvalidIndex error with the given index and max bounds
    pub fn invalid_index(index: usize, max: usize) -> Self {
        Self::InvalidIndex { index, max }
    }

    /// Create an UnsupportedPlatform error with the current platform name
    pub fn unsupported_platform() -> Self {
        Self::UnsupportedPlatform(std::env::consts::OS.to_string())
    }

    /// Create an EnumerationFailed error with the given message
    pub fn enumeration_failed(msg: impl Into<String>) -> Self {
        Self::EnumerationFailed(msg.into())
    }

    /// Create an EnumerationFailed error for platform-specific failures
    ///
    /// Use this for platform API failures such as:
    /// - X11 connection failures on Linux
    /// - CGWindowListCopyWindowInfo returning NULL on macOS
    /// - EnumWindows failing on Windows
    /// - Permission denied errors during window enumeration
    pub fn platform_error(msg: impl Into<String>) -> Self {
        Self::EnumerationFailed(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_not_found_display() {
        let err = AppError::window_not_found("Chrome");
        assert_eq!(err.to_string(), "Window not found matching: Chrome");
    }

    #[test]
    fn test_invalid_index_display() {
        let err = AppError::invalid_index(5, 3);
        assert_eq!(
            err.to_string(),
            "No window at index 5. Use --list to see valid indices (0-3)"
        );
    }

    #[test]
    fn test_unsupported_platform_display() {
        let err = AppError::UnsupportedPlatform("freebsd".to_string());
        assert_eq!(err.to_string(), "Platform 'freebsd' is not supported");
    }

    #[test]
    fn test_enumeration_failed_display() {
        let err = AppError::enumeration_failed("permission denied");
        assert_eq!(
            err.to_string(),
            "Failed to enumerate windows: permission denied"
        );
    }

    #[test]
    fn test_error_trait_implementation() {
        // Verify that AppError implements std::error::Error
        fn assert_error_trait<T: std::error::Error>() {}
        assert_error_trait::<AppError>();
    }

    #[test]
    fn test_unsupported_platform_auto_detect() {
        let err = AppError::unsupported_platform();
        // Should contain the current OS name
        let msg = err.to_string();
        assert!(msg.contains(std::env::consts::OS));
    }

    #[test]
    fn test_platform_error_display() {
        let err = AppError::platform_error("X11 connection refused");
        assert_eq!(
            err.to_string(),
            "Failed to enumerate windows: X11 connection refused"
        );
    }

    #[test]
    fn test_platform_error_creates_enumeration_failed() {
        // platform_error is an alias for enumeration_failed with clearer semantics
        let err = AppError::platform_error("CGWindowListCopyWindowInfo returned NULL");
        assert!(err.to_string().starts_with("Failed to enumerate windows:"));
    }

    #[test]
    fn test_platform_error_with_permission_denied() {
        let err = AppError::platform_error("permission denied");
        assert_eq!(
            err.to_string(),
            "Failed to enumerate windows: permission denied"
        );
    }
}
