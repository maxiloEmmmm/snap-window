use std::fmt;

/// Information about a window for display and targeting
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct WindowInfo {
    /// Display index for targeting (0-based)
    pub index: usize,
    /// Platform-specific window identifier
    pub window_id: u64,
    /// Window title text
    pub title: String,
    /// Process ID owning the window
    pub pid: u32,
    /// Application name
    pub app_name: String,
    /// X coordinate on screen
    pub x: i32,
    /// Y coordinate on screen
    pub y: i32,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

impl fmt::Display for WindowInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} (PID: {}, App: {})",
            self.index, self.title, self.pid, self.app_name
        )
    }
}

impl WindowInfo {
    /// Create a new WindowInfo with the given parameters
    pub fn new(
        index: usize,
        window_id: u64,
        title: impl Into<String>,
        pid: u32,
        app_name: impl Into<String>,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            index,
            window_id,
            title: title.into(),
            pid,
            app_name: app_name.into(),
            x,
            y,
            width,
            height,
        }
    }
}
