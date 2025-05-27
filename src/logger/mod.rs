// Structured logging setup (tracing)
use anyhow::Result;
use tracing_subscriber::{fmt, EnvFilter};

pub fn init() -> Result<()> {
    let subscriber = fmt()
        .json()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}