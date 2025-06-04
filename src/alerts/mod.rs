//! Alerting subsystem: syslog, HTTP webhook, or local script.

mod local_script;
mod syslog;
mod webhook;

pub use local_script::execute_script;
pub use syslog::send_syslog;
pub use webhook::send_webhook;

use crate::config::AlertsConfig;
use liquid::{ParserBuilder, object};
use serde_json::json;
use std::path::Path;
use tracing::error;

/// Dispatch a file-change alert to all enabled channels.
/// If `cfg.payload_template` is `Some(tmpl)`, we try to render that Liquid template.
/// Otherwise, we default to the fixed JSON: { "path": “…”, "old": “…", "new": "…" }.
pub fn dispatch(cfg: &AlertsConfig, path: &Path, old: String, new: String) {
    let path_str = path.display().to_string();

    // 1) Build the payload string: either via Liquid or fallback to serde_json!
    let default_payload = json!({
        "path": path_str,
        "old": old,
        "new": new,
    })
    .to_string();

    let payload = if let Some(template_str) = &cfg.payload_template {
        // Try to compile & render the Liquid template
        match ParserBuilder::with_stdlib().build() {
            Ok(parser) => match parser.parse(template_str) {
                Ok(template) => {
                    // Create the Liquid "globals" object
                    let globals = object!({
                        "path": path_str.clone(),
                        "old": old.clone(),
                        "new": new.clone(),
                    });
                    match template.render(&globals) {
                        Ok(output) => output, // successfully rendered
                        Err(e) => {
                            error!("payload template render error: {}", e);
                            default_payload.clone()
                        }
                    }
                }
                Err(e) => {
                    error!("failed to parse payload_template: {}", e);
                    default_payload.clone()
                }
            },
            Err(e) => {
                error!("failed to initialize Liquid parser: {}", e);
                default_payload.clone()
            }
        }
    } else {
        default_payload
    };

    // 2) Syslog
    if cfg.use_syslog {
        let msg = format!("Integrity change: {}", payload);
        if let Err(e) = send_syslog(&msg) {
            error!("syslog alert failed: {}", e);
        }
    }

    // 3) Webhook
    if let Some(url) = &cfg.webhook_url {
        if let Err(e) = send_webhook(url, &payload) {
            error!("webhook alert to {} failed: {}", url, e);
        }
    }

    // 4) Local script
    if let Some(script) = &cfg.script_path {
        if let Err(e) = execute_script(script) {
            error!("script alert `{}` failed: {}", script, e);
        }
    }
}
