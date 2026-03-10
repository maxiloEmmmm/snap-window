use crate::error::AppError;
use crate::window::WindowInfo;

/// Find a window by name using case-insensitive substring matching.
/// Checks both the window title and app name.
pub fn find_by_name<'a>(windows: &'a [WindowInfo], name: &str) -> Result<&'a WindowInfo, AppError> {
    let needle = name.to_lowercase();
    windows
        .iter()
        .find(|w| {
            w.title.to_lowercase().contains(&needle)
                || w.app_name.to_lowercase().contains(&needle)
        })
        .ok_or_else(|| AppError::window_not_found(name))
}

/// Find a window by PID using exact match.
pub fn find_by_pid<'a>(windows: &'a [WindowInfo], pid: u32) -> Result<&'a WindowInfo, AppError> {
    windows
        .iter()
        .find(|w| w.pid == pid)
        .ok_or_else(|| AppError::window_not_found(format!("PID {}", pid)))
}

/// Find a window by index.
/// Guards against integer underflow on empty window lists.
pub fn find_by_index(windows: &[WindowInfo], index: usize) -> Result<&WindowInfo, AppError> {
    if windows.is_empty() {
        return Err(AppError::invalid_index(index, 0));
    }
    windows
        .get(index)
        .ok_or_else(|| AppError::invalid_index(index, windows.len() - 1))
}

/// Print available windows to stderr for user guidance on lookup failures.
pub fn print_available_windows(windows: &[WindowInfo]) {
    eprintln!("\nAvailable windows:");
    for w in windows {
        eprintln!("  {}", w);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_window(index: usize, title: &str, pid: u32, app_name: &str) -> WindowInfo {
        WindowInfo::new(index, index as u64, title, pid, app_name, 0, 0, 100, 100)
    }

    fn sample_windows() -> Vec<WindowInfo> {
        vec![
            make_window(0, "Firefox - Wikipedia", 1000, "Firefox"),
            make_window(1, "Terminal", 1001, "Terminal"),
            make_window(2, "Code - main.rs", 1002, "iTerm2"),
        ]
    }

    // WIN-02: case-insensitive name matching
    #[test]
    fn test_find_by_name_case_insensitive() {
        let windows = sample_windows();
        let result = find_by_name(&windows, "firefox");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Firefox - Wikipedia");
    }

    // WIN-02: partial substring matching on title
    #[test]
    fn test_find_by_name_partial_match() {
        let windows = sample_windows();
        let result = find_by_name(&windows, "wiki");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Firefox - Wikipedia");
    }

    // WIN-02: fallback to app_name matching
    #[test]
    fn test_find_by_name_app_name_fallback() {
        let windows = sample_windows();
        let result = find_by_name(&windows, "iterm");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().app_name, "iTerm2");
    }

    // WIN-02: not found returns Err with the search string
    #[test]
    fn test_find_by_name_not_found() {
        let windows = sample_windows();
        let result = find_by_name(&windows, "NonExistentWindowXYZ");
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("NonExistentWindowXYZ"));
    }

    // WIN-03: find by PID exact match (Terminal, pid 1001)
    #[test]
    fn test_find_by_pid_found() {
        let windows = sample_windows();
        // Index 1 has pid 1001, title "Terminal"
        let result = find_by_pid(&windows, 1001);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Terminal");
    }

    // WIN-03: find by PID exact match (Code, pid 1002)
    #[test]
    fn test_find_by_pid_found_code() {
        let windows = sample_windows();
        // Index 2 has pid 1002, title "Code - main.rs"
        let result = find_by_pid(&windows, 1002);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Code - main.rs");
    }

    // WIN-03: find by PID not found returns Err
    #[test]
    fn test_find_by_pid_not_found() {
        let windows = sample_windows();
        let result = find_by_pid(&windows, 99999);
        assert!(result.is_err());
    }

    // WIN-04: find by index
    #[test]
    fn test_find_by_index_found() {
        let windows = sample_windows();
        let result = find_by_index(&windows, 2);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().title, "Code - main.rs");
    }

    // WIN-04 / ERR-02: out-of-bounds index returns Err with index and max
    #[test]
    fn test_find_by_index_out_of_bounds() {
        let windows = sample_windows();
        let result = find_by_index(&windows, 99);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("99"));
        assert!(err.contains("2"));
    }

    // WIN-05 / ERR-02: empty list guard — no integer underflow
    #[test]
    fn test_find_by_index_empty_list() {
        let result = find_by_index(&[], 0);
        assert!(result.is_err());
    }
}
