//! X11 backend implementation for Linux
//!
//! Provides window enumeration and highlight functionality using the X11 protocol
//! via the x11rb crate. This backend is used when X11 is detected as the display server.

use anyhow::{Context, Result};
use crate::error::AppError;
use crate::window::WindowInfo;
use super::LinuxBackend;

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

/// X11 backend implementation
///
/// Holds the X11 connection and screen number needed for all operations.
/// Created via `X11Backend::new()` which connects to the X server.
pub struct X11Backend {
    conn: RustConnection,
    screen_num: usize,
}

impl X11Backend {
    /// Create a new X11 backend by connecting to the X server
    ///
    /// Attempts to connect using the DISPLAY environment variable.
    /// Returns an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The DISPLAY environment variable is not set or invalid
    /// - Connection to the X server fails
    pub fn new() -> Result<Self> {
        let (conn, screen_num) = RustConnection::connect(None)
            .map_err(|e| AppError::platform_error(format!("Failed to connect to X server: {}", e)))?;

        Ok(Self { conn, screen_num })
    }

    /// Resolve an atom by name. Atoms are cached implicitly by the X server.
    fn intern_atom(&self, name: &[u8]) -> Result<u32> {
        Ok(self.conn.intern_atom(false, name)?.reply()?.atom)
    }

    /// Get window title: try _NET_WM_NAME (UTF-8) first, fall back to WM_NAME.
    fn get_window_title(
        &self,
        window: u32,
        net_wm_name: u32,
        utf8_string: u32,
    ) -> Result<String> {
        let reply = self
            .conn
            .get_property(false, window, net_wm_name, utf8_string, 0, u32::MAX / 4)?
            .reply()?;

        if reply.value_len > 0 {
            return Ok(String::from_utf8_lossy(&reply.value).into_owned());
        }

        // Fallback: WM_NAME (Latin-1 / ASCII)
        let reply = self
            .conn
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
    fn get_window_pid(&self, window: u32, net_wm_pid: u32) -> Result<Option<u32>> {
        let reply = self
            .conn
            .get_property(false, window, net_wm_pid, AtomEnum::CARDINAL, 0, 1)?
            .reply()?;

        Ok(reply.value32().and_then(|mut v| v.next()))
    }

    /// Get app name from WM_CLASS. WM_CLASS is "instance\0class\0"; return class (second string).
    fn get_app_name(&self, window: u32, wm_class_atom: u32) -> Result<Option<String>> {
        let reply = self
            .conn
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
    fn get_window_geometry(&self, window: u32, root: u32) -> Result<(i32, i32, u32, u32)> {
        let geom = self.conn.get_geometry(window)?.reply()?;

        // translate_coordinates converts window-relative (0,0) to root-relative
        let translate = self
            .conn
            .translate_coordinates(window, root, 0, 0)?
            .reply()?;

        Ok((
            translate.dst_x as i32,
            translate.dst_y as i32,
            geom.width as u32,
            geom.height as u32,
        ))
    }
}

impl LinuxBackend for X11Backend {
    /// List all visible windows on X11
    ///
    /// Reads _NET_CLIENT_LIST from the root window and retrieves EWMH properties
    /// for each window. Windows without titles are skipped. Results are sorted
    /// by app_name then title with sequential indices.
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let screen = &self.conn.setup().roots[self.screen_num];
        let root = screen.root;

        // Intern required atoms once — reuse for all windows
        let net_client_list = self
            .intern_atom(b"_NET_CLIENT_LIST")
            .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_CLIENT_LIST: {}", e)))?;
        let net_wm_name = self
            .intern_atom(b"_NET_WM_NAME")
            .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_WM_NAME: {}", e)))?;
        let net_wm_pid = self
            .intern_atom(b"_NET_WM_PID")
            .map_err(|e| AppError::enumeration_failed(format!("Failed to intern _NET_WM_PID: {}", e)))?;
        let utf8_string = self
            .intern_atom(b"UTF8_STRING")
            .map_err(|e| AppError::enumeration_failed(format!("Failed to intern UTF8_STRING: {}", e)))?;
        let wm_class_atom = self
            .intern_atom(b"WM_CLASS")
            .map_err(|e| AppError::enumeration_failed(format!("Failed to intern WM_CLASS: {}", e)))?;

        // Get _NET_CLIENT_LIST from root — list of all managed client windows
        let reply = self
            .conn
            .get_property(
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
                let title = match self.get_window_title(window, net_wm_name, utf8_string) {
                    Ok(t) if !t.is_empty() => t,
                    _ => continue,
                };

                let pid = self
                    .get_window_pid(window, net_wm_pid)
                    .unwrap_or(None)
                    .unwrap_or(0);

                let app_name = self
                    .get_app_name(window, wm_class_atom)
                    .unwrap_or(None)
                    .unwrap_or_else(|| format!("PID:{}", pid));

                let (x, y, width, height) = self
                    .get_window_geometry(window, root)
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

    /// Show a red highlight border around a window using 4 X11 overlay windows.
    ///
    /// Creates 4 borderless X11 windows with red backgrounds, positioned above
    /// the target window using _NET_WM_STATE_ABOVE. Windows auto-dismiss after 3 seconds.
    fn show_highlight_border(&self, info: &WindowInfo) -> Result<()> {
        const THICKNESS: u32 = 4;

        let screen = &self.conn.setup().roots[self.screen_num];
        let root = screen.root;
        let depth = screen.root_depth;
        let visual = screen.root_visual;

        // Intern atoms for window properties
        let net_wm_state = self
            .conn
            .intern_atom(false, b"_NET_WM_STATE")
            .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_STATE: {}", e)))?
            .reply()
            .map_err(|e| AppError::platform_error(format!("_NET_WM_STATE reply error: {}", e)))?
            .atom;

        let net_wm_state_above = self
            .conn
            .intern_atom(false, b"_NET_WM_STATE_ABOVE")
            .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_STATE_ABOVE: {}", e)))?
            .reply()
            .map_err(|e| AppError::platform_error(format!("_NET_WM_STATE_ABOVE reply error: {}", e)))?
            .atom;

        let net_wm_window_type = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE")
            .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_WINDOW_TYPE: {}", e)))?
            .reply()
            .map_err(|e| AppError::platform_error(format!("_NET_WM_WINDOW_TYPE reply error: {}", e)))?
            .atom;

        let net_wm_window_type_notification = self
            .conn
            .intern_atom(false, b"_NET_WM_WINDOW_TYPE_NOTIFICATION")
            .map_err(|e| AppError::platform_error(format!("Failed to intern _NET_WM_WINDOW_TYPE_NOTIFICATION: {}", e)))?
            .reply()
            .map_err(|e| AppError::platform_error(format!("_NET_WM_WINDOW_TYPE_NOTIFICATION reply error: {}", e)))?
            .atom;

        // Allocate red color (TrueColor visual - use RGB directly)
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
            let win_id = self
                .conn
                .generate_id()
                .map_err(|e| AppError::platform_error(format!("Failed to generate window ID: {}", e)))?;

            // Create window with override_redirect = true (no window manager decoration)
            let create_aux = CreateWindowAux::new()
                .background_pixel(red_pixel)
                .override_redirect(1)
                .event_mask(EventMask::empty());

            self.conn
                .create_window(
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
            self.conn
                .change_property(
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
            self.conn
                .change_property(
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
            self.conn
                .map_window(win_id)
                .map_err(|e| AppError::platform_error(format!("Failed to map window: {}", e)))?;

            windows.push(win_id);
        }

        // Flush all pending requests
        self.conn
            .flush()
            .map_err(|e| AppError::platform_error(format!("Failed to flush X connection: {}", e)))?;

        // Display for 3 seconds
        thread::sleep(Duration::from_secs(3));

        // Destroy all windows
        for win_id in windows {
            let _ = self.conn.destroy_window(win_id);
        }

        self.conn
            .flush()
            .map_err(|e| AppError::platform_error(format!("Failed to flush after destroy: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test: X11Backend implements LinuxBackend trait
    #[test]
    fn test_x11_backend_implements_trait() {
        fn _assert_implements_trait<T: LinuxBackend>() {}
        _assert_implements_trait::<X11Backend>();
    }

    // Test: X11Backend::new() returns error when X11 is unavailable
    #[test]
    fn test_x11_backend_new_fails_when_unavailable() {
        // This test will fail on non-Linux platforms or when DISPLAY is not set
        // On Linux CI without X11, it should fail gracefully
        std::env::remove_var("DISPLAY");
        let result = X11Backend::new();
        assert!(result.is_err());
    }

    // Test: X11Backend can be created as a trait object
    #[test]
    fn test_x11_backend_as_trait_object() {
        fn _assert_trait_object(_: Box<dyn LinuxBackend>) {}
        // Cannot actually call this without a real X11 connection,
        // but the type system verifies it compiles
    }
}
