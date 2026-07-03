//! The plugin contract: any audio source (radio, YouTube, `.so` plugin)
//! implements this trait.

use crate::types::{PluginError, TrackInfo};
use async_trait::async_trait;
use tokio::io::AsyncRead;

/// Public contract for all LoNode audio sources.
///
/// The runner queries registered sources via [`supports`](AudioSource::supports)
/// and, on the first match, calls [`resolve`](AudioSource::resolve) for
/// metadata then [`stream`](AudioSource::stream) to open a byte stream of
/// compressed audio (MP3, Opus, HLS segments, …). Decoding to PCM is done
/// downstream by `lonode-core`'s Symphonia integration.
#[async_trait]
pub trait AudioSource: Send + Sync {
    /// Human-readable source name (e.g. `"radio"`, `"youtube"`).
    fn name(&self) -> &str;

    /// Return `true` if this source can handle `url`.
    fn supports(&self, url: &str) -> bool;

    /// Resolve `url` to metadata without downloading the stream.
    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError>;

    /// Open a byte stream of compressed audio for `url`.
    async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::empty;

    struct Dummy;

    #[async_trait]
    impl AudioSource for Dummy {
        fn name(&self) -> &str {
            "dummy"
        }
        fn supports(&self, _: &str) -> bool {
            false
        }
        async fn resolve(&self, _: &str) -> Result<TrackInfo, PluginError> {
            Ok(TrackInfo::default())
        }
        async fn stream(&self, _: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
            Ok(Box::new(empty()))
        }
    }

    #[tokio::test]
    async fn dummy_round_trips() {
        let s: Box<dyn AudioSource> = Box::new(Dummy);
        assert_eq!(s.name(), "dummy");
        assert!(!s.supports("x"));
        s.resolve("x").await.unwrap();
        s.stream("x").await.unwrap();
    }
}
