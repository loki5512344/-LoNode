//! Shared types for the plugin API.

/// Metadata for a resolved track.
///
/// Returned by [`AudioSource::resolve`](crate::AudioSource::resolve) after a
/// URL has been accepted by [`AudioSource::supports`](crate::AudioSource::supports).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrackInfo {
    /// Track title (best-effort; may be empty for live streams).
    pub title: String,
    /// Author, artist, or station name.
    pub author: String,
    /// Duration in milliseconds (`0` for live streams).
    pub duration_ms: u64,
    /// Canonical URL of the streamable resource.
    pub url: String,
}

/// Error type returned by all [`AudioSource`](crate::AudioSource) methods.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// The source does not support this URL.
    #[error("unsupported URL: {0}")]
    Unsupported(String),
    /// Metadata resolution failed (network, parse, …).
    #[error("resolve failed: {0}")]
    Resolve(String),
    /// Stream opening or reading failed.
    #[error("stream failed: {0}")]
    Stream(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_info_default_is_empty() {
        let t = TrackInfo::default();
        assert!(t.title.is_empty());
        assert!(t.author.is_empty());
        assert_eq!(t.duration_ms, 0);
        assert!(t.url.is_empty());
    }

    #[test]
    fn plugin_error_displays() {
        let e = PluginError::Unsupported("ftp://x".into());
        assert_eq!(e.to_string(), "unsupported URL: ftp://x");
    }
}
