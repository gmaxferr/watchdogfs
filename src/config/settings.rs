// Config structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
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
    /// If set, send event via an HTTP POST (JSON) to this URL
    pub webhook_url: Option<String>,

    /// If set, execute a local script (no args) on alert
    pub script_path: Option<String>,

    /// If true, emit a syslog message
    pub use_syslog: bool,
    
    /// If set, load this shared library and invoke its `run_alert` function
    /// with the JSON payload (C‐ABI: `fn run_alert(payload: *const c_char) -> i32`).
    pub plugin_path: Option<String>,

    /// An optional Liquid template (as a string) to render the JSON payload.
    /// Available variables: `path`, `old`, `new`.
    pub payload_template: Option<String>,
}
