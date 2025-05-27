// CLI parsing (clap)

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "watchdogfs")]
#[command(about = "Lightweight filesystem integrity monitor", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize configuration for specified files/directories
    Init { files: Vec<String> },

    /// Generate baseline SHA256 checksums
    Baseline,

    /// Start monitoring files (optionally as daemon)
    Start {
        #[arg(short, long)]
        daemon: bool,
    },
}

pub fn parse() -> Cli {
    Cli::parse()
}