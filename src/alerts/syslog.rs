// Syslog alerts
use anyhow::Result;
use syslog::{Facility, Formatter3164};

/// Send a one-line message to the local syslog.
pub fn send_syslog(message: &str) -> Result<()> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "watchdogfs".into(),
        pid: 0,
    };

    let mut writer = syslog::unix(formatter)?;
    writer.info(message)?;
    Ok(())
}
