// Config structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct JobConfig {
    /// Which paths this job should watch
    pub watch_paths: Vec<String>,

    /// Which glob patterns to ignore (not yet used but reserved)
    pub ignore_patterns: Vec<String>,

    /// Per‐job alert settings
    pub alerts: AlertsConfig,

    /// Per‐job watcher settings
    pub watcher: WatcherConfig,
}

impl Default for JobConfig {
    fn default() -> Self {
        JobConfig {
            watch_paths: Vec::new(),
            ignore_patterns: Vec::new(),
            alerts: AlertsConfig::default(),
            watcher: WatcherConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Config {
    /// A map from “job name” to its configuration
    pub jobs: HashMap<String, JobConfig>,

    /// Optional path to a file containing the expected SHA256 of this binary (self‐integrity)
    pub self_integrity_path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct AlertsConfig {
    pub webhook_url: Option<String>,
    pub script_path: Option<String>,
    pub use_syslog: bool,
}
