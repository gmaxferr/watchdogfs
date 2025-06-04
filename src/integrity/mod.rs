// Integrity checking module

mod baseline;
mod checksum;

use crate::config::{Config, write_default};
use anyhow::{Context, Result};
use serde_json;
use serde_yaml;
use std::fs;

pub use baseline::{Baseline, generate as generate_map};
pub use checksum::*;

/// To write a default config and auto‐generate all baselines.
pub fn init(files: Vec<String>) -> Result<()> {
    // files[0] might be the path to config.yaml
    // (depends on how the CLI is wired up)
    if files.is_empty() {
        anyhow::bail!("Expected at least one argument for `init`");
    }
    let config_path = &files[0];
    let path = std::path::Path::new(config_path);
    if path.exists() {
        anyhow::bail!("Config file {} already exists", config_path);
    }

    // 1) Create default config.yaml at `config_path`
    write_default(path)?;
    println!("✅ Created default config at {}", config_path);

    // 2) Immediately generate all the per‐job baseline files.
    generate_baseline()?;
    println!("✅ All baselines generated.");

    Ok(())
}

/// Generate one `baseline_<job_name>.json` file for every named job in `config.yaml`.
///
/// Old behavior (single `cfg.watch_paths`) has been replaced by looping over
/// `cfg.jobs` (each `JobConfig` contains its own `watch_paths`).
pub fn generate_baseline() -> Result<()> {
    // 1) Load & parse config.yaml
    let cfg_str = fs::read_to_string("config.yaml")
        .context("Failed to read `config.yaml` in current directory")?;
    let cfg: Config =
        serde_yaml::from_str(&cfg_str).context("Failed to parse `config.yaml` as YAML")?;

    // 2) For each job, generate (or regenerate) that job’s baseline
    for (job_name, job_cfg) in &cfg.jobs {
        // Generate the baseline map for this job’s watch_paths
        let baseline_map = generate_map(&job_cfg.watch_paths)
            .with_context(|| format!("Failed to generate baseline for job '{}'", job_name))?;

        // Serialize & write to `baseline_<job_name>.json`
        let filename = format!("baseline_{}.json", job_name);
        let json = serde_json::to_string_pretty(&baseline_map)
            .with_context(|| format!("Failed to serialize baseline for job '{}'", job_name))?;

        fs::write(&filename, json).with_context(|| {
            format!(
                "Failed to write baseline file `{}` for job '{}'",
                filename, job_name
            )
        })?;

        println!(
            "✅ Baseline for job '{}' generated and saved to {}",
            job_name, filename
        );
    }

    Ok(())
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
