//! Configuration loading.
//!
//! Public API: [`load`] — read & parse `config.toml`, falling back to
//! [`Config::default`](crate::config::types::Config::default) when the file
//! is missing.

pub mod sources;
pub mod types;

pub use sources::{
    AppleMusicConfig, DeezerConfig, SourcesConfig, SpotifyConfig, TtsGoogleConfig,
    YandexMusicConfig,
};
pub use types::{AudioConfig, Config, LimitsConfig, ServerConfig, TtsSection};

use crate::Result;

/// Read `config.toml` from `path`. Returns `Config::default()` if the file
/// does not exist (typical for first run).
///
/// # Errors
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load(path: &str) -> Result<Config> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(path, "config file not found, using defaults");
            return Ok(Config::default());
        }
        Err(e) => return Err(e.into()),
    };
    parse(&text)
}

/// Parse a TOML string into [`Config`].
///
/// # Errors
/// Returns an error if the input is not valid TOML or doesn't match the schema.
pub fn parse(text: &str) -> Result<Config> {
    let cfg: Config = toml::from_str(text)?;
    validate(&cfg)?;
    Ok(cfg)
}

/// Sanity-check a parsed config (port range, non-empty password, sane limits).
fn validate(cfg: &Config) -> Result<()> {
    if cfg.server.port == 0 {
        anyhow::bail!("server.port must be > 0");
    }
    if cfg.server.password.is_empty() {
        anyhow::bail!("server.password must not be empty");
    }
    if cfg.limits.max_players == 0 {
        anyhow::bail!("limits.max_players must be > 0");
    }
    if cfg.limits.max_queue == 0 {
        anyhow::bail!("limits.max_queue must be > 0");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_toml() {
        let toml = r#"
[server]
host = "127.0.0.1"
port = 8080
password = "secret"
"#;
        let cfg = parse(toml).unwrap();
        assert_eq!(cfg.server.host, "127.0.0.1");
        assert_eq!(cfg.server.port, 8080);
        assert_eq!(cfg.server.password, "secret");
    }

    #[test]
    fn parses_full_config_with_credentials() {
        let toml = r#"
[server]
host = "0.0.0.0"
port = 2333
password = "pass"

[spotify]
client_id = "abc"
client_secret = "def"

[yandex_music]
access_token = "ya.123"
user_id = "42"

[apple_music]
developer_token = "jwt"
"#;
        let cfg = parse(toml).unwrap();
        assert!(cfg.spotify.enabled());
        assert!(cfg.yandex_music.enabled());
        assert_eq!(cfg.yandex_music.user_id, "42");
        assert!(cfg.apple_music.enabled());
    }

    #[test]
    fn rejects_zero_port() {
        let toml = r#"
[server]
host = "0.0.0.0"
port = 0
password = "x"
"#;
        assert!(parse(toml).is_err());
    }

    #[test]
    fn rejects_zero_max_players() {
        let toml = r#"
[server]
host = "0.0.0.0"
port = 1
password = "x"
[limits]
max_players = 0
max_queue = 100
"#;
        assert!(parse(toml).is_err());
    }
}
