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

/// Show a red highlight border around a window using 4 X11 overlay windows.
///
/// Creates 4 borderless X11 windows with red backgrounds, positioned above
/// the target window using _NET_WM_STATE_ABOVE. Windows auto-dismiss after 3 seconds.
#[cfg(target_os = "linux")]
pub fn show_highlight_border(info: &WindowInfo) -> Result<()> {
    use crate::error::AppError;
    use std::thread;
    use std::time::Duration;
    use x11rb::{
        connection::Connection,
        protocol::xproto::{
            AtomEnum, ChangeWindowAttributesAux, ConfigWindow, ConfigureWindowAux, ConnectionExt,
            CreateWindowAux, EventMask, PropMode, SetWindowAttributes, WindowClass, CW,
        },
        rust_connection::RustConnection,
    };

    const THICKNESS: u32 = 4;

    let (conn, screen_num) = RustConnection::connect(None)
        .map_err(|e| AppError::platform_error(format!("Failed to connect to X server: {}", e)))?;

    let screen = &conn.setup().roots[screen_num];
    let root = screen.root;
    let depth = screen.root_depth;
    let visual = screen.root_visual;

    // Intern atoms for window properties
    let net_wm_state = conn
        .intern_atom(false, b"_NET_WM_STATE")
        .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_STATE: {}", e)))?
        .reply()
        .map_err(|e| AppError::platform_error(format!("_NET_WM_STATE reply error: {}", e)))?
        .atom;

    let net_wm_state_above = conn
        .intern_atom(false, b"_NET_WM_STATE_ABOVE")
        .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_STATE_ABOVE: {}", e)))?
        .reply()
        .map_err(|e| AppError::platform_error(format!("_NET_WM_STATE_ABOVE reply error: {}", e)))?
        .atom;

    let net_wm_window_type = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
        .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_WINDOW_TYPE: {}", e)))?
        .reply()
        .map_err(|e| AppError::platform_error(format!("_NET_WM_WINDOW_TYPE reply error: {}", e)))?
        .atom;

    let net_wm_window_type_notification = conn
        .intern_atom(false, b"_NET_WM_WINDOW_TYPE_NOTIFICATION")
        .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_WINDOW_TYPE_NOTIFICATION: {}", e)))?
        .reply()
        .map_err(|e| AppError::platform_error(format!("_NET_WM_WINDOW_TYPE_NOTIFICATION reply error: {}", e)))?
        .atom;

    // Allocate red color (TrueColor visual - use RGB directly)
    // For TrueColor, we can use the pixel value directly: 0xFF0000 for red
    let red_pixel: u32 = 0xFF0000;

    let x = info.x as i16;
    let y = info.y as i16;
    let width = info.width as u16;
    let height = info.height as u16;

    // Calculate border positions
    let positions = [
        // Top
        (x, y, width as u16, THICKNESS as u16),
        // Bottom
        (x, y + height as i16 - THICKNESS as i16, width as u16, THICKNESS as u16),
        // Left
        (x, y + THICKNESS as i16, THICKNESS as u16, (height - 2 * THICKNESS as u16) as u16),
        // Right
        (x + width as i16 - THICKNESS as i16, y + THICKNESS as i16, THICKNESS as u16, (height - 2 * THICKNESS as u16) as u16),
    ];

    let mut windows: Vec<u32> = Vec::with_capacity(4);

    for (win_x, win_y, win_w, win_h) in positions {
        let win_id = conn
            .generate_id()
            .map_err(|e| AppError::platform_error(format!("Failed to generate window ID: {}", e)))?;

        // Create window with override_redirect = true (no window manager decoration)
        let create_aux = CreateWindowAux::new()
            .background_pixel(red_pixel)
            .override_redirect(1)
            .event_mask(EventMask::empty());

        conn.create_window(
            depth,
            win_id,
            root,
            win_x,
            win_y,
            win_w,
            win_h,
            0, // border width
            WindowClass::INPUT_OUTPUT,
            visual,
            &create_aux,
        )
        .map_err(|e| AppError::platform_error(format!("Failed to create window: {}", e)))?;

        // Set _NET_WM_STATE_ABOVE to keep window on top
        let above_atom_bytes: [u8; 4] = net_wm_state_above.to_le_bytes();
        conn.change_property(
            PropMode::REPLACE,
            win_id,
            net_wm_state,
            AtomEnum::ATOM,
            32,
            1,
            &above_atom_bytes,
        )
        .map_err(|e| AppError::platform_error(format!("Failed to set _NET_WM_STATE_ABOVE: {}", e)))?;

        // Set _NET_WM_WINDOW_TYPE to _NET_WM_WINDOW_TYPE_NOTIFICATION
        let window_type_bytes: [u8; 4] = net_wm_window_type_notification.to_le_bytes();
        conn.change_property(
            PropMode::REPLACE,
            win_id,
            net_wm_window_type,
            AtomEnum::ATOM,
            32,
            1,
            &window_type_bytes,
        )
        .map_err(|e| AppError::platform_error(format!("Failed to set _NET_WM_WINDOW_TYPE: {}", e)))?;

        // Map (show) the window
        conn.map_window(win_id)
            .map_err(|e| AppError::platform_error(format!("Failed to map window: {}", e)))?;

        windows.push(win_id);
    }

    // Flush all pending requests
    conn.flush()
        .map_err(|e| AppError::platform_error(format!("Failed to flush X connection: {}", e)))?;

    // Display for 3 seconds
    thread::sleep(Duration::from_secs(3));

    // Destroy all windows
    for win_id in windows {
        let _ = conn.destroy_window(win_id);
    }

    conn.flush()
        .map_err(|e| AppError::platform_error(format!("Failed to flush after destroy: {}", e)))?;

    Ok(())
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn list_windows() -> Result<Vec<WindowInfo>> {
    anyhow::bail!("Linux platform module is not available on this platform")
}

/// Stub for non-Linux platforms (prevents compilation errors during development)
#[cfg(not(target_os = "linux"))]
pub fn show_highlight_border(_info: &WindowInfo) -> Result<()> {
    anyhow::bail!("Linux highlight border is not available on this platform")
}
