//! YouTube source — stub.
//!
//! `supports()` returns `false` so the registry skips it. Real implementation
//! will use `rusty-ytdl` (Phase 3 follow-up). The type exists now so the
//! registry can list it in `/v4/info` and the API surface is stable.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

const NOT_IMPL: &str = "YouTube support not yet implemented (planned: rusty-ytdl)";

/// YouTube source (currently disabled).
pub struct YoutubeSource;

impl Default for YoutubeSource {
    fn default() -> Self {
        Self
    }
}

impl YoutubeSource {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// `true` if `url` points at a YouTube watch page (used by `/v4/info`
    /// capability reporting, not by `supports`).
    #[must_use]
    pub fn is_youtube_url(url: &str) -> bool {
        url.contains("youtube.com/watch") || url.contains("youtu.be/")
    }
}

#[async_trait]
impl AudioSource for YoutubeSource {
    fn name(&self) -> &str {
        "youtube"
    }

    fn supports(&self, _url: &str) -> bool {
        // Disabled until rusty-ytdl is integrated.
        false
    }

    async fn resolve(&self, _url: &str) -> Result<TrackInfo, PluginError> {
        Err(PluginError::Resolve(NOT_IMPL.into()))
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(NOT_IMPL.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_until_implemented() {
        let y = YoutubeSource::new();
        assert!(!y.supports("https://youtube.com/watch?v=abc"));
        assert!(!y.supports("https://youtu.be/abc"));
    }

    #[test]
    fn detects_youtube_urls() {
        assert!(YoutubeSource::is_youtube_url(
            "https://youtube.com/watch?v=abc"
        ));
        assert!(YoutubeSource::is_youtube_url("https://youtu.be/abc"));
        assert!(!YoutubeSource::is_youtube_url("https://example.com"));
    }

    #[test]
    fn name_is_youtube() {
        assert_eq!(YoutubeSource::new().name(), "youtube");
    }
}
