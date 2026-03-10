//! Windows platform-specific window enumeration
//!
//! This module uses the Windows API (windows-rs) to enumerate visible windows.
//! Uses EnumWindows to iterate top-level windows, filtering by visibility and
//! non-empty title. Process names are retrieved via OpenProcess +
//! QueryFullProcessImageNameW.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on Windows
///
/// Uses EnumWindows to enumerate top-level windows.
/// Filters to only visible windows with non-empty titles.
/// Sorts by app name then title and assigns sequential indices.
#[cfg(target_os = "windows")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    use crate::error::AppError;
    use std::sync::{Arc, Mutex};
    use windows::{
        Win32::{
            Foundation::{BOOL, HWND, LPARAM, RECT},
            System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
            System::ProcessStatus::QueryFullProcessImageNameW,
            UI::WindowsAndMessaging::{
                EnumWindows, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId,
                IsWindowVisible,
            },
        },
    };

    let windows: Arc<Mutex<Vec<WindowInfo>>> = Arc::new(Mutex::new(Vec::new()));
    let windows_clone = Arc::clone(&windows);

    unsafe {
        EnumWindows(
            Some(enum_window_proc),
            LPARAM(&windows_clone as *const _ as isize),
        )
        .map_err(|e| AppError::enumeration_failed(e.to_string()))?;
    }

    let mut result = windows.lock().map_err(|e| AppError::enumeration_failed(e.to_string()))?;
    result.sort_by(|a, b| {
        a.app_name
            .cmp(&b.app_name)
            .then_with(|| a.title.cmp(&b.title))
    });

    for (i, window) in result.iter_mut().enumerate() {
        window.index = i;
    }

    Ok(result.drain(..).collect())
}

#[cfg(target_os = "windows")]
extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    use windows::Win32::{
        Foundation::{RECT},
        System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        System::ProcessStatus::QueryFullProcessImageNameW,
        UI::WindowsAndMessaging::{
            GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
        },
    };

    unsafe {
        // Filter: must be visible
        if !IsWindowVisible(hwnd).as_bool() {
            return true.into();
        }

        // Get title — skip windows with empty titles
        let mut buffer = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buffer);
        if len == 0 {
            return true.into();
        }
        let title = String::from_utf16_lossy(&buffer[..len as usize]);

        // Get PID
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        // Get bounds
        let mut rect = RECT::default();
        let _ = GetWindowRect(hwnd, &mut rect);
        let x = rect.left;
        let y = rect.top;
        let width = (rect.right - rect.left) as u32;
        let height = (rect.bottom - rect.top) as u32;

        // Get process name, fall back to PID string if unavailable
        let app_name = get_process_name(pid).unwrap_or_else(|| format!("PID:{}", pid));

        let windows_arc = &*(lparam.0 as *const std::sync::Arc<std::sync::Mutex<Vec<WindowInfo>>>);
        if let Ok(mut guard) = windows_arc.lock() {
            guard.push(WindowInfo::new(
                0, // Index assigned after sorting
                hwnd.0 as u64,
                title,
                pid,
                app_name,
                x,
                y,
                width,
                height,
            ));
        }

        true.into() // Continue enumeration
    }
}

/// Get process name from PID using QueryFullProcessImageNameW.
/// Returns None if the process cannot be accessed.
#[cfg(target_os = "windows")]
fn get_process_name(pid: u32) -> Option<String> {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows::Win32::System::ProcessStatus::QueryFullProcessImageNameW;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; 260];
        let mut size = buffer.len() as u32;
        QueryFullProcessImageNameW(handle, Default::default(), windows::core::PWSTR(buffer.as_mut_ptr()), &mut size).ok()?;

        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        // Extract filename from full path
        path.split('\\').last().map(|s| s.to_string())
    }
}

/// Stub for non-Windows platforms (prevents compilation errors during development)
#[cfg(not(target_os = "windows"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Windows platform module is not available on this platform")
}
