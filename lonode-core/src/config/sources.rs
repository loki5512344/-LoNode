//! Source toggles + per-service credentials.

use serde::{Deserialize, Serialize};

/// Toggles for built-in sources and plugin discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesConfig {
    #[serde(default = "default_true")]
    pub youtube: bool,
    #[serde(default = "default_true")]
    pub radio: bool,
    #[serde(default = "default_plugins_dir")]
    pub plugins_dir: String,
}

fn default_true() -> bool {
    true
}
fn default_plugins_dir() -> String {
    "./plugins".to_string()
}

impl Default for SourcesConfig {
    fn default() -> Self {
        Self {
            youtube: default_true(),
            radio: default_true(),
            plugins_dir: default_plugins_dir(),
        }
    }
}

/// Spotify credentials (public + private client). Empty strings = disabled.
/// Register a Spotify app at <https://developer.spotify.com/dashboard>.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpotifyConfig {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

impl SpotifyConfig {
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.client_id.is_empty() && !self.client_secret.is_empty()
    }
}

/// Apple Music credentials. `developer_token` is a JWT signed with your Apple
/// Music API private key. Empty = disabled.
/// See: <https://developer.apple.com/documentation/applemusicapi>
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppleMusicConfig {
    #[serde(default)]
    pub developer_token: String,
    #[serde(default)]
    pub user_token: String,
}

impl AppleMusicConfig {
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.developer_token.is_empty()
    }
}

/// Yandex Music credentials. `access_token` from <https://oauth.yandex.ru>.
/// Empty = disabled. Without a token, only low-quality previews may work.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct YandexMusicConfig {
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub user_id: String,
}

impl YandexMusicConfig {
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.access_token.is_empty()
    }
}

/// Deezer credentials. Preview streaming works without credentials; full
/// tracks require a Premium account token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeezerConfig {
    #[serde(default)]
    pub arl_token: String,
}

impl DeezerConfig {
    #[must_use]
    pub fn premium_enabled(&self) -> bool {
        !self.arl_token.is_empty()
    }
}

/// Google Cloud TTS credentials.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TtsGoogleConfig {
    /// API key from Google Cloud Console.
    #[serde(default)]
    pub api_key: String,
}

impl TtsGoogleConfig {
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.api_key.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sources_default_enables_builtins() {
        let s = SourcesConfig::default();
        assert!(s.youtube);
        assert!(s.radio);
        assert_eq!(s.plugins_dir, "./plugins");
    }

    #[test]
    fn spotify_disabled_when_empty() {
        assert!(!SpotifyConfig::default().enabled());
        let c = SpotifyConfig {
            client_id: "id".into(),
            client_secret: "".into(),
        };
        assert!(!c.enabled());
        let c = SpotifyConfig {
            client_id: "id".into(),
            client_secret: "s".into(),
        };
        assert!(c.enabled());
    }

    #[test]
    fn yandex_disabled_when_empty() {
        assert!(!YandexMusicConfig::default().enabled());
    }
}
