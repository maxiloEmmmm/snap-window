use anyhow::Result;
use clap::Parser;

mod cli;
mod error;
mod platform;
mod window;

use cli::{resolve_output_path, Cli, Mode};

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Resolve output path with timestamped default if not provided
    let output_path = resolve_output_path(cli.output);

    match cli.mode {
        Mode { list: true, .. } => {
            let windows = platform::list_windows()?;
            for window in windows {
                println!("{}", window);
            }
            // Show resolved output path for informational purposes
            eprintln!("\nDefault output path (when not specified): {}", output_path.display());
        }
        Mode { window: Some(name), .. } => {
            println!("Window targeting by name: {} (not implemented)", name);
            println!("Output path: {}", output_path.display());
        }
        Mode { pid: Some(pid), .. } => {
            println!("Window targeting by PID: {} (not implemented)", pid);
            println!("Output path: {}", output_path.display());
        }
        Mode { index: Some(index), .. } => {
            println!("Window targeting by index: {} (not implemented)", index);
            println!("Output path: {}", output_path.display());
        }
        Mode { highlight: Some(index), .. } => {
            println!("Highlight window at index: {} (not implemented)", index);
        }
        _ => {
            unreachable!("Clap should enforce exactly one mode");
        }
    }

    Ok(())
}
