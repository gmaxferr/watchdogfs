// CLI parsing (clap)
mod commands;
pub use commands::*;

use clap::Parser;

#[derive(Parser)]
#[command(name = "watchdogfs")]
#[command(about = "Filesystem integrity monitor", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub fn parse() -> Cli {
    Cli::parse()
}
