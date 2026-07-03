//! Configuration types for LoNode.

use super::extra::{LimitsConfig, SourcesConfig};
use serde::{Deserialize, Serialize};

/// Top-level configuration loaded from `config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// HTTP/WS server settings.
    #[serde(default)]
    pub server: ServerConfig,
    /// Audio pipeline settings.
    #[serde(default)]
    pub audio: AudioConfig,
    /// Source/plugin settings.
    #[serde(default)]
    pub sources: SourcesConfig,
    /// Runtime limits.
    #[serde(default)]
    pub limits: LimitsConfig,
}

/// HTTP/WebSocket server settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Bind host.
    pub host: String,
    /// Bind port (Lavalink default 2333).
    pub port: u16,
    /// Password used for `Authorization` header.
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
    /// Ring-buffer length in milliseconds (~1 s ahead).
    #[serde(default = "default_buffer_ms")]
    pub buffer_ms: u32,
    /// Opus bitrate in bits per second.
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
