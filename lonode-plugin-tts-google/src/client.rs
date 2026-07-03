//! Google Cloud TTS API client.

use crate::types::{AudioEncoding, TtsRequest};
use base64::Engine;
use serde::{Deserialize, Serialize};

/// Google Cloud TTS client. Uses an API key for authentication.
pub struct TtsClient {
    http: reqwest::Client,
    api_key: String,
}

impl TtsClient {
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key: api_key.into(),
        }
    }

    /// Synthesize `request` → MP3/Opus/PCM bytes.
    ///
    /// # Errors
    /// Returns an error on network, auth, or synthesis failure.
    pub async fn synthesize(&self, request: &TtsRequest) -> Result<Vec<u8>, TtsError> {
        let input = if request.is_ssml {
            SynthesisInput {
                ssml: Some(request.text.clone()),
                text: None,
            }
        } else {
            SynthesisInput {
                text: Some(request.text.clone()),
                ssml: None,
            }
        };
        let body = SynthesizeRequest {
            input,
            voice: VoiceParams {
                language_code: request.voice.language_code.clone(),
                name: request.voice.name.clone(),
                ssml_gender: request.voice.ssml_gender,
            },
            audio_config: AudioConfig {
                audio_encoding: request.encoding,
                speaking_rate: Some(request.rate),
                pitch: Some(request.pitch),
            },
        };
        let url = format!(
            "https://texttospeech.googleapis.com/v1/text:synthesize?key={}",
            self.api_key
        );
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| TtsError::Network(e.to_string()))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(TtsError::Api(format!("{status}: {text}")));
        }
        let json: SynthesizeResponse = resp
            .json()
            .await
            .map_err(|e| TtsError::Parse(e.to_string()))?;
        base64::engine::general_purpose::STANDARD
            .decode(&json.audio_content)
            .map_err(|e| TtsError::Decode(e.to_string()))
    }
}

#[derive(Serialize)]
struct SynthesizeRequest {
    input: SynthesisInput,
    voice: VoiceParams,
    audio_config: AudioConfig,
}

#[derive(Serialize)]
struct SynthesisInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct VoiceParams {
    language_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ssml_gender: Option<crate::types::Gender>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AudioConfig {
    audio_encoding: AudioEncoding,
    #[serde(skip_serializing_if = "Option::is_none")]
    speaking_rate: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pitch: Option<f64>,
}

#[derive(Deserialize)]
struct SynthesizeResponse {
    audio_content: String,
}

/// Errors returned by the TTS client.
#[derive(Debug, thiserror::Error)]
pub enum TtsError {
    #[error("network error: {0}")]
    Network(String),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("base64 decode error: {0}")]
    Decode(String),
    #[error("google tts api error: {0}")]
    Api(String),
}
