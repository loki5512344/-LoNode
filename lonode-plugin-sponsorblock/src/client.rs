//! SponsorBlock API client — fetches segments for a YouTube video ID.

use crate::types::{Category, Segment};
use reqwest::Client;
use serde::Deserialize;

/// HTTP client for the SponsorBlock API (default server: `sponsor.ajay.app`).
pub struct SponsorBlockClient {
    http: Client,
    base_url: String,
}

impl Default for SponsorBlockClient {
    fn default() -> Self {
        Self::new()
    }
}

impl SponsorBlockClient {
    #[must_use]
    pub fn new() -> Self {
        Self::with_base_url("https://sponsor.ajay.app".to_string())
    }

    #[must_use]
    pub fn with_base_url(base_url: String) -> Self {
        Self {
            http: Client::new(),
            base_url,
        }
    }

    /// Borrow the configured base URL (for tests / introspection).
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// URL-encode a string (exposed for tests).
    #[must_use]
    pub fn urlencoding_encode(s: &str) -> String {
        url_encode(s)
    }

    /// Fetch all sponsor segments for `video_id`.
    ///
    /// # Errors
    /// Returns an error if the API request fails or the response is malformed.
    pub async fn get_segments(&self, video_id: &str) -> Result<Vec<Segment>, SponsorBlockError> {
        self.get_segments_filtered(video_id, Category::all()).await
    }

    /// Fetch segments filtered by `categories`.
    ///
    /// # Errors
    /// Returns an error if the API request fails.
    pub async fn get_segments_filtered(
        &self,
        video_id: &str,
        categories: &[Category],
    ) -> Result<Vec<Segment>, SponsorBlockError> {
        let cats: Vec<&str> = categories.iter().map(|c| c.as_str()).collect();
        let cats_json = serde_json::to_string(&cats)?;
        let url = format!(
            "{}/api/skipSegments?videoID={}&categories={}",
            self.base_url,
            video_id,
            url_encode(&cats_json)
        );
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| SponsorBlockError::Network(e.to_string()))?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(Vec::new());
        }
        if !resp.status().is_success() {
            return Err(SponsorBlockError::Api(resp.status().to_string()));
        }
        let raw: Vec<RawSegment> = resp
            .json()
            .await
            .map_err(|e| SponsorBlockError::Parse(e.to_string()))?;
        Ok(raw.into_iter().filter_map(|r| r.into_segment()).collect())
    }
}

/// Raw SponsorBlock API response item.
#[derive(Debug, Deserialize)]
struct RawSegment {
    #[serde(alias = "segment")]
    segment: Vec<f64>,
    category: String,
}

impl RawSegment {
    fn into_segment(self) -> Option<Segment> {
        if self.segment.len() < 2 {
            return None;
        }
        let category = match self.category.as_str() {
            "sponsor" => Category::Sponsor,
            "intro" => Category::Intro,
            "outro" => Category::Outro,
            "selfpromo" => Category::Selfpromo,
            "preview" => Category::Preview,
            "filler" => Category::Filler,
            "interaction" => Category::Interaction,
            "music_offtopic" => Category::MusicOfftopic,
            _ => return None,
        };
        Some(Segment {
            start: self.segment[0],
            end: self.segment[1],
            category,
        })
    }
}

/// Errors returned by the SponsorBlock client.
#[derive(Debug, thiserror::Error)]
pub enum SponsorBlockError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("sponsorblock api error: {0}")]
    Api(String),
}

impl From<serde_json::Error> for SponsorBlockError {
    fn from(e: serde_json::Error) -> Self {
        Self::Parse(e.to_string())
    }
}
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}
