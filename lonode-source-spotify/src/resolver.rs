//! Spotify resolver — implements `AudioSource` for Spotify URLs.
//!
//! Unlike other sources, Spotify cannot stream audio directly. `stream()`
//! returns an error; the runtime layer is responsible for taking the
//! resolved metadata (artist + title) and handing it to the YouTube source
//! for actual playback.

use crate::client::{SpotifyClient, SpotifyCredentials, SpotifyTrack as RawTrack};
use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::AsyncRead;

/// Resolved Spotify track + a suggested YouTube search query.
#[derive(Debug, Clone)]
pub struct SpotifyTrack {
    pub id: String,
    pub title: String,
    pub artist: String,
    pub duration_ms: u64,
    /// Suggested YouTube search string (`"{artist} - {title}"`).
    pub youtube_query: String,
}

/// Spotify `AudioSource` — resolves metadata, delegates playback to YouTube.
pub struct SpotifyResolver {
    client: Option<SpotifyClient>,
}

impl Default for SpotifyResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl SpotifyResolver {
    #[must_use]
    pub fn new() -> Self {
        Self { client: None }
    }

    /// Configure with Spotify credentials (enables `supports()`).
    #[must_use]
    pub fn with_credentials(creds: SpotifyCredentials) -> Self {
        Self {
            client: Some(SpotifyClient::new(creds)),
        }
    }

    /// `true` if `url` is a Spotify track URL.
    #[must_use]
    pub fn is_spotify_url(url: &str) -> bool {
        url.contains("open.spotify.com/track/") || url.starts_with("spotify:track:")
    }

    /// Extract the 22-char track ID from a Spotify URL or URI.
    #[must_use]
    pub fn extract_track_id(url: &str) -> Option<&str> {
        if let Some(pos) = url.find("spotify:track:") {
            return Some(&url[pos + "spotify:track:".len()..]);
        }
        if let Some(pos) = url.find("open.spotify.com/track/") {
            let rest = &url[pos + "open.spotify.com/track/".len()..];
            // Strip query params if present.
            return Some(rest.split('?').next().unwrap_or(rest));
        }
        None
    }

    fn enabled(&self) -> bool {
        self.client.is_some()
    }
}

#[async_trait]
impl AudioSource for SpotifyResolver {
    fn name(&self) -> &str {
        "spotify"
    }

    fn supports(&self, url: &str) -> bool {
        self.enabled() && Self::is_spotify_url(url)
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        let client = self
            .client
            .as_ref()
            .ok_or_else(|| PluginError::Resolve("spotify credentials not set".into()))?;
        let track_id = Self::extract_track_id(url)
            .ok_or_else(|| PluginError::Resolve(format!("invalid spotify url: {url}")))?;
        let raw: RawTrack = client
            .get_track(track_id)
            .await
            .map_err(|e| PluginError::Resolve(e.to_string()))?;
        let artist = raw
            .artists
            .first()
            .map(|a| a.name.clone())
            .unwrap_or_else(|| "Unknown".into());
        Ok(TrackInfo {
            title: raw.name.clone(),
            author: artist.clone(),
            duration_ms: raw.duration_ms,
            url: url.to_string(),
        })
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        // Spotify cannot stream audio directly. The runtime should take the
        // resolved TrackInfo (title + author), build a YouTube search query,
        // and hand it to the YouTube source for playback.
        Err(PluginError::Stream(
            "spotify does not support direct streaming — use YouTube fallback (see resolve())"
                .into(),
        ))
    }
}

impl SpotifyResolver {
    /// Resolve a Spotify URL to a [`SpotifyTrack`] (includes YouTube query).
    ///
    /// # Errors
    /// Returns an error if credentials aren't set or the API call fails.
    pub async fn resolve_full(&self, url: &str) -> Result<SpotifyTrack, PluginError> {
        let info = self.resolve(url).await?;
        let youtube_query = format!("{} - {}", info.author, info.title);
        Ok(SpotifyTrack {
            id: Self::extract_track_id(url).unwrap_or("").to_string(),
            title: info.title,
            artist: info.author,
            duration_ms: info.duration_ms,
            youtube_query,
        })
    }
}
