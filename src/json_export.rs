//! JSON export module for window metadata serialization
//!
//! Provides serde-based serialization of WindowInfo to JSON format,
//! including platform-specific attributes.

use crate::window::WindowInfo;
use serde::Serialize;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};

/// JSON-serializable window information
#[derive(Debug, Clone, Serialize)]
pub struct WindowInfoJson {
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
    /// Platform name (windows, macos, linux)
    pub platform: String,
    /// Platform-specific attributes (flattened into the main JSON object)
    #[serde(flatten)]
    pub platform_attrs: serde_json::Map<String, serde_json::Value>,
}

impl WindowInfoJson {
    /// Create a WindowInfoJson from a WindowInfo, populating common fields
    /// and platform-specific attributes
    pub fn from_window_info(info: &WindowInfo) -> Self {
        let platform = std::env::consts::OS.to_string();
        let platform_attrs = build_platform_attrs(info);

        Self {
            window_id: info.window_id,
            title: info.title.clone(),
            pid: info.pid,
            app_name: info.app_name.clone(),
            x: info.x,
            y: info.y,
            width: info.width,
            height: info.height,
            platform,
            platform_attrs,
        }
    }
}

/// Build platform-specific attributes map based on the current platform
#[cfg(target_os = "macos")]
pub fn build_platform_attrs(info: &WindowInfo) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Value;

    let mut attrs = serde_json::Map::new();
    attrs.insert("window_number".to_string(), Value::Number(info.window_id.into()));
    attrs.insert("owner_name".to_string(), Value::String(info.app_name.clone()));
    attrs.insert("owner_pid".to_string(), Value::Number(info.pid.into()));
    attrs.insert("sharing_state".to_string(), Value::Number(0.into()));
    attrs
}

#[cfg(target_os = "windows")]
pub fn build_platform_attrs(info: &WindowInfo) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Value;

    let mut attrs = serde_json::Map::new();
    attrs.insert("hwnd".to_string(), Value::Number(info.window_id.into()));
    attrs.insert("window_class".to_string(), Value::String(info.app_name.clone()));
    attrs.insert("thread_id".to_string(), Value::Number(info.pid.into()));
    attrs
}

#[cfg(target_os = "linux")]
pub fn build_platform_attrs(info: &WindowInfo) -> serde_json::Map<String, serde_json::Value> {
    use serde_json::Value;

    let mut attrs = serde_json::Map::new();
    attrs.insert("xid".to_string(), Value::Number(info.window_id.into()));
    attrs.insert("wm_class".to_string(), Value::String(info.app_name.clone()));
    attrs.insert("wm_window_role".to_string(), Value::String("".to_string()));
    attrs
}

/// Convert a PNG path to a JSON path by swapping the extension
pub fn json_output_path(png_path: &Path) -> PathBuf {
    png_path.with_extension("json")
}

/// Write WindowInfoJson to a file as pretty-printed JSON
/// Creates parent directories if needed
pub fn write_json(info: &WindowInfoJson, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, info)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    fn create_test_window_info() -> WindowInfo {
        WindowInfo::new(
            0,
            12345,
            "Test Window",
            6789,
            "TestApp",
            100,
            200,
            800,
            600,
        )
    }

    #[test]
    fn test_json_output_path_swaps_extension() {
        let path = Path::new("screenshot.png");
        let result = json_output_path(path);
        assert_eq!(result, PathBuf::from("screenshot.json"));
    }

    #[test]
    fn test_json_output_path_with_directory() {
        let path = Path::new("/tmp/captures/window.png");
        let result = json_output_path(path);
        assert_eq!(result, PathBuf::from("/tmp/captures/window.json"));
    }

    #[test]
    fn test_json_output_path_no_extension() {
        let path = Path::new("no_extension");
        let result = json_output_path(path);
        assert_eq!(result, PathBuf::from("no_extension.json"));
    }

    #[test]
    fn test_window_info_json_from_window_info() {
        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);

        assert_eq!(json_info.window_id, 12345);
        assert_eq!(json_info.title, "Test Window");
        assert_eq!(json_info.pid, 6789);
        assert_eq!(json_info.app_name, "TestApp");
        assert_eq!(json_info.x, 100);
        assert_eq!(json_info.y, 200);
        assert_eq!(json_info.width, 800);
        assert_eq!(json_info.height, 600);
        assert_eq!(json_info.platform, std::env::consts::OS);
    }

    #[test]
    fn test_window_info_json_serialization() {
        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);

        let json_str = serde_json::to_string_pretty(&json_info).unwrap();

        // Verify common fields are present
        assert!(json_str.contains("\"window_id\": 12345"));
        assert!(json_str.contains("\"title\": \"Test Window\""));
        assert!(json_str.contains("\"pid\": 6789"));
        assert!(json_str.contains("\"app_name\": \"TestApp\""));
        assert!(json_str.contains("\"x\": 100"));
        assert!(json_str.contains("\"y\": 200"));
        assert!(json_str.contains("\"width\": 800"));
        assert!(json_str.contains("\"height\": 600"));
        assert!(json_str.contains("\"platform\""));
    }

    #[test]
    fn test_platform_attrs_present() {
        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);

        // Platform-specific attrs should be populated
        #[cfg(target_os = "macos")]
        {
            assert!(json_info.platform_attrs.contains_key("window_number"));
            assert!(json_info.platform_attrs.contains_key("owner_name"));
            assert!(json_info.platform_attrs.contains_key("owner_pid"));
            assert!(json_info.platform_attrs.contains_key("sharing_state"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(json_info.platform_attrs.contains_key("hwnd"));
            assert!(json_info.platform_attrs.contains_key("window_class"));
            assert!(json_info.platform_attrs.contains_key("thread_id"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(json_info.platform_attrs.contains_key("xid"));
            assert!(json_info.platform_attrs.contains_key("wm_class"));
            assert!(json_info.platform_attrs.contains_key("wm_window_role"));
        }
    }

    #[test]
    fn test_platform_attrs_in_serialized_json() {
        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);
        let json_str = serde_json::to_string_pretty(&json_info).unwrap();

        // Platform-specific attrs should appear at top level (flattened)
        #[cfg(target_os = "macos")]
        {
            assert!(json_str.contains("\"window_number\":"));
            assert!(json_str.contains("\"owner_name\":"));
            assert!(json_str.contains("\"owner_pid\":"));
            assert!(json_str.contains("\"sharing_state\":"));
        }

        #[cfg(target_os = "windows")]
        {
            assert!(json_str.contains("\"hwnd\":"));
            assert!(json_str.contains("\"window_class\":"));
            assert!(json_str.contains("\"thread_id\":"));
        }

        #[cfg(target_os = "linux")]
        {
            assert!(json_str.contains("\"xid\":"));
            assert!(json_str.contains("\"wm_class\":"));
            assert!(json_str.contains("\"wm_window_role\":"));
        }
    }

    #[test]
    fn test_write_json_creates_file() {
        let temp_dir = std::env::temp_dir();
        let test_path = temp_dir.join("test_window_info.json");

        // Clean up if exists
        let _ = fs::remove_file(&test_path);

        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);

        write_json(&json_info, &test_path).unwrap();

        assert!(test_path.exists());

        // Verify content is valid JSON
        let mut file = File::open(&test_path).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&contents).unwrap();
        assert_eq!(parsed["window_id"], 12345);
        assert_eq!(parsed["title"], "Test Window");

        // Clean up
        fs::remove_file(&test_path).unwrap();
    }

    #[test]
    fn test_write_json_creates_parent_directories() {
        let temp_dir = std::env::temp_dir();
        let nested_dir = temp_dir.join("snap_window_test").join("nested").join("dirs");
        let test_path = nested_dir.join("window.json");

        // Clean up if exists
        let _ = fs::remove_dir_all(&temp_dir.join("snap_window_test"));

        let info = create_test_window_info();
        let json_info = WindowInfoJson::from_window_info(&info);

        write_json(&json_info, &test_path).unwrap();

        assert!(nested_dir.exists());
        assert!(test_path.exists());

        // Clean up
        fs::remove_dir_all(&temp_dir.join("snap_window_test")).unwrap();
    }
}
