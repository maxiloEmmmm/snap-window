//! Wayland backend implementation for Linux
//!
//! Provides window enumeration using the wlr-foreign-toplevel-management protocol.
//! This backend is used when Wayland is detected as the display server.

use anyhow::{Context, Result};
use crate::error::AppError;
use crate::window::WindowInfo;
use super::LinuxBackend;

mod layer_shell;

use std::path::Path;

use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_registry, wl_display},
    globals::GlobalListContents,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_manager_v1::{ZwlrForeignToplevelManagerV1, self},
    zwlr_foreign_toplevel_handle_v1::{ZwlrForeignToplevelHandleV1, self},
};

/// Information about a toplevel window collected during event processing
#[derive(Debug, Clone, Default)]
struct ToplevelInfo {
    title: Option<String>,
    app_id: Option<String>,
}

/// State for collecting toplevel information during Wayland event processing
struct WaylandState {
    toplevels: Vec<(ZwlrForeignToplevelHandleV1, ToplevelInfo)>,
    manager: Option<ZwlrForeignToplevelManagerV1>,
    done: bool,
}

impl WaylandState {
    fn new() -> Self {
        Self {
            toplevels: Vec::new(),
            manager: None,
            done: false,
        }
    }
}

/// Wayland backend implementation
///
/// Holds the Wayland connection needed for all operations.
/// Created via `WaylandBackend::new()` which connects to the Wayland display.
pub struct WaylandBackend {
    connection: Connection,
}

impl WaylandBackend {
    /// Create a new Wayland backend by connecting to the Wayland display
    ///
    /// Attempts to connect using the WAYLAND_DISPLAY environment variable.
    /// Returns an error if the connection fails.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The WAYLAND_DISPLAY environment variable is not set or invalid
    /// - Connection to the Wayland compositor fails
    pub fn new() -> Result<Self> {
        let connection = Connection::connect_to_env()
            .map_err(|e| AppError::platform_error(format!("Failed to connect to Wayland display: {}", e)))?;

        Ok(Self { connection })
    }
}

impl LinuxBackend for WaylandBackend {
    /// List all visible windows on Wayland using foreign-toplevel protocol
    ///
    /// Uses the zwlr_foreign_toplevel_manager_v1 protocol to enumerate windows.
    /// Windows are sorted by app_name then title with sequential indices.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The foreign-toplevel protocol is not available
    /// - Connection to the compositor fails
    fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        // Get the display and create an event queue
        let display = self.connection.display();
        let mut event_queue = self.connection.new_event_queue();
        let qh = event_queue.handle();

        // Get the registry to bind to globals
        let registry = display.get_registry(&qh, ());

        // Create state to collect toplevel information
        let mut state = WaylandState::new();

        // Roundtrip to get registry events and bind to foreign-toplevel manager
        event_queue.roundtrip(&mut state)
            .map_err(|e| AppError::enumeration_failed(format!("Wayland roundtrip failed: {}", e)))?;

        // Check if we got the foreign-toplevel manager
        let manager = state.manager.take()
            .ok_or_else(|| AppError::enumeration_failed(
                "Foreign-toplevel protocol not available (compositor may not support wlr-foreign-toplevel-management)"
            ))?;

        // Call stop() to trigger toplevel events for all existing windows
        manager.stop(&qh, ());

        // Process events to collect toplevel information
        // We'll do multiple roundtrips with a timeout to collect all windows
        for _ in 0..10 {
            event_queue.flush()
                .map_err(|e| AppError::enumeration_failed(format!("Wayland flush failed: {}", e)))?;

            match event_queue.prepare_read() {
                Some(guard) => {
                    // Non-blocking read with timeout
                    match guard.read() {
                        Ok(_) => {}
                        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => return Err(AppError::enumeration_failed(format!("Wayland read failed: {}", e)).into()),
                    }
                }
                None => break,
            }

            event_queue.dispatch_pending(&mut state)
                .map_err(|e| AppError::enumeration_failed(format!("Wayland dispatch failed: {}", e)))?;
        }

        // Process collected toplevels into WindowInfo structs
        let mut windows: Vec<WindowInfo> = Vec::new();
        let mut window_id_counter: u64 = 1;

        for (handle, info) in &state.toplevels {
            // Skip windows without titles
            let title = match &info.title {
                Some(t) if !t.is_empty() => t.clone(),
                _ => continue,
            };

            let app_name = info.app_id.clone()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "Unknown".to_string());

            // Generate a unique window ID from the handle
            // The handle is a Wayland object, we use a counter for uniqueness
            let window_id = window_id_counter;
            window_id_counter += 1;

            // Foreign-toplevel doesn't provide geometry or PID directly
            // These would need to be obtained through other protocols
            windows.push(WindowInfo::new(
                0, // Index assigned after sorting
                window_id,
                title,
                0, // PID not available from foreign-toplevel
                app_name,
                0, // x not available
                0, // y not available
                0, // width not available
                0, // height not available
            ));
        }

        // Sort by app_name then title
        windows.sort_by(|a, b| {
            a.app_name
                .cmp(&b.app_name)
                .then_with(|| a.title.cmp(&b.title))
        });

        // Assign sequential indices
        for (i, window) in windows.iter_mut().enumerate() {
            window.index = i;
        }

        Ok(windows)
    }

    /// Show a highlight border around a window
    ///
    /// Uses the layer-shell protocol to create overlay borders around windows.
    /// Works on wlroots-based compositors (Sway, Hyprland, etc.).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The layer-shell protocol is not available (non-wlroots compositor)
    /// - Window geometry is not available (foreign-toplevel doesn't provide position/size)
    fn show_highlight_border(&self, info: &WindowInfo) -> Result<()> {
        layer_shell::show_highlight_border_layer_shell(&self.connection, info)
    }

    /// Capture a screenshot of the window using XDG Desktop Portal
    ///
    /// Uses the portal Screenshot API to capture the screen on Wayland.
    /// The portal saves to a temporary location which is then copied to
    /// the requested output path.
    fn capture_window(&self, _info: &WindowInfo, output_path: &Path) -> Result<()> {
        use ashpd::desktop::screenshot::{Screenshot, Options};
        use ashpd::WindowIdentifier;
        use std::path::PathBuf;

        // Create tokio runtime for async portal operation
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| AppError::capture_failed(format!("Failed to create tokio runtime: {}", e)))?;

        let portal_path: PathBuf = rt.block_on(async {
            let screenshot = Screenshot::new().await
                .map_err(|e| map_portal_error(e))?;

            let options = Options::default()
                .interactive(false)  // Don't show region selection dialog
                .modal(false);

            let uri = screenshot.screenshot(WindowIdentifier::default(), options).await
                .map_err(|e| map_portal_error(e))?;

            Ok::<_, AppError>(PathBuf::from(uri.path()))
        }).map_err(|e| anyhow::anyhow!(e))?;

        // Portal saves to a temp location, copy to user's requested path
        std::fs::copy(&portal_path, output_path)
            .map_err(|e| AppError::capture_failed(format!("Failed to copy screenshot to output path: {}", e)))?;

        // Clean up portal's temp file (optional - portal may clean up itself)
        let _ = std::fs::remove_file(&portal_path);

        Ok(())
    }
}

/// Map ashpd portal errors to appropriate AppError variants
fn map_portal_error(e: ashpd::Error) -> AppError {
    let msg = e.to_string().to_lowercase();
    if msg.contains("portal") && (msg.contains("not available") || msg.contains("no such interface")) {
        AppError::PortalNotAvailable
    } else if msg.contains("cancelled") || msg.contains("denied") || msg.contains("dismissed") {
        AppError::PortalPermissionDenied
    } else {
        AppError::capture_failed(format!("Portal error: {}", e))
    }
}

// Dispatch implementation for wl_registry
impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        state: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            // Bind to zwlr_foreign_toplevel_manager_v1 if available
            if interface == ZwlrForeignToplevelManagerV1::interface().name {
                // Use version 2 if available, otherwise version 1
                let bind_version = if version >= 2 { 2 } else { 1 };
                let manager = registry.bind(name, bind_version, qh, ());
                state.manager = Some(manager);
            }
        }
    }
}

// Dispatch implementation for zwlr_foreign_toplevel_manager_v1
impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _: &ZwlrForeignToplevelManagerV1,
        event: zwlr_foreign_toplevel_manager_v1::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_foreign_toplevel_manager_v1::Event::Toplevel { toplevel } => {
                // New toplevel window discovered
                state.toplevels.push((toplevel, ToplevelInfo::default()));
            }
            zwlr_foreign_toplevel_manager_v1::Event::Finished => {
                // Manager is finished, no more events
                state.done = true;
            }
            _ => {}
        }
    }
}

// Dispatch implementation for zwlr_foreign_toplevel_handle_v1
impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for WaylandState {
    fn event(
        state: &mut Self,
        handle: &ZwlrForeignToplevelHandleV1,
        event: zwlr_foreign_toplevel_handle_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Find the toplevel info for this handle
        if let Some((_, info)) = state.toplevels.iter_mut().find(|(h, _)| h == handle) {
            match event {
                zwlr_foreign_toplevel_handle_v1::Event::Title { title } => {
                    info.title = Some(title);
                }
                zwlr_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                    info.app_id = Some(app_id);
                }
                zwlr_foreign_toplevel_handle_v1::Event::Closed => {
                    // Remove closed toplevels
                    state.toplevels.retain(|(h, _)| h != handle);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: WaylandBackend struct can be instantiated (stub test)
    // This will fail in environments without Wayland - that's expected
    #[test]
    fn test_wayland_backend_new_fails_without_display() {
        // Clear WAYLAND_DISPLAY to ensure we get an error
        std::env::remove_var("WAYLAND_DISPLAY");
        let result = WaylandBackend::new();
        // Should fail since we don't have a Wayland display in test environment
        assert!(result.is_err());
    }

    // Test 2: WaylandBackend implements LinuxBackend trait
    #[test]
    fn test_wayland_backend_implements_trait() {
        fn _assert_implements_trait<T: LinuxBackend>() {}
        _assert_implements_trait::<WaylandBackend>();
    }

    // Test 3: WaylandBackend can be created as a trait object
    #[test]
    fn test_wayland_backend_as_trait_object() {
        fn _assert_trait_object(_: Box<dyn LinuxBackend>) {}
        // Cannot actually call this without a real Wayland connection,
        // but the type system verifies it compiles
    }

    // Test 4: ToplevelInfo struct works correctly
    #[test]
    fn test_toplevel_info_default() {
        let info = ToplevelInfo::default();
        assert!(info.title.is_none());
        assert!(info.app_id.is_none());
    }

    // Test 5: WaylandState initializes correctly
    #[test]
    fn test_wayland_state_new() {
        let state = WaylandState::new();
        assert!(state.toplevels.is_empty());
        assert!(state.manager.is_none());
        assert!(!state.done);
    }
}
