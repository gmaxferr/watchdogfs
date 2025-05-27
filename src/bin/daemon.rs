use anyhow::{Context, Result};
use serde_yaml;
use std::{fs, path::PathBuf};
use watchdogfs::{
    config::Config,
    integrity::{Baseline, generate_map},
    logger,
    watcher,
};

fn main() -> Result<()> {
    logger::init().context("Logger init")?;
    tracing::info!("Starting WatchdogFS daemon");

    let config = load_config("/etc/watchdogfs/config.yaml")?;
    tracing::info!("Config loaded: {:?}", config);

    let baseline = load_or_create_baseline(&config)?;
    tracing::info!("Baseline ready: {} entries", baseline.len());

    watcher::start(true)?;
    Ok(())
}

fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).context("Reading daemon config")?;
    let c: Config = serde_yaml::from_str(&s).context("Parsing daemon config")?;
    Ok(c)
}

fn load_or_create_baseline(config: &Config) -> Result<Baseline> {
    let p = PathBuf::from("/var/lib/watchdogfs/baseline.json");
    if p.exists() {
        let s = fs::read_to_string(&p)?;
        let b: Baseline = serde_json::from_str(&s)?;
        Ok(b)
    } else {
        let b = generate_map(&config.watch_paths)?;
        let j = serde_json::to_string_pretty(&b)?;
        fs::write(&p, j)?;
        Ok(b)
    }
}
