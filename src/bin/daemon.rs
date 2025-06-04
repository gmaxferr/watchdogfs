use anyhow::{Context, Result};
use std::fs;
use watchdogfs::{
    config::Config,
    logger, selfcheck, watcher,
};

fn main() -> Result<()> {
    logger::init().context("Logger init")?;
    tracing::info!("Starting WatchdogFS daemon");

    let config = load_config("/etc/watchdogfs/config.yaml")?;
    tracing::info!("Config loaded (jobs): {:?}", config.jobs.keys());

    // Self‐integrity check (if provided)
    if let Some(sip) = &config.self_integrity_path {
        selfcheck::verify(sip)?;
    }

    // The watcher::start function now handles baseline creation/reading for each job
    watcher::start(true)?;
    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).context("Reading daemon config")?;
    let c: Config = serde_yaml::from_str(&s).context("Parsing daemon config")?;
    Ok(c)
}