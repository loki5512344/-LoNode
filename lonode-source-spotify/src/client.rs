//! Spotify Web API client (track metadata + search query builder).

use serde::Deserialize;

/// Credentials from the Spotify Developer dashboard.
#[derive(Debug, Clone)]
pub struct SpotifyCredentials {
    pub client_id: String,
    pub client_secret: String,
}

/// Minimal HTTP client for the Spotify Web API.
pub struct SpotifyClient {
    http: reqwest::Client,
    creds: SpotifyCredentials,
    cached_token: tokio::sync::Mutex<Option<CachedToken>>,
}

#[derive(Debug)]
struct CachedToken {
    access: String,
    expires_at: std::time::Instant,
}

/// Subset of Spotify's track object we care about.
#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyTrack {
    pub id: String,
    pub name: String,
    pub duration_ms: u64,
    pub artists: Vec<SpotifyArtist>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpotifyArtist {
    pub name: String,
}

impl SpotifyClient {
    /// Create a new client. Call [`refresh_token`](Self::refresh_token) once
    /// before making API requests.
    #[must_use]
    pub fn new(creds: SpotifyCredentials) -> Self {
        Self {
            http: reqwest::Client::new(),
            creds,
            cached_token: tokio::sync::Mutex::new(None),
        }
    }

    /// Fetch a new client-credentials access token from Spotify.
    ///
    /// # Errors
    /// Returns an error if the request fails or Spotify rejects the credentials.
    pub async fn refresh_token(&self) -> Result<String, SpotifyError> {
        let resp = self
            .http
            .post("https://accounts.spotify.com/api/token")
            .form(&[
                ("grant_type", "client_credentials"),
                ("client_id", &self.creds.client_id),
                ("client_secret", &self.creds.client_secret),
            ])
            .send()
            .await
            .map_err(|e| SpotifyError::Network(e.to_string()))?;
        let json: TokenResponse = resp
            .json()
            .await
            .map_err(|e| SpotifyError::Parse(e.to_string()))?;
        let token = CachedToken {
            access: json.access_token.clone(),
            expires_at: std::time::Instant::now() + std::time::Duration::from_secs(json.expires_in),
        };
        let access = token.access.clone();
        *self.cached_token.lock().await = Some(token);
        Ok(access)
    }

    async fn token(&self) -> Result<String, SpotifyError> {
        let cached = {
            let guard = self.cached_token.lock().await;
            guard
                .as_ref()
                .filter(|t| {
                    t.expires_at > std::time::Instant::now() + std::time::Duration::from_secs(60)
                })
                .map(|t| t.access.clone())
        };
        if let Some(access) = cached {
            return Ok(access);
        }
        self.refresh_token().await
    }

    /// Fetch a track by its Spotify ID (e.g. `0VjIjW4GlUZAMYd2vXMi3b`).
    ///
    /// # Errors
    /// Returns an error on network, parse, or auth failure.
    pub async fn get_track(&self, track_id: &str) -> Result<SpotifyTrack, SpotifyError> {
        let token = self.token().await?;
        let url = format!("https://api.spotify.com/v1/tracks/{track_id}");
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .map_err(|e| SpotifyError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(SpotifyError::Api(resp.status().to_string()));
        }
        resp.json()
            .await
            .map_err(|e| SpotifyError::Parse(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

/// Errors returned by the Spotify client.
#[derive(Debug, thiserror::Error)]
pub enum SpotifyError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("spotify api error: {0}")]
    Api(String),
}
