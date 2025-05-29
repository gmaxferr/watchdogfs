// Config structures
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub watch_paths: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub alerts: AlertsConfig,
    pub watcher: WatcherConfig,

    /// Optional path to a file containing the expected SHA256 of this binary.
    pub self_integrity_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            watch_paths: Vec::new(),
            ignore_patterns: Vec::new(),
            alerts: AlertsConfig::default(),
            watcher: WatcherConfig::default(),
            self_integrity_path: None,
        }
    }
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AlertsConfig {
    pub webhook_url: Option<String>,
    pub script_path: Option<String>,
    pub use_syslog: bool,
}

impl Default for AlertsConfig {
    fn default() -> Self {
        AlertsConfig {
            webhook_url: None,
            script_path: None,
            use_syslog: false,
        }
    }
}
