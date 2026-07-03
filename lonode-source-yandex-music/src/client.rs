//! Yandex Music API client (track metadata + stream URL).

use serde::Deserialize;

/// Errors returned by the Yandex Music client.
#[derive(Debug, thiserror::Error)]
pub enum YandexMusicError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("yandex music api error: {0}")]
    Api(String),
}

/// Subset of Yandex Music's track object.
#[derive(Debug, Clone, Deserialize)]
pub struct YandexMusicTrack {
    pub id: u64,
    pub title: String,
    #[serde(rename = "artists")]
    pub artists: Vec<YandexArtist>,
    #[serde(rename = "durationMs")]
    pub duration_ms: u64,
    #[serde(default)]
    pub og_image: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct YandexArtist {
    pub name: String,
}

/// Minimal HTTP client for the Yandex Music API.
pub struct YandexMusicClient {
    http: reqwest::Client,
    access_token: Option<String>,
    user_id: Option<String>,
}

impl Default for YandexMusicClient {
    fn default() -> Self {
        Self::new()
    }
}

impl YandexMusicClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
            access_token: None,
            user_id: None,
        }
    }

    #[must_use]
    pub fn with_token(access_token: String, user_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            access_token: Some(access_token),
            user_id: Some(user_id),
        }
    }

    /// Borrow the inner HTTP client (for streaming preview URLs).
    pub(crate) fn http(&self) -> &reqwest::Client {
        &self.http
    }

    /// `true` if a user access token is configured (enables full streaming).
    #[must_use]
    pub fn has_token(&self) -> bool {
        self.access_token.is_some()
    }

    /// Fetch a track by its ID.
    ///
    /// # Errors
    /// Returns an error on network or parse failure.
    pub async fn get_track(&self, track_id: &str) -> Result<YandexMusicTrack, YandexMusicError> {
        let url = format!("https://api.music.yandex.net/tracks/{track_id}");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| YandexMusicError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(YandexMusicError::Api(resp.status().to_string()));
        }
        let json: ApiResponse = resp
            .json()
            .await
            .map_err(|e| YandexMusicError::Parse(e.to_string()))?;
        json.result
            .into_iter()
            .next()
            .ok_or_else(|| YandexMusicError::Api("no track in response".into()))
    }

    /// Get the download URL for a full track (requires user token).
    ///
    /// # Errors
    /// Returns an error if no token is set or the API call fails.
    pub async fn get_download_url(&self, track_id: &str) -> Result<String, YandexMusicError> {
        let token = self.access_token.as_ref().ok_or_else(|| {
            YandexMusicError::Api("access token required for full streaming".into())
        })?;
        let user_id = self.user_id.as_deref().unwrap_or("0");
        let url = format!(
            "https://api.music.yandex.net/tracks/{track_id}/download-link?user-id={user_id}"
        );
        let resp = self
            .http
            .get(&url)
            .header("Authorization", format!("OAuth {token}"))
            .send()
            .await
            .map_err(|e| YandexMusicError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(YandexMusicError::Api(resp.status().to_string()));
        }
        let json: DownloadResponse = resp
            .json()
            .await
            .map_err(|e| YandexMusicError::Parse(e.to_string()))?;
        Ok(json.result)
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    result: Vec<YandexMusicTrack>,
}

#[derive(Debug, Deserialize)]
struct DownloadResponse {
    result: String,
}
