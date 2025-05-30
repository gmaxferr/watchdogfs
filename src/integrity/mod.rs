// Integrity checking module

mod baseline;
mod checksum;

use crate::config::Config;
use anyhow::{Context, Result};
use serde_json;
use serde_yaml;
use std::{fs, path::Path};

pub use baseline::{Baseline, generate as generate_map};
pub use checksum::*;

pub fn init(_files: Vec<String>) -> Result<()> {
    // TODO: implement `watchdogfs init` logic (e.g. write config.yaml)
    Ok(())
}

pub fn generate_baseline() -> Result<Baseline> {
    // 1) Load & parse config.yaml
    let cfg_str = fs::read_to_string("config.yaml")
        .context("Failed to read `config.yaml` in current directory")?;
    let cfg: Config =
        serde_yaml::from_str(&cfg_str).context("Failed to parse `config.yaml` as YAML")?;

    // 2) Generate the baseline map
    let baseline_map = generate_map(&cfg.watch_paths)?;

    // 3) Serialize & write to baseline.json
    let out_path = Path::new("baseline.json").to_path_buf();
    let json = serde_json::to_string_pretty(&baseline_map)
        .context("Failed to serialize baseline to JSON")?;
    fs::write(out_path.as_path(), json)
        .with_context(|| format!("Failed to write baseline to {}", out_path.display()))?;

    println!("✅ Baseline generated and saved to {}", out_path.display());
    Ok(baseline_map)
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
