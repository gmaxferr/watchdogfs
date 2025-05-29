use anyhow::Result;
use watchdogfs::{
    cli,
    config::{self, Config},
    integrity, logger, selfcheck, watcher,
};

fn main() -> Result<()> {
    logger::init()?;
    let args = cli::parse();

    // Before anything else: self-integrity check
    if let Some(_path) = &args.self_integrity_path() {
        // assume you expose a helper in cli to get the config location
        let cfg: Config = crate::config::load("config.yaml")?;
        if let Some(sip) = &cfg.self_integrity_path {
            selfcheck::verify(sip)?;
        }
    }

    match args.command {
        cli::Commands::Init { config } => crate::config::write_default(&config)?,
        cli::Commands::Baseline => {
            integrity::generate_baseline()?;
        }
        cli::Commands::Start { daemon } => watcher::start(daemon)?,
    }

    Ok(())
}
