//! Deezer resolver — `AudioSource` impl + URL parsing + native preview stream.

use crate::client::DeezerClient;
use async_trait::async_trait;
use futures_util::StreamExt;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;
use tokio_util::io::StreamReader;

/// Deezer audio source. Resolves via Deezer API, streams the 30-second preview
/// (full streaming requires a Premium token — not yet implemented).
pub struct DeezerSource {
    client: DeezerClient,
}

impl Default for DeezerSource {
    fn default() -> Self {
        Self::new()
    }
}

impl DeezerSource {
    #[must_use]
    pub fn new() -> Self {
        Self {
            client: DeezerClient::new(),
        }
    }

    /// `true` if `url` is a Deezer track URL.
    #[must_use]
    pub fn is_deezer_url(url: &str) -> bool {
        url.contains("deezer.com/track/")
    }
}

#[async_trait]
impl AudioSource for DeezerSource {
    fn name(&self) -> &str {
        "deezer"
    }

    fn supports(&self, url: &str) -> bool {
        Self::is_deezer_url(url)
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        let track_id = extract_track_id(url)
            .ok_or_else(|| PluginError::Resolve(format!("invalid deezer url: {url}")))?;
        let raw = self
            .client
            .get_track(track_id)
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        Ok(TrackInfo {
            title: raw.title,
            author: raw.artist,
            duration_ms: raw.duration * 1000,
            url: url.to_string(),
        })
    }

    async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        let track_id = extract_track_id(url)
            .ok_or_else(|| PluginError::Stream(format!("invalid deezer url: {url}")))?;
        let raw = self
            .client
            .get_track(track_id)
            .await
            .map_err(|e| PluginError::Stream(e.to_string()))?;
        if raw.preview.is_empty() {
            return Err(PluginError::Stream("no preview URL for this track".into()));
        }
        let resp = self
            .client
            .http()
            .get(&raw.preview)
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

/// Extract the numeric track ID from a Deezer URL.
#[must_use]
pub fn extract_track_id(url: &str) -> Option<&str> {
    let pos = url.find("deezer.com/track/")?;
    let rest = &url[pos + "deezer.com/track/".len()..];
    Some(rest.split('?').next().unwrap_or(rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_deezer_urls() {
        assert!(DeezerSource::is_deezer_url(
            "https://www.deezer.com/track/123456"
        ));
        assert!(!DeezerSource::is_deezer_url("https://example.com"));
    }

    #[test]
    fn extracts_track_id() {
        assert_eq!(
            extract_track_id("https://www.deezer.com/track/123456?utm_source=x"),
            Some("123456")
        );
    }

    #[test]
    fn returns_none_for_non_deezer_url() {
        assert_eq!(extract_track_id("https://example.com"), None);
    }

    #[test]
    fn name_is_deezer() {
        assert_eq!(DeezerSource::new().name(), "deezer");
    }
}
