//! Bandcamp stub source.
//!
//! Returns `supports() = false` until a real implementation (bandcamp-dl or
//! direct media URL extraction) is added. The type exists so the registry
//! can list it in `/v4/info` and the API surface is stable.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

const NOT_IMPL: &str = "Bandcamp support not yet implemented";

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
        Err(PluginError::Resolve(NOT_IMPL.into()))
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(NOT_IMPL.into()))
    }
}
