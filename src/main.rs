mod cli;

use clap::Parser;
use cli::Cli;

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    println!("snap-window - CLI parsing successful");
    Ok(())
}
