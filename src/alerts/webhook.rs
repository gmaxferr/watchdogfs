// HTTP webhook alerts
use anyhow::Result;
use reqwest::blocking::Client;

pub fn send_webhook(url: &str, payload: &str) -> Result<()> {
    let client = Client::new();
    client.post(url).body(payload.to_string()).send()?;
    Ok(())
}

