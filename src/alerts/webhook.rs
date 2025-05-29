// HTTP webhook alerts

use anyhow::Result;
use reqwest::blocking::Client;

/// Send a JSON payload via HTTP POST to `url`.
pub fn send_webhook(url: &str, payload: &str) -> Result<()> {
    let client = Client::new();
    client
        .post(url)
        .header("Content-Type", "application/json")
        .body(payload.to_string())
        .send()?
        .error_for_status()?; // treat 4xx/5xx as errors
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::send_webhook;

    #[test]
    fn invalid_url_errors() {
        // Any malformed URL should return Err
        assert!(send_webhook("not-a-valid-url", "{}").is_err());
    }

    #[test]
    fn connection_refused_errors() {
        // Unbound port should return Err
        assert!(send_webhook("http://127.0.0.1:0", "{}").is_err());
    }
}
