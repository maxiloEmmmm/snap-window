use anyhow::Result;
use clap::Parser;

mod cli;
mod error;
mod platform;
mod window;

use cli::{Cli, Mode};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.mode {
        Mode { list: true, .. } => {
            let windows = platform::list_windows()?;
            for window in windows {
                println!("{}", window);
            }
        }
        Mode { window: Some(name), .. } => {
            println!("Window targeting by name: {} (not implemented)", name);
        }
        Mode { pid: Some(pid), .. } => {
            println!("Window targeting by PID: {} (not implemented)", pid);
        }
        Mode { index: Some(index), .. } => {
            println!("Window targeting by index: {} (not implemented)", index);
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
