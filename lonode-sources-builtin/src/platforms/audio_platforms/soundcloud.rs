//! SoundCloud source — resolves track URLs via SoundCloud's public API.
//!
//! SoundCloud exposes a public resolve endpoint that redirects to the
//! track's streamable media URL. We follow the redirect and return the
//! body as an `AsyncRead` for downstream PCM decoding.
//!
//! Note: SoundCloud requires a `client_id` for the resolve API. For now
//! `supports()` returns `false` until a client_id is configured; the
//! registry will skip it. Real implementation will accept a client_id
//! via constructor.

use async_trait::async_trait;
use futures_util::StreamExt;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use reqwest::Client;
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

/// SoundCloud source (currently disabled — needs client_id).
pub struct SoundCloudSource {
    client: Client,
    client_id: Option<String>,
}

impl Default for SoundCloudSource {
    fn default() -> Self {
        Self::new()
    }
}

impl SoundCloudSource {
    #[must_use]
    pub fn new() -> Self {
        let client = Client::builder().build().unwrap_or_else(|_| Client::new());
        Self {
            client,
            client_id: None,
        }
    }

    /// Configure with a SoundCloud `client_id` (enables `supports()`).
    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// `true` if `url` points at a SoundCloud track.
    #[must_use]
    pub fn is_soundcloud_url(url: &str) -> bool {
        url.contains("soundcloud.com/") && !url.contains("soundcloud.com/stream")
    }

    fn enabled(&self) -> bool {
        self.client_id.is_some()
    }
}

#[async_trait]
impl AudioSource for SoundCloudSource {
    fn name(&self) -> &str {
        "soundcloud"
    }

    fn supports(&self, url: &str) -> bool {
        self.enabled() && Self::is_soundcloud_url(url)
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        if !self.enabled() {
            return Err(PluginError::Resolve("soundcloud client_id not set".into()));
        }
        let resolve_url = format!(
            "https://api.soundcloud.com/resolve?url={}&client_id={}",
            url,
            self.client_id.as_ref().unwrap()
        );
        let resp = self
            .client
            .get(&resolve_url)
            .send()
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        Ok(TrackInfo {
            title: json
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .into(),
            author: json
                .pointer("/user/username")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown Artist")
                .into(),
            duration_ms: json.get("duration").and_then(|v| v.as_u64()).unwrap_or(0),
            url: url.to_string(),
        })
    }

    async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        if !self.enabled() {
            return Err(PluginError::Stream("soundcloud client_id not set".into()));
        }
        // Resolve → get media URL → stream it.
        let resolve_url = format!(
            "https://api.soundcloud.com/resolve?url={}&client_id={}",
            url,
            self.client_id.as_ref().unwrap()
        );
        let resp = self
            .client
            .get(&resolve_url)
            .send()
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        let media_url = json
            .pointer("/stream_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| PluginError::Stream("no stream_url in soundcloud response".into()))?;
        let stream_url = format!("{media_url}?client_id={}", self.client_id.as_ref().unwrap());
        let resp = self
            .client
            .get(&stream_url)
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
