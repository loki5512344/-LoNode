//! Twitch source — stub.
//!
//! Twitch VODs/clips require HLS segment fetching + OAuth for some content.
//! `supports()` returns `false` until implemented.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

const NOT_IMPL: &str = "Twitch support not yet implemented (HLS + OAuth planned)";

/// Twitch source (currently disabled).
pub struct TwitchSource;

impl Default for TwitchSource {
    fn default() -> Self {
        Self
    }
}

impl TwitchSource {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn is_twitch_url(url: &str) -> bool {
        url.contains("twitch.tv/videos/")
            || url.contains("clips.twitch.tv/")
            || url.contains("twitch.tv/clip/")
    }
}

#[async_trait]
impl AudioSource for TwitchSource {
    fn name(&self) -> &str {
        "twitch"
    }

    fn supports(&self, _url: &str) -> bool {
        false
    }

    async fn resolve(&self, _url: &str) -> Result<TrackInfo, PluginError> {
        Err(PluginError::Resolve(NOT_IMPL.into()))
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(NOT_IMPL.into()))
    }
}
