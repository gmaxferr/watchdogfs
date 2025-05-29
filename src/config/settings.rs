// Config structures
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub alerts: AlertsConfig,
    pub watcher: WatcherConfig,

    /// Optional path to a file containing the expected SHA256 of this binary.
    pub self_integrity_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WatcherConfig {
    pub mode: String,
    pub poll_interval: Option<u64>,
    pub debounce_ms: Option<u64>,
}

impl Default for WatcherConfig {
    fn default() -> Self {
        WatcherConfig {
            mode: "inotify".into(),
            poll_interval: Some(5),
            debounce_ms: Some(500),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct AlertsConfig {
    pub webhook_url: Option<String>,
    pub script_path: Option<String>,
    pub use_syslog: bool,
}
