//! Platform module tests
//!
//! These tests verify that platform-specific modules compile correctly
//! with conditional compilation attributes.

use snap_window::window::WindowInfo;

/// Test that list_windows() returns valid window data on the current platform
#[test]
fn test_list_windows_returns_data() {
    let windows = snap_window::platform::list_windows().unwrap();

    // Should return at least one mock window
    assert!(!windows.is_empty(), "list_windows should return mock data");

    // Each window should have valid fields
    for window in &windows {
        assert!(!window.title.is_empty(), "Window title should not be empty");
        assert!(window.pid > 0, "Window PID should be positive");
        assert!(!window.app_name.is_empty(), "App name should not be empty");
        assert!(window.width > 0, "Window width should be positive");
        assert!(window.height > 0, "Window height should be positive");
    }
}

/// Test that the first window has index 0
#[test]
fn test_window_indices_start_at_zero() {
    let windows = snap_window::platform::list_windows().unwrap();

    if let Some(first) = windows.first() {
        assert_eq!(first.index, 0, "First window should have index 0");
    }
}

/// Test that window indices are sequential
#[test]
fn test_window_indices_are_sequential() {
    let windows = snap_window::platform::list_windows().unwrap();

    for (i, window) in windows.iter().enumerate() {
        assert_eq!(
            window.index, i,
            "Window at position {} should have index {}",
            i, i
        );
    }
}

/// Test that platform-specific window IDs are unique
#[test]
fn test_window_ids_are_unique() {
    let windows = snap_window::platform::list_windows().unwrap();

    let mut ids: Vec<u64> = windows.iter().map(|w| w.window_id).collect();
    ids.sort();
    ids.dedup();

    assert_eq!(
        ids.len(),
        windows.len(),
        "All window IDs should be unique"
    );
}
