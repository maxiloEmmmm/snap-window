use anyhow::{Context, Result};
use clap::Parser;

mod cli;
mod error;
mod platform;
mod window;

use cli::{resolve_output_path, Cli, Mode};
use error::AppError;

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
            // Attempt to find window by name - for now, simulate not found to demonstrate error handling
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            // Check if window exists (mock: always fail for now to demonstrate error handling)
            let found = windows.iter().find(|w| w.title.contains(&name) || w.app_name.contains(&name));

            match found {
                Some(window) => {
                    println!("Found window: {}", window);
                    println!("Output path: {}", output_path.display());
                }
                None => {
                    // Return WindowNotFound error and trigger auto-list (ERR-02 requirement)
                    return Err(AppError::window_not_found(&name))
                        .with_context(|| {
                            // Print available windows before returning error
                            eprintln!("\nAvailable windows:");
                            for window in &windows {
                                eprintln!("  {}", window);
                            }
                            format!("Window '{}' not found", name)
                        });
                }
            }
        }
        Mode { pid: Some(pid), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            let found = windows.iter().find(|w| w.pid == pid);

            match found {
                Some(window) => {
                    println!("Found window: {}", window);
                    println!("Output path: {}", output_path.display());
                }
                None => {
                    return Err(AppError::window_not_found(format!("PID {}", pid)))
                        .with_context(|| {
                            eprintln!("\nAvailable windows:");
                            for window in &windows {
                                eprintln!("  {}", window);
                            }
                            format!("Window with PID {} not found", pid)
                        });
                }
            }
        }
        Mode { index: Some(index), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            if index >= windows.len() {
                return Err(AppError::invalid_index(index, windows.len() - 1))
                    .with_context(|| {
                        eprintln!("\nAvailable windows:");
                        for window in &windows {
                            eprintln!("  {}", window);
                        }
                        format!("Invalid window index {}", index)
                    });
            }

            let window = &windows[index];
            println!("Selected window: {}", window);
            println!("Output path: {}", output_path.display());
        }
        Mode { highlight: Some(index), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            if index >= windows.len() {
                return Err(AppError::invalid_index(index, windows.len() - 1))
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
