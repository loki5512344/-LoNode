//! `lonode-plugin-tts-google` — Google Cloud Text-to-Speech plugin for LoNode.
//!
//! Synthesizes speech from text via the [Google Cloud TTS API](https://cloud.google.com/text-to-speech).
//! The resulting MP3 bytes can be fed into the audio pipeline as an `AsyncRead`.
//!
//! Public API:
//! - [`TtsClient`] — Google Cloud TTS API client.
//! - [`TtsRequest`] — input text + voice config.
//! - [`Voice`] — voice selection (language, gender, name).

pub mod client;
pub mod types;

pub use client::{TtsClient, TtsError};
pub use types::{AudioEncoding, Gender, TtsRequest, Voice};
