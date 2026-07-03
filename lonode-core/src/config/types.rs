//! Configuration types for LoNode.

use super::sources::{
    AppleMusicConfig, DeezerConfig, SourcesConfig, SpotifyConfig, TtsGoogleConfig,
    YandexMusicConfig,
};
use serde::{Deserialize, Serialize};

/// Top-level configuration loaded from `config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub sources: SourcesConfig,
    #[serde(default)]
    pub limits: LimitsConfig,
    /// Service credentials. Each section is optional; empty fields disable
    /// the corresponding source.
    #[serde(default)]
    pub spotify: SpotifyConfig,
    #[serde(default)]
    pub apple_music: AppleMusicConfig,
    #[serde(default)]
    pub yandex_music: YandexMusicConfig,
    #[serde(default)]
    pub deezer: DeezerConfig,
    #[serde(default)]
    pub tts: TtsSection,
}

/// HTTP/WebSocket server settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 2333,
            password: "youshallnotpass".to_string(),
        }
    }
}

/// Audio pipeline tuning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(default = "default_buffer_ms")]
    pub buffer_ms: u32,
    #[serde(default = "default_opus_bitrate")]
    pub opus_bitrate: u32,
}

fn default_buffer_ms() -> u32 {
    1_000
}
fn default_opus_bitrate() -> u32 {
    128_000
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            buffer_ms: default_buffer_ms(),
            opus_bitrate: default_opus_bitrate(),
        }
    }
}

/// TTS section (currently only Google Cloud TTS).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtsSection {
    #[serde(default)]
    pub google: TtsGoogleConfig,
}

/// Runtime caps to protect the node from runaway queues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitsConfig {
    /// Maximum concurrent guild players.
    #[serde(default = "default_max_players")]
    pub max_players: u32,
    /// Maximum tracks queued per guild.
    #[serde(default = "default_max_queue")]
    pub max_queue: u32,
}

fn default_max_players() -> u32 {
    100
}
fn default_max_queue() -> u32 {
    500
}

impl Default for LimitsConfig {
    fn default() -> Self {
        Self {
            max_players: default_max_players(),
            max_queue: default_max_queue(),
        }
    }
}
