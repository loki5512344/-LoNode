//! Internet radio source: Icecast / Shoutcast HTTP streams with ICY metadata.
//!
//! `supports()` accepts any `http(s)://` URL that isn't a YouTube link.
//! `resolve()` issues a GET with `Icy-MetaData: 1` and extracts `icy-name` /
//! `icy-genre` from response headers. `stream()` returns the response body
//! as an `AsyncRead` for downstream PCM decoding.

use async_trait::async_trait;
use futures_util::StreamExt;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use reqwest::Client;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

/// HTTP-based internet radio source (Icecast/Shoutcast).
pub struct RadioSource {
    client: Client,
}

impl Default for RadioSource {
    fn default() -> Self {
        Self::new()
    }
}

impl RadioSource {
    #[must_use]
    pub fn new() -> Self {
        let client = Client::builder().build().unwrap_or_else(|_| Client::new());
        Self { client }
    }
}

#[async_trait]
impl AudioSource for RadioSource {
    fn name(&self) -> &str {
        "radio"
    }

    fn supports(&self, url: &str) -> bool {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return false;
        }
        // Reject URLs claimed by other built-in sources (they take priority).
        let lower = url.to_lowercase();
        for domain in [
            "youtube.com",
            "youtu.be",
            "soundcloud.com",
            "spotify.com",
            "bandcamp.com",
            "twitch.tv",
            "vimeo.com",
        ] {
            if lower.contains(domain) {
                return false;
            }
        }
        true
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        let resp = self
            .client
            .get(url)
            .header("Icy-MetaData", "1")
            .send()
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;

        let headers = resp.headers();
        let title = header_str(headers, "icy-name")
            .unwrap_or("Unknown Station")
            .to_string();
        let author = header_str(headers, "icy-genre")
            .unwrap_or("Icecast/Shoutcast")
            .to_string();

        Ok(TrackInfo {
            title,
            author,
            duration_ms: 0,
            url: url.to_string(),
        })
    }

    async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        let resp = self
            .client
            .get(url)
            .header("Icy-MetaData", "1")
            .send()
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(PluginError::Stream(format!("HTTP {}", resp.status())));
        }

        let stream = resp
            .bytes_stream()
            .map(|r| r.map_err(|e| std::io::Error::other(e.to_string())));
        Ok(Box::new(StreamReader::new(stream)))
    }
}

fn header_str<'a>(headers: &'a reqwest::header::HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|v| v.to_str().ok())
}
