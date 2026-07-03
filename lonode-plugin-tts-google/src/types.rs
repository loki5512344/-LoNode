//! TTS request types + voice selection.

use serde::{Deserialize, Serialize};

/// Audio encoding format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioEncoding {
    #[serde(rename = "MP3")]
    Mp3,
    #[serde(rename = "LINEAR16")]
    Linear16,
    #[serde(rename = "OGG_OPUS")]
    OggOpus,
}

/// Voice gender.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Gender {
    #[serde(rename = "MALE")]
    Male,
    #[serde(rename = "FEMALE")]
    Female,
    #[serde(rename = "NEUTRAL")]
    Neutral,
}

/// Voice selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Voice {
    /// BCP-47 language code (e.g. `en-US`, `ru-RU`).
    pub language_code: String,
    /// Optional specific voice name (e.g. `en-US-Wavenet-D`).
    pub name: Option<String>,
    /// Optional gender preference.
    pub ssml_gender: Option<Gender>,
}

impl Voice {
    /// Create a voice with just a language code.
    #[must_use]
    pub fn new(language_code: impl Into<String>) -> Self {
        Self {
            language_code: language_code.into(),
            name: None,
            ssml_gender: None,
        }
    }

    /// Specify a voice name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Specify a gender preference.
    #[must_use]
    pub fn with_gender(mut self, gender: Gender) -> Self {
        self.ssml_gender = Some(gender);
        self
    }
}

/// TTS synthesis request.
#[derive(Debug, Clone)]
pub struct TtsRequest {
    /// Text to synthesize (plain text or SSML — see `is_ssml`).
    pub text: String,
    /// `true` if `text` is SSML.
    pub is_ssml: bool,
    /// Voice to use.
    pub voice: Voice,
    /// Output encoding.
    pub encoding: AudioEncoding,
    /// Speaking rate (0.25–4.0, default 1.0).
    pub rate: f64,
    /// Pitch (-20.0 to +20.0, default 0.0).
    pub pitch: f64,
}

impl TtsRequest {
    /// Create a plain-text request with default voice + MP3 encoding.
    #[must_use]
    pub fn text(text: impl Into<String>, language_code: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            is_ssml: false,
            voice: Voice::new(language_code),
            encoding: AudioEncoding::Mp3,
            rate: 1.0,
            pitch: 0.0,
        }
    }

    /// Mark the input text as SSML.
    #[must_use]
    pub fn ssml(mut self) -> Self {
        self.is_ssml = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_builds_with_defaults() {
        let v = Voice::new("en-US");
        assert_eq!(v.language_code, "en-US");
        assert!(v.name.is_none());
        assert!(v.ssml_gender.is_none());
    }

    #[test]
    fn voice_chains_builders() {
        let v = Voice::new("en-US")
            .with_name("en-US-Wavenet-D")
            .with_gender(Gender::Male);
        assert_eq!(v.name.as_deref(), Some("en-US-Wavenet-D"));
        assert_eq!(v.ssml_gender, Some(Gender::Male));
    }

    #[test]
    fn tts_request_defaults_to_mp3() {
        let r = TtsRequest::text("hello", "en-US");
        assert_eq!(r.encoding, AudioEncoding::Mp3);
        assert!(!r.is_ssml);
        assert_eq!(r.rate, 1.0);
    }

    #[test]
    fn tts_request_ssml_flag() {
        let r = TtsRequest::text("<speak>hi</speak>", "en-US").ssml();
        assert!(r.is_ssml);
    }
}
