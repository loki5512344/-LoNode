//! Vimeo source — stub.
//!
//! Vimeo videos require extracting the direct MP4/HLS URL from the page.
//! `supports()` returns `false` until implemented.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

const NOT_IMPL: &str = "Vimeo support not yet implemented";

/// Vimeo source (currently disabled).
pub struct VimeoSource;

impl Default for VimeoSource {
    fn default() -> Self {
        Self
    }
}

impl VimeoSource {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    #[must_use]
    pub fn is_vimeo_url(url: &str) -> bool {
        url.contains("vimeo.com/")
            && !url.contains("player.vimeo.com")
            && url.split('/').filter(|s| !s.is_empty()).count() >= 2
    }
}

#[async_trait]
impl AudioSource for VimeoSource {
    fn name(&self) -> &str {
        "vimeo"
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
