use chrono::Local;
use clap::{Args, Parser};
use std::path::PathBuf;

/// Cross-platform CLI window screenshot tool
///
/// Capture screenshots of any visible window using simple CLI commands.
/// List windows, target by name/PID/index, and save as PNG.
#[derive(Parser, Debug)]
#[command(name = "snap-window")]
#[command(version)]
#[command(about = "Capture screenshots of application windows", long_about = None)]
pub struct Cli {
    /// Output file path
    #[arg(short, long, value_name = "PATH")]
    pub output: Option<PathBuf>,

    /// Targeting mode (exactly one required)
    #[command(flatten)]
    pub mode: Mode,
}

/// Targeting mode - exactly one must be specified
#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
pub struct Mode {
    /// Target window by substring match on title
    #[arg(short, long, value_name = "NAME")]
    pub window: Option<String>,

    /// Target window by process ID
    #[arg(short, long, value_name = "PID")]
    pub pid: Option<u32>,

    /// Target window by index from list
    #[arg(short, long, value_name = "INDEX")]
    pub index: Option<usize>,

    /// List all windows with indices
    #[arg(short, long)]
    pub list: bool,

    /// Highlight window with red border (no screenshot)
    #[arg(long, value_name = "INDEX")]
    pub highlight: Option<usize>,
}

/// Resolve the output path, generating a timestamped default if none provided
///
/// When `cli_output` is None, generates a filename in the format:
/// `screenshot_YYYYMMDD_HHMMSS.png` using the current local time.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// // With user-provided path
/// let path = resolve_output_path(Some(PathBuf::from("my_capture.png")));
/// assert_eq!(path, PathBuf::from("my_capture.png"));
/// ```
pub fn resolve_output_path(cli_output: Option<PathBuf>) -> PathBuf {
    cli_output.unwrap_or_else(|| {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        PathBuf::from(format!("screenshot_{}.png", timestamp))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_resolve_output_path_with_user_path() {
        let user_path = PathBuf::from("my_screenshot.png");
        let result = resolve_output_path(Some(user_path.clone()));
        assert_eq!(result, user_path);
    }

    #[test]
    fn test_resolve_output_path_generates_timestamp() {
        let result = resolve_output_path(None);
        let filename = result.file_name().unwrap().to_str().unwrap();

        // Should start with "screenshot_" and end with ".png"
        assert!(filename.starts_with("screenshot_"));
        assert!(filename.ends_with(".png"));

        // Should contain timestamp in format YYYYMMDD_HHMMSS
        // screenshot_ (11) + YYYYMMDD_HHMMSS (15) + .png (4) = 30
        assert_eq!(filename.len(), 30);
    }

    #[test]
    fn test_resolve_output_path_timestamp_format() {
        let result = resolve_output_path(None);
        let filename = result.file_name().unwrap().to_str().unwrap();

        // Extract timestamp portion: screenshot_YYYYMMDD_HHMMSS.png
        let timestamp_part = &filename[11..filename.len()-4]; // Skip "screenshot_" and ".png"

        // Should be exactly 15 characters: YYYYMMDD_HHMMSS
        assert_eq!(timestamp_part.len(), 15);

        // Should contain underscore at position 8 (after YYYYMMDD)
        assert_eq!(&timestamp_part[8..9], "_");

        // All other characters should be digits
        for (i, c) in timestamp_part.chars().enumerate() {
            if i != 8 {
                assert!(c.is_ascii_digit(), "Character at position {} should be a digit", i);
            }
        }
    }

    #[test]
    fn test_resolve_output_path_consecutive_calls_differ() {
        let path1 = resolve_output_path(None);

        // Sleep briefly to ensure different timestamp
        thread::sleep(Duration::from_millis(1100));

        let path2 = resolve_output_path(None);

        // Paths should be different (different timestamps)
        assert_ne!(path1, path2);
    }

    #[test]
    fn test_resolve_output_path_preserves_absolute_path() {
        let abs_path = PathBuf::from("/tmp/captures/window.png");
        let result = resolve_output_path(Some(abs_path.clone()));
        assert_eq!(result, abs_path);
    }

    #[test]
    fn test_resolve_output_path_preserves_relative_path() {
        let rel_path = PathBuf::from("../output/capture.png");
        let result = resolve_output_path(Some(rel_path.clone()));
        assert_eq!(result, rel_path);
    }
}
