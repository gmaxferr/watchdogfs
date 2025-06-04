// CLI parsing (clap)
mod commands;
pub use commands::*;

use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[command(name = "watchdogfs")]
#[command(about = "Filesystem integrity monitor", version)]
pub struct Cli {
    /// Path to your config file
    #[arg(short, long, default_value = "config.yaml")]
    pub config: String,

    /// Optional path to a file containing the expected SHA256 of this binary
    #[arg(long)]
    pub self_integrity_path: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

impl Cli {
    /// Helper to refer to the config file path
    pub fn config_path(&self) -> &str {
        &self.config
    }

    /// Helper to refer to the self-integrity path, if any
    pub fn self_integrity_path(&self) -> Option<&str> {
        self.self_integrity_path.as_deref()
    }
}

/// Wrapper to invoke clap
pub fn parse() -> Cli {
    Cli::parse()
}

pub fn init_command(path: &str) -> Result<()> {
    let p = std::path::Path::new(path);
    if p.exists() {
        anyhow::bail!("Config file {} already exists", path);
    }
    crate::config::write_default(p)?;
    println!("✅ Created new config at {}", path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Cli;
    use clap::Parser;

    #[test]
    fn default_config_and_no_selfcheck() {
        // note: subcommands are lowercase
        let args = Cli::parse_from(&["watchdogfs", "baseline"]);
        assert_eq!(args.config_path(), "config.yaml");
        assert!(args.self_integrity_path().is_none());
        match args.command {
            super::Commands::Baseline => (),
            _ => panic!("expected Baseline command"),
        }
    }

    #[test]
    fn custom_config_and_selfcheck() {
        let args = Cli::parse_from(&[
            "watchdogfs",
            "-c",
            "custom.yaml",
            "--self-integrity-path",
            "me.sha256",
            "start",
            "--daemon",
        ]);
        assert_eq!(args.config_path(), "custom.yaml");
        assert_eq!(args.self_integrity_path(), Some("me.sha256"));
        match args.command {
            super::Commands::Start { daemon } => assert!(daemon),
            _ => panic!("expected Start command"),
        }
    }

    #[test]
    fn init_command_parses() {
        // Old: ["watchdogfs", "init", "-c", "foo.yaml"]
        // Now we must provide at least the boolean (even if false), or adjust the signature.
        // By default, `with_baseline` is false, so tests can stay the same:

        let args = Cli::parse_from(&["watchdogfs", "init", "-c", "foo.yaml"]);
        match args.command {
            super::Commands::Init {
                config,
                with_baseline,
            } => {
                assert_eq!(config, "foo.yaml");
                assert!(!with_baseline);
            }
            _ => panic!("expected Init command"),
        }
    }

    #[test]
    fn init_with_baseline_parses() {
        let args = Cli::parse_from(&["watchdogfs", "init", "-c", "foo.yaml", "--with-baseline"]);
        match args.command {
            super::Commands::Init {
                config,
                with_baseline,
            } => {
                assert_eq!(config, "foo.yaml");
                assert!(with_baseline);
            }
            _ => panic!("expected Init command"),
        }
    }
}
