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
