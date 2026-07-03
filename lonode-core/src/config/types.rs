//! Configuration types for LoNode.

use serde::{Deserialize, Serialize};

/// Top-level configuration loaded from `config.toml`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    /// HTTP/WS server settings.
    #[serde(default)]
    pub server: ServerConfig,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_lavalink_port() {
        assert_eq!(Config::default().server.port, 2333);
    }

    #[test]
    fn default_password_is_classic() {
        assert_eq!(Config::default().server.password, "youshallnotpass");
    }
}
