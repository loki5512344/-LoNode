//! Bandcamp + YouTube stub sources.
//!
//! Both return `supports() = false` until real implementations are added
//! (bandcamp-dl / rusty-ytdl). The types exist now so the registry can list
//! them in `/v4/info` and the API surface is stable.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

const BANDCAMP_NOT_IMPL: &str = "Bandcamp support not yet implemented";
const YOUTUBE_NOT_IMPL: &str = "YouTube support not yet implemented (planned: rusty-ytdl)";

/// Bandcamp source (currently disabled).
pub struct BandcampSource;

impl Default for BandcampSource {
    fn default() -> Self {
        Self
    }
}

impl BandcampSource {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn is_bandcamp_url(url: &str) -> bool {
        url.contains("bandcamp.com/track/") || url.contains("bandcamp.com/album/")
    }
}

#[async_trait]
impl AudioSource for BandcampSource {
    fn name(&self) -> &str {
        "bandcamp"
    }

    fn supports(&self, _url: &str) -> bool {
        false
    }

    async fn resolve(&self, _url: &str) -> Result<TrackInfo, PluginError> {
        Err(PluginError::Resolve(BANDCAMP_NOT_IMPL.into()))
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(BANDCAMP_NOT_IMPL.into()))
    }
}
/// YouTube source — stub.
///
/// `supports()` returns `false` so the registry skips it. Real implementation
/// will use `rusty-ytdl` (Phase 3 follow-up). The type exists now so the
/// registry can list it in `/v4/info` and the API surface is stable.
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
        Err(PluginError::Resolve(YOUTUBE_NOT_IMPL.into()))
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(YOUTUBE_NOT_IMPL.into()))
    }
}
