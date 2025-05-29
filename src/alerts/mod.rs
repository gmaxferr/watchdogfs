//! Alerting subsystem: syslog, HTTP webhook, or local script.

mod local_script;
mod syslog;
mod webhook;

pub use local_script::execute_script;
pub use syslog::send_syslog;
pub use webhook::send_webhook;

use crate::config::AlertsConfig;
use serde_json::json;
use std::path::Path;
use tracing::error;

/// Dispatch a file-change alert to all enabled channels.
pub fn dispatch(cfg: &AlertsConfig, path: &Path, old: String, new: String) {
    let path_str = path.display().to_string();
    let payload = json!({
        "path": path_str,
        "old": old,
        "new": new,
    })
    .to_string();

    // 1) Syslog
    if cfg.use_syslog {
        let msg = format!("Integrity change: {}", payload);
        if let Err(e) = send_syslog(&msg) {
            error!("syslog alert failed: {}", e);
        }
    }

    // 2) Webhook
    if let Some(url) = &cfg.webhook_url {
        if let Err(e) = send_webhook(url, &payload) {
            error!("webhook alert to {} failed: {}", url, e);
        }
    }

    // 3) Local script
    if let Some(script) = &cfg.script_path {
        if let Err(e) = execute_script(script) {
            error!("script alert `{}` failed: {}", script, e);
        }
    }
}
