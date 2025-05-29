// Structured logging setup (tracing)
use anyhow::Result;
use tracing_subscriber::EnvFilter;

pub fn init() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    Ok(())
}
