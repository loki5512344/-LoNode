//! Apple Music resolver — `AudioSource` impl + URL parsing.

use crate::client::{AppleMusicClient, AppleMusicCredentials};
use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

/// Resolved Apple Music track + suggested YouTube search query.
#[derive(Debug, Clone)]
pub struct AppleMusicTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
    /// Suggested YouTube search string (`"{artist} - {title}"`).
    pub youtube_query: String,
}

/// Apple Music `AudioSource` — resolves metadata, delegates playback to YouTube.
pub struct AppleMusicSource {
    client: Option<AppleMusicClient>,
}

impl Default for AppleMusicSource {
    fn default() -> Self {
        Self::new()
    }
}

impl AppleMusicSource {
    #[must_use]
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Configure with Apple Music credentials (enables `supports()`).
    #[must_use]
    pub fn with_credentials(creds: AppleMusicCredentials) -> Self {
        Self {
            client: Some(AppleMusicClient::new(creds)),
        }
    }

    /// `true` if `url` is an Apple Music track/album URL.
    #[must_use]
    pub fn is_apple_music_url(url: &str) -> bool {
        url.contains("music.apple.com/") && (url.contains("/album/") || url.contains("/song/"))
    }

    fn enabled(&self) -> bool {
        self.client.is_some()
    }
}

/// Extract the track ID from `music.apple.com/{country}/album/{name}/{id}`.
#[must_use]
pub fn extract_track_id(url: &str) -> Option<&str> {
    // URL format: .../song/{id} or .../album/{name}/{id}?i={id}
    if let Some(pos) = url.find("?i=") {
        return url[pos + "?i=".len()..].split('&').next();
    }
    // Album URL: last numeric segment.
    url.split('/')
        .rfind(|s| s.chars().all(|c| c.is_ascii_digit()))
}

#[async_trait]
impl AudioSource for AppleMusicSource {
    fn name(&self) -> &str {
        "applemusic"
    }

    fn supports(&self, url: &str) -> bool {
        self.enabled() && Self::is_apple_music_url(url)
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| PluginError::Resolve("apple music credentials not set".into()))?;
        let track_id = extract_track_id(url)
            .ok_or_else(|| PluginError::Resolve(format!("invalid apple music url: {url}")))?;
        let raw = client
            .get_track(track_id)
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        Ok(TrackInfo {
            title: raw.attributes.name,
            author: raw.attributes.artist_name,
            duration_ms: raw.attributes.duration_ms,
            url: url.to_string(),
        })
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Err(PluginError::Stream(
            "apple music does not support direct streaming — use YouTube fallback (see resolve())"
                .into(),
        ))
    }
}

impl AppleMusicSource {
    /// Resolve to a full [`AppleMusicTrack`] (includes YouTube query).
    ///
    /// # Errors
    /// Returns an error if credentials aren't set or the API call fails.
    pub async fn resolve_full(&self, url: &str) -> Result<AppleMusicTrack, PluginError> {
        let info = self.resolve(url).await?;
        let youtube_query = format!("{} - {}", info.author, info.title);
        let id = extract_track_id(url).unwrap_or("").to_string();
        Ok(AppleMusicTrack {
            id,
            title: info.title,
            artist: info.author,
            duration_ms: info.duration_ms,
            youtube_query,
        })
    }
}
