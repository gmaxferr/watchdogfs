// YAML configuration parsing (serde_yaml)
mod settings;
pub use settings::{Config, AlertsConfig};

use std::{fs, path::Path};
use anyhow::{Context, Result};
use serde_yaml;

pub fn write_default<P: AsRef<Path>>(path: P) -> Result<()> {
    let cfg: Config = Config::default();
    let yaml = serde_yaml::to_string(&cfg)
        .context("Failed to serialize default Config")?;
    fs::write(&path, yaml)
        .with_context(|| format!("Unable to write config to {:?}", path.as_ref()))?;
    println!("✅ Created new config at {:?}", path.as_ref());
    Ok(())
}

pub fn load<P: AsRef<Path>>(path: P) -> Result<Config> {
    let s = fs::read_to_string(&path)
        .with_context(|| format!("reading config file {:?}", path.as_ref()))?;
    let cfg: Config = serde_yaml::from_str(&s)
        .context("parsing YAML config")?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::{write_default, load};
    use tempfile::NamedTempFile;

    #[test]
    fn write_and_load_roundtrip() {
        let tmp = NamedTempFile::new().unwrap();
        let path = tmp.path();
        write_default(path).unwrap();
        let cfg = load(path).unwrap();
        // Default Config has no watch_paths and no alerts
        assert!(cfg.watch_paths.is_empty());
        assert!(!cfg.alerts.use_syslog);
        // The YAML we wrote should parse back to the same structure
    }
}