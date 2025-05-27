// SHA256 checksum logic
use sha2::{Sha256, Digest};
use std::fs;
use anyhow::Result;

pub fn calculate_checksum(path: &str) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}
