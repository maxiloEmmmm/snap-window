//! Linux platform-specific window enumeration
//!
//! Uses X11 via x11rb to enumerate client windows from the root window's
//! _NET_CLIENT_LIST property. Extracts title, PID, app name, and bounds
//! using EWMH/ICCCM properties. Sorts by app_name then title.

use anyhow::Result;
use crate::window::WindowInfo;

/// List all visible windows on Linux (X11)
///
/// Connects to the X display, reads _NET_CLIENT_LIST from the root window,
/// and retrieves EWMH properties for each window. Windows without titles
/// are skipped. Sorted by app_name then title with sequential indices.
#[cfg(target_os = "linux")]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    use crate::error::AppError;
    use x11rb::{
        connection::Connection,
        protocol::xproto::AtomEnum,
        rust_connection::RustConnection,
    };

    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| AppError::enumeration_failed(format!("Failed to connect to X server: {}", e)))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;

    // Intern required atoms once — reuse for all windows
    let net_client_list = intern_atom(&conn, b"_NET_CLIENT_LIST")
        .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_CLIENT_LIST: {}", e)))?;
    let net_wm_name = intern_atom(&conn, b"_NET_WM_NAME")
        .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_WM_NAME: {}", e)))?;
    let net_wm_pid = intern_atom(&conn, b"_NET_WM_PID")
        .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_WM_PID: {}", e)))?;
    let utf8_string = intern_atom(&conn, b"UTF8_STRING")
        .map_err(|e| AppError::enumeration_failed(format!("Failed to intern UTF8_STRING: {}", e)))?;
    let wm_class_atom = intern_atom(&conn, b"WM_CLASS")
        .map_err(|e| AppError::enumeration_failed(format!("Failed to intern WM_CLASS: {}", e)))?;

    // Get _NET_CLIENT_LIST from root — list of all managed client windows
    let reply = conn.get_property(
        false,
        root,
        net_client_list,
        AtomEnum::WINDOW,
        0,
        u32::MAX / 4, // max 4-byte units
    )
    .map_err(|e| AppError::enumeration_failed(format!("Failed to query _NET_CLIENT_LIST: {}", e)))?
    .reply()
    .map_err(|e| AppError::enumeration_failed(format!("_NET_CLIENT_LIST reply error: {}", e)))?;

    let mut windows: Vec<WindowInfo> = Vec::new();

    if let Some(window_ids) = reply.value32() {
        for window in window_ids {
            // Get title — skip windows without a title
            let title = match get_window_title(&conn, window, net_wm_name, utf8_string) {
                Ok(t) if !t.is_empty() => t,
                _ => continue,
            };

            let pid = get_window_pid(&conn, window, net_wm_pid)
                .unwrap_or(None)
                .unwrap_or(0);

            let app_name = get_app_name(&conn, window, wm_class_atom)
                .unwrap_or(None)
                .unwrap_or_else(|| format!("PID:{}", pid));

            let (x, y, width, height) = get_window_geometry(&conn, window, root)
                .unwrap_or((0, 0, 0, 0));

            windows.push(WindowInfo::new(
                0, // Index assigned after sorting
                window as u64,
                title,
                pid,
                app_name,
                x,
                y,
                width,
                height,
            ));
        }
    }

    windows.sort_by(|a, b| {
        a.app_name
            .cmp(&b.app_name)
            .then_with(|| a.title.cmp(&b.title))
    });

    for (i, window) in windows.iter_mut().enumerate() {
        window.index = i;
    }

    Ok(windows)
}

/// Resolve an atom by name. Atoms are cached implicitly by the X server.
#[cfg(target_os = "linux")]
fn intern_atom(conn: &x11rb::rust_connection::RustConnection, name: &[u8]) -> Result<u32> {
    use x11rb::connection::Connection;
    Ok(conn.intern_atom(false, name)?.reply()?.atom)
}

/// Get window title: try _NET_WM_NAME (UTF-8) first, fall back to WM_NAME.
#[cfg(target_os = "linux")]
fn get_window_title(
    conn: &x11rb::rust_connection::RustConnection,
    window: u32,
    net_wm_name: u32,
    utf8_string: u32,
) -> Result<String> {
    use x11rb::{connection::Connection, protocol::xproto::AtomEnum};

    let reply = conn
        .get_property(false, window, net_wm_name, utf8_string, 0, u32::MAX / 4)?
        .reply()?;

    if reply.value_len > 0 {
        return Ok(String::from_utf8_lossy(&reply.value).into_owned());
    }

    // Fallback: WM_NAME (Latin-1 / ASCII)
    let reply = conn
        .get_property(
            false,
            window,
            AtomEnum::WM_NAME,
            AtomEnum::STRING,
            0,
            u32::MAX / 4,
        )?
        .reply()?;

    Ok(String::from_utf8_lossy(&reply.value).into_owned())
}

/// Get PID from _NET_WM_PID. Returns None if the property is absent.
#[cfg(target_os = "linux")]
fn get_window_pid(
    conn: &x11rb::rust_connection::RustConnection,
    window: u32,
    net_wm_pid: u32,
) -> Result<Option<u32>> {
    use x11rb::{connection::Connection, protocol::xproto::AtomEnum};

    let reply = conn
        .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)?
        .reply()?;

    Ok(reply.value32().and_then(|mut v| v.next()))
}

/// Get app name from WM_CLASS. WM_CLASS is "instance\0class\0"; return class (second string).
#[cfg(target_os = "linux")]
fn get_app_name(
    conn: &x11rb::rust_connection::RustConnection,
    window: u32,
    wm_class_atom: u32,
) -> Result<Option<String>> {
    use x11rb::{connection::Connection, protocol::xproto::AtomEnum};

    let reply = conn
        .get_property(
            false,
            window,
            wm_class_atom,
            AtomEnum::STRING,
            0,
            u32::MAX / 4,
        )?
        .reply()?;

    if reply.value.is_empty() {
        return Ok(None);
    }

    // Format: "instance\0class\0" — take the second null-delimited string as the class
    let parts: Vec<&[u8]> = reply.value.split(|&b| b == 0).collect();
    let class = parts
        .get(1)
        .filter(|s| !s.is_empty())
        .or_else(|| parts.first().filter(|s| !s.is_empty()))
        .map(|s| String::from_utf8_lossy(s).into_owned());

    Ok(class)
}

/// Get absolute window geometry using get_geometry + translate_coordinates.
#[cfg(target_os = "linux")]
fn get_window_geometry(
    conn: &x11rb::rust_connection::RustConnection,
    window: u32,
    root: u32,
) -> Result<(i32, i32, u32, u32)> {
    use x11rb::connection::Connection;

    let geom = conn.get_geometry(window)?.reply()?;

    // translate_coordinates converts window-relative (0,0) to root-relative
    let translate = conn
        .translate_coordinates(window, root, 0, 0)?
        .reply()?;

    Ok((
        translate.dst_x as i32,
        translate.dst_y as i32,
        geom.width as u32,
        geom.height as u32,
    ))
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Linux platform module is not available on this platform")
}
