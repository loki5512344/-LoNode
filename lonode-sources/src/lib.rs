//! `lonode-sources` — built-in audio sources for LoNode.
//!
//! Provides:
//! - [`RadioSource`] — Icecast/Shoutcast HTTP streams with ICY metadata.
//! - [`YoutubeSource`] — stub (rusty-ytdl integration planned).
//! - [`SourceRegistry`] — finds the right source for a URL.

pub mod radio;
pub mod youtube;

pub use radio::RadioSource;
pub use youtube::YoutubeSource;

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

    /// Build a registry pre-populated with the default built-in sources
    /// (radio first, then youtube).
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut r = Self::new();
        r.register(Arc::new(RadioSource::new()));
        r.register(Arc::new(YoutubeSource::new()));
        r
    }

    /// Find the first source that supports `url`.
    #[must_use]
    pub fn find_for(&self, url: &str) -> Option<Arc<dyn AudioSource>> {
        self.sources.iter().find(|s| s.supports(url)).cloned()
    }

    /// Number of registered sources.
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
        assert_eq!(r.len(), 2);
        let names = r.source_names();
        assert_eq!(names, vec!["radio", "youtube"]);
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
