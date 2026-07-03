//! `lonode-sources-builtin` — built-in audio sources for LoNode.
//!
//! Sources (in registration order):
//! - [`SoundCloudSource`] — SoundCloud tracks/playlists (API resolve).
//! - [`BandcampSource`] — Bandcamp track URLs (stub).
//! - [`TwitchSource`] — Twitch VODs/clips (stub).
//! - [`VimeoSource`] — Vimeo videos (stub).
//! - [`RadioSource`] — Icecast/Shoutcast HTTP streams with ICY metadata.
//!
//! Note: YouTube is in the separate `lonode-source-youtube` crate (real impl
//! via `rusty_ytdl`). Spotify is in `lonode-source-spotify`.
//! - [`SourceRegistry`] — finds the right source for a URL.

pub mod http;
pub mod platforms;

pub use http::RadioSource;
pub use platforms::{BandcampSource, SoundCloudSource, TwitchSource, VimeoSource};

use lonode_plugin_api::AudioSource;
use std::sync::Arc;

/// Registry of built-in sources. Finds the first source whose `supports()`
/// returns `true` for a given URL.
#[derive(Default)]
pub struct SourceRegistry {
    sources: Vec<Arc<dyn AudioSource>>,
}

impl SourceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a source. Sources are queried in registration order.
    pub fn register(&mut self, source: Arc<dyn AudioSource>) {
        self.sources.push(source);
    }

    /// Build a registry pre-populated with all default built-in sources.
    /// Registration order: platform sources first, radio LAST (fallback).
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut r = Self::new();
        r.register(Arc::new(SoundCloudSource::new()));
        r.register(Arc::new(BandcampSource::new()));
        r.register(Arc::new(TwitchSource::new()));
        r.register(Arc::new(VimeoSource::new()));
        r.register(Arc::new(RadioSource::new()));
        r
    }

    /// Find the first source that supports `url`.
    #[must_use]
    pub fn find_for(&self, url: &str) -> Option<Arc<dyn AudioSource>> {
        self.sources.iter().find(|s| s.supports(url)).cloned()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.sources.is_empty()
    }

    /// Names of all registered sources (for `/v4/info` capability reporting).
    #[must_use]
    pub fn source_names(&self) -> Vec<&str> {
        self.sources.iter().map(|s| s.name()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_registered_in_order() {
        let r = SourceRegistry::with_builtins();
        assert_eq!(r.len(), 5);
        let names = r.source_names();
        assert_eq!(
            names,
            vec!["soundcloud", "bandcamp", "twitch", "vimeo", "radio"]
        );
    }

    #[test]
    fn finds_radio_for_http_url() {
        let r = SourceRegistry::with_builtins();
        let s = r.find_for("http://stream.example.com/mp3");
        assert!(s.is_some());
        assert_eq!(s.unwrap().name(), "radio");
    }

    #[test]
    fn returns_none_for_unsupported_scheme() {
        let r = SourceRegistry::with_builtins();
        assert!(r.find_for("ftp://x").is_none());
    }

    #[test]
    fn empty_registry_finds_nothing() {
        let r = SourceRegistry::new();
        assert!(r.is_empty());
        assert!(r.find_for("http://x").is_none());
    }
}
