// Baseline generation & validation
use std::collections::HashMap;
use crate::integrity::calculate_checksum;
use anyhow::Result;

pub type Baseline = HashMap<String, String>;

pub fn generate(paths: &[String]) -> Result<Baseline> {
    let mut baseline = Baseline::new();
    for path in paths {
        let checksum = calculate_checksum(path)?;
        baseline.insert(path.clone(), checksum);
    }
    Ok(baseline)
}
