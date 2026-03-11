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

/// Show a red highlight border around a window using 4 overlay windows.
///
/// Creates 4 topmost, tool windows with red backgrounds forming a border frame
/// around the target window. Windows are destroyed after 3 seconds.
#[cfg(target_os = "windows")]
pub fn show_highlight_border(info: &WindowInfo) -> Result<()> {
    use crate::error::AppError;
    use std::thread;
    use std::time::Duration;
    use windows::Win32::{
        Foundation::{COLORREF, HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Gdi::{CreateSolidBrush, RGB},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DestroyWindow, RegisterClassW, ShowWindow, CS_HREDRAW, CS_VREDRAW,
            CW_USEDEFAULT, HCURSOR, HICON, HMENU, SW_SHOWNA, WINDOW_EX_STYLE, WM_DESTROY,
            WNDCLASSW, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP,
        },
    };

    const THICKNESS: i32 = 4;
    const CLASS_NAME: &[u16] = w!("SnapWindowHighlight");

    unsafe {
        // Register window class with red background
        let hbrush = CreateSolidBrush(COLORREF(RGB(255, 0, 0).0));
        let wc = WNDCLASSW {
            lpfnWndProc: Some(highlight_wnd_proc),
            hInstance: windows::Win32::System::LibraryLoader::GetModuleHandleW(None)
                .map_err(|e| AppError::platform_error(format!("GetModuleHandleW failed: {}", e)))?
                .into(),
            hCursor: HCURSOR::default(),
            hIcon: HICON::default(),
            lpszClassName: windows::core::PCWSTR(CLASS_NAME.as_ptr()),
            hbrBackground: hbrush,
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };

        RegisterClassW(&wc);

        // Calculate border positions
        let x = info.x;
        let y = info.y;
        let width = info.width as i32;
        let height = info.height as i32;

        // Create 4 overlay windows
        let ex_style = WS_EX_TOPMOST | WS_EX_TOOLWINDOW;
        let style = WS_POPUP;

        let hwnds: Vec<HWND> = [
            // Top
            (x, y, width, THICKNESS),
            // Bottom
            (x, y + height - THICKNESS, width, THICKNESS),
            // Left
            (x, y + THICKNESS, THICKNESS, height - 2 * THICKNESS),
            // Right
            (x + width - THICKNESS, y + THICKNESS, THICKNESS, height - 2 * THICKNESS),
        ]
        .iter()
        .map(|(x, y, w, h)| {
            CreateWindowExW(
                ex_style,
                windows::core::PCWSTR(CLASS_NAME.as_ptr()),
                w!(""),
                style,
                *x,
                *y,
                *w,
                *h,
                None,
                HMENU::default(),
                wc.hInstance,
                None,
            )
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| AppError::platform_error(format!("CreateWindowExW failed: {}", e)))?;

        // Show all windows
        for hwnd in &hwnds {
            let _ = ShowWindow(*hwnd, SW_SHOWNA);
        }

        // Display for 3 seconds
        thread::sleep(Duration::from_secs(3));

        // Destroy all windows
        for hwnd in hwnds {
            let _ = DestroyWindow(hwnd);
        }
    }

    Ok(())
}

/// Window procedure for highlight overlay windows (minimal)
#[cfg(target_os = "windows")]
extern "system" fn highlight_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::DefWindowProcW;
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

/// Stub for non-Windows platforms (prevents compilation errors during development)
#[cfg(not(target_os = "windows"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Windows platform module is not available on this platform")
}

/// Stub for non-Windows platforms (prevents compilation errors during development)
#[cfg(not(target_os = "windows"))]
pub fn show_highlight_border(_info: &WindowInfo) -> Result<()> {
    anyhow::bail!("Windows highlight border is not available on this platform")
}
