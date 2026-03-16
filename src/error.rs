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

    /// Failed to enumerate windows
    #[error("Failed to enumerate windows: {0}")]
    EnumerationFailed(String),

    /// Window capture failed (non-permission error)
    #[error("Capture failed: {0}")]
    CaptureFailed(String),

    /// Screen Recording permission not granted (macOS)
    #[error("Screen Recording permission required.\nGo to: System Preferences > Privacy & Security > Screen Recording\nEnable access for this terminal application, then retry.\n(Original error: {0})")]
    PermissionDenied(String),

    /// Invalid regex pattern provided by user
    #[error("Invalid regex pattern '{pattern}': {details}")]
    InvalidRegexPattern { pattern: String, details: String },

    /// XDG Desktop Portal not available (Wayland)
    #[error("XDG Desktop Portal not available. Install xdg-desktop-portal and a backend (xdg-desktop-portal-gtk or xdg-desktop-portal-kde)")]
    PortalNotAvailable,

    /// Screenshot permission denied via portal (Wayland)
    #[error("Screenshot permission denied via portal. Grant permission in the dialog or system settings")]
    PortalPermissionDenied,
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

    /// Create a CaptureFailed error with the given message
    pub fn capture_failed(msg: impl Into<String>) -> Self {
        Self::CaptureFailed(msg.into())
    }

    /// Create a PermissionDenied error with the given message
    pub fn permission_denied(msg: impl Into<String>) -> Self {
        Self::PermissionDenied(msg.into())
    }

    /// Create an InvalidRegexPattern error with the pattern and error details
    pub fn invalid_regex_pattern(pattern: impl Into<String>, details: impl Into<String>) -> Self {
        Self::InvalidRegexPattern {
            pattern: pattern.into(),
            details: details.into(),
        }
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

    // REGEXP-01: InvalidRegexPattern error displays user-friendly message with pattern
    #[test]
    fn test_invalid_regex_pattern_display() {
        let err = AppError::invalid_regex_pattern("[invalid", "unclosed character class");
        assert_eq!(
            err.to_string(),
            "Invalid regex pattern '[invalid': unclosed character class"
        );
    }

    // REGEXP-01: Error constructor accepts pattern and error details
    #[test]
    fn test_invalid_regex_pattern_constructor() {
        let err = AppError::invalid_regex_pattern("test.*", "some error");
        match err {
            AppError::InvalidRegexPattern { pattern, details } => {
                assert_eq!(pattern, "test.*");
                assert_eq!(details, "some error");
            }
            _ => panic!("Expected InvalidRegexPattern variant"),
        }
    }

    // REGEXP-01: Error implements std::error::Error trait
    #[test]
    fn test_invalid_regex_pattern_error_trait() {
        let err = AppError::invalid_regex_pattern("pattern", "details");
        // Verify it can be used as a dyn Error
        fn assert_error_trait(_: &dyn std::error::Error) {}
        assert_error_trait(&err);
    }

    // LIN-01: PortalNotAvailable error displays user-friendly message
    #[test]
    fn test_portal_not_available_display() {
        let err = AppError::PortalNotAvailable;
        let msg = err.to_string();
        assert!(msg.contains("XDG Desktop Portal"), "Error should mention XDG Desktop Portal");
        assert!(msg.contains("not available"), "Error should mention not available");
    }

    // LIN-01: PortalPermissionDenied error displays user-friendly message
    #[test]
    fn test_portal_permission_denied_display() {
        let err = AppError::PortalPermissionDenied;
        let msg = err.to_string();
        assert!(msg.contains("permission denied"), "Error should mention permission denied");
        assert!(msg.contains("portal"), "Error should mention portal");
    }
}
