use anyhow::{Context, Result};
use clap::Parser;

mod capture_service;
mod cli;
mod error;
pub mod platform;
mod window;
mod window_service;

use cli::{resolve_output_path, Cli, Mode};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Resolve output path with timestamped default if not provided
    let output_path = resolve_output_path(cli.output);

    match cli.mode {
        Mode { list: true, .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;
            for window in windows {
                println!("{}", window);
            }
            // Show resolved output path for informational purposes
            eprintln!("\nDefault output path (when not specified): {}", output_path.display());
        }
        Mode { window: Some(name), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            match window_service::find_by_name(&windows, &name) {
                Ok(w) => {
                    capture_service::capture_window(w, &output_path)?;
                    println!("Saved screenshot to: {}", output_path.display());
                }
                Err(e) => {
                    window_service::print_available_windows(&windows);
                    return Err(e.into());
                }
            }
        }
        Mode { pid: Some(pid), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            match window_service::find_by_pid(&windows, pid) {
                Ok(w) => {
                    capture_service::capture_window(w, &output_path)?;
                    println!("Saved screenshot to: {}", output_path.display());
                }
                Err(e) => {
                    window_service::print_available_windows(&windows);
                    return Err(e.into());
                }
            }
        }
        Mode { index: Some(index), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            match window_service::find_by_index(&windows, index) {
                Ok(w) => {
                    capture_service::capture_window(w, &output_path)?;
                    println!("Saved screenshot to: {}", output_path.display());
                }
                Err(e) => {
                    window_service::print_available_windows(&windows);
                    return Err(e.into());
                }
            }
        }
        Mode { highlight: Some(index), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            if index >= windows.len() {
                return Err(crate::error::AppError::invalid_index(index, windows.len() - 1))
                    .with_context(|| {
                        eprintln!("\nAvailable windows:");
                        for window in &windows {
                            eprintln!("  {}", window);
                        }
                        format!("Invalid window index {} for highlight", index)
                    });
            }

            let window = &windows[index];
            println!("Would highlight window: {}", window);
        }
        _ => {
            unreachable!("Clap should enforce exactly one mode");
        }
    }

    Ok(())
}
