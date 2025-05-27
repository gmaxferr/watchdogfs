// Config structures
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub alerts: AlertsConfig,
}

#[derive(Debug, Deserialize)]
pub struct AlertsConfig {
    pub webhook_url: Option<String>,
    pub script_path: Option<String>,
    pub use_syslog: bool,
}