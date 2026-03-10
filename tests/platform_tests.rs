//! Platform module tests
//!
//! These tests verify that platform-specific modules compile correctly
//! with conditional compilation attributes and that window enumeration
//! behaves correctly on the current platform.

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

/// Test that all windows have valid bounds (positive dimensions)
///
/// Width and height must be positive. X and Y coordinates can be
/// any integer value since windows can be positioned off-screen.
#[test]
fn test_windows_have_valid_bounds() {
    let windows = snap_window::platform::list_windows().unwrap();

    assert!(
        !windows.is_empty(),
        "list_windows should return at least one window"
    );

    for window in &windows {
        assert!(
            window.width > 0,
            "Window '{}' should have positive width, got {}",
            window.title,
            window.width
        );
        assert!(
            window.height > 0,
            "Window '{}' should have positive height, got {}",
            window.title,
            window.height
        );
        // x and y can be any value (windows can be off-screen or on secondary monitors)
    }
}

/// Test that all returned windows have non-empty titles
///
/// This validates that filtering is working - system windows with empty titles
/// should be excluded from results.
#[test]
fn test_windows_have_non_empty_titles() {
    let windows = snap_window::platform::list_windows().unwrap();

    assert!(
        !windows.is_empty(),
        "list_windows should return at least one window"
    );

    for window in &windows {
        assert!(
            !window.title.is_empty(),
            "Window at index {} should have a non-empty title",
            window.index
        );
    }
}

/// Test that all windows have positive PIDs
///
/// Validates that PID extraction is working correctly.
/// All valid processes have a PID > 0.
#[test]
fn test_windows_have_positive_pids() {
    let windows = snap_window::platform::list_windows().unwrap();

    assert!(
        !windows.is_empty(),
        "list_windows should return at least one window"
    );

    for window in &windows {
        assert!(
            window.pid > 0,
            "Window '{}' should have a positive PID, got {}",
            window.title,
            window.pid
        );
    }
}

/// Test that all windows have non-empty app names
///
/// Validates that process name extraction is working correctly.
#[test]
fn test_windows_have_app_names() {
    let windows = snap_window::platform::list_windows().unwrap();

    assert!(
        !windows.is_empty(),
        "list_windows should return at least one window"
    );

    for window in &windows {
        assert!(
            !window.app_name.is_empty(),
            "Window '{}' (PID: {}) should have a non-empty app name",
            window.title,
            window.pid
        );
    }
}

/// Test that window enumeration is deterministic
///
/// Calling list_windows() twice should produce identical results
/// in the same order, validating consistent sorting.
#[test]
fn test_enumeration_is_deterministic() {
    let windows_first = snap_window::platform::list_windows().unwrap();
    let windows_second = snap_window::platform::list_windows().unwrap();

    assert_eq!(
        windows_first.len(),
        windows_second.len(),
        "Two calls to list_windows should return the same number of windows"
    );

    for (first, second) in windows_first.iter().zip(windows_second.iter()) {
        assert_eq!(
            first.window_id, second.window_id,
            "Window at position should have same window_id across calls"
        );
        assert_eq!(
            first.title, second.title,
            "Window at position should have same title across calls"
        );
        assert_eq!(
            first.index, second.index,
            "Window at position should have same index across calls"
        );
    }
}
