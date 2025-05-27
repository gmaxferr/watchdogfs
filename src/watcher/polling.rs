// Polling fallback implementation
use std::{thread, time};
use std::fs::metadata;
use anyhow::Result;

pub fn poll(path: &str, interval: u64) -> Result<()> {
    loop {
        let meta = metadata(path)?;
        println!("Checked {}: {:?}", path, meta.modified()?);
        thread::sleep(time::Duration::from_secs(interval));
    }
}
