//! Apple Music API client (track metadata).

use serde::Deserialize;

/// Apple Music API credentials.
///
/// `developer_token` is a JWT signed with your Apple Music API private key.
/// See: <https://developer.apple.com/documentation/applemusicapi/getting_keys_and_creating_tokens>
#[derive(Debug, Clone)]
pub struct AppleMusicCredentials {
    /// JWT developer token (from Apple Developer portal).
    pub developer_token: String,
    /// Optional user music token (for authenticated requests).
    pub user_token: Option<String>,
}

/// Minimal HTTP client for the Apple Music API.
pub struct AppleMusicClient {
    http: reqwest::Client,
    creds: AppleMusicCredentials,
}

/// Subset of Apple Music's track object.
#[derive(Debug, Clone, Deserialize)]
pub struct AppleMusicTrack {
    pub id: String,
    #[serde(rename = "attributes")]
    pub attributes: TrackAttributes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackAttributes {
    pub name: String,
    #[serde(rename = "artistName")]
    pub artist_name: String,
    #[serde(rename = "durationInMillis")]
    pub duration_ms: u64,
}

impl AppleMusicClient {
    #[must_use]
    pub fn new(creds: AppleMusicCredentials) -> Self {
        Self {
            http: reqwest::Client::new(),
            creds,
        }
    }

    /// Fetch a track by its Apple Music ID.
    ///
    /// # Errors
    /// Returns an error on network, auth, or parse failure.
    pub async fn get_track(&self, track_id: &str) -> Result<AppleMusicTrack, AppleMusicError> {
        let url = format!("https://api.music.apple.com/v1/catalog/us/songs/{track_id}");
        let mut req = self.http.get(&url).bearer_auth(&self.creds.developer_token);
        if let Some(ref user_token) = self.creds.user_token {
            req = req.header("Music-User-Token", user_token);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| AppleMusicError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            return Err(AppleMusicError::Api(resp.status().to_string()));
        }
        let json: ApiResponse = resp
            .json()
            .await
            .map_err(|e| AppleMusicError::Parse(e.to_string()))?;
        json.data
            .into_iter()
            .next()
            .ok_or_else(|| AppleMusicError::Api("no track in response".into()))
    }
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    data: Vec<AppleMusicTrack>,
}

/// Errors returned by the Apple Music client.
#[derive(Debug, thiserror::Error)]
pub enum AppleMusicError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("apple music api error: {0}")]
    Api(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_constructs_with_creds() {
        let creds = AppleMusicCredentials {
            developer_token: "jwt-token".into(),
            user_token: None,
        };
        let _c = AppleMusicClient::new(creds);
    }
}
