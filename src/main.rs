use anyhow::{Context, Result};
use clap::Parser;

mod capture_service;
mod cli;
mod error;
pub mod highlight_service;
pub mod json_export;
pub mod platform;
mod window;
mod window_service;

use cli::{resolve_output_path, Cli, Mode};

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
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
        Mode { regexp: Some(pattern), .. } => {
            let windows = platform::list_windows()
                .context("Failed to enumerate windows")?;

            match window_service::find_by_regexp(&windows, &pattern) {
                Ok(matches) if matches.len() == 1 => {
                    capture_service::capture_window(matches[0], &output_path)?;
                    println!("Saved screenshot to: {}", output_path.display());
                }
                Ok(matches) if matches.len() > 1 => {
                    eprintln!("Multiple windows matched pattern '{}'.", pattern);
                    for w in &matches {
                        eprintln!("  [{}] {} (PID: {}, {})", w.index, w.title, w.pid, w.app_name);
                    }
                    eprintln!("\nUse --index to target a specific window.");
                    return Err(error::AppError::window_not_found(pattern).into());
                }
                Ok(_) => {
                    // Empty matches - no windows matched the pattern
                    window_service::print_available_windows(&windows);
                    return Err(error::AppError::window_not_found(pattern).into());
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

            // Validate index (highlight_service handles this too, but we need the window ref for JSON)
            if index >= windows.len() {
                window_service::print_available_windows(&windows);
                return Err(crate::error::AppError::invalid_index(index, windows.len().saturating_sub(1)).into());
            }

            let window = &windows[index];

            // Show visual highlight (overlay for ~3 seconds)
            // Errors are non-fatal -- log and continue to JSON export
            if let Err(e) = platform::show_highlight_border(window) {
                eprintln!("Warning: Could not display highlight border: {}", e);
            }

            // Export window info as JSON (this is the persistent output)
            let json_path = json_export::json_output_path(&output_path);
            let info_json = json_export::WindowInfoJson::from_window_info(window);
            json_export::write_json(&info_json, &json_path)?;

            println!("Saved window info to: {}", json_path.display());
        }
        _ => {
            unreachable!("Clap should enforce exactly one mode");
        }
    }

    Ok(())
}
