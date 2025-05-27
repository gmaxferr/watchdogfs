// CLI commands logic
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize monitoring for files/directories
    Init { files: Vec<String> },

    /// Generate baseline checksums
    Baseline,

    /// Start monitoring process
    Start {
        #[arg(short, long)]
        daemon: bool,
    },
}
