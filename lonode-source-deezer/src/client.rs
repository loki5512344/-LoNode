//! Deezer API client (track metadata + preview URL).

use serde::Deserialize;

/// Minimal HTTP client for the Deezer API.
pub struct DeezerClient {
    http: reqwest::Client,
}

/// Subset of Deezer's track object.
#[derive(Debug, Clone, Deserialize)]
pub struct DeezerTrack {
    pub id: u64,
    pub title: String,
    #[serde(rename = "artist_name")]
    pub artist: String,
    pub duration: u64,
    pub preview: String,
}

impl DeezerClient {
    #[must_use]
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// Fetch a track by its Deezer ID.
    ///
    /// # Errors
    /// Returns an error on network or parse failure.
    pub async fn get_track(&self, track_id: &str) -> Result<DeezerTrack, DeezerError> {
        let url = format!("https://api.deezer.com/track/{track_id}");
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| DeezerError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(DeezerError::Api(resp.status().to_string()));
        }
        resp.json()
            .await
            .map_err(|e| DeezerError::Parse(e.to_string()))
    }
}

impl Default for DeezerClient {
    fn default() -> Self {
        Self::new()
    }
}

impl DeezerClient {
    /// Borrow the inner `reqwest::Client` (used by `DeezerSource::stream`).
    pub fn http(&self) -> &reqwest::Client {
        &self.http
    }
}

/// Errors returned by the Deezer client.
#[derive(Debug, thiserror::Error)]
pub enum DeezerError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("deezer api error: {0}")]
    Api(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs() {
        let _c = DeezerClient::new();
    }
}
