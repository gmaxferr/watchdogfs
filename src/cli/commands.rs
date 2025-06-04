// CLI commands logic
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize monitoring for files/directories
    Init {
        /// Path to create config at (default: ./config.yaml)
        #[arg(short, long, default_value = "config.yaml")]
        config: String,
        
        /// Also generate per‐job baseline files immediately
        #[arg(long)]
        with_baseline: bool,
    },

    /// Generate baseline checksums
    Baseline,

    /// Start monitoring process
    Start {
        #[arg(short, long)]
        daemon: bool,
    },
}
