//! Source- and limits-related configuration types.

use serde::{Deserialize, Serialize};

/// Toggles for built-in sources and plugin discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourcesConfig {
    /// Enable the YouTube source (Phase 3).
    #[serde(default = "default_true")]
    pub youtube: bool,
    /// Enable the radio source (Phase 3).
    #[serde(default = "default_true")]
    pub radio: bool,
    /// Directory scanned for `.so` plugins (Phase 4).
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
    fn limits_default_matches_lavalink_sizing() {
        let l = LimitsConfig::default();
        assert_eq!(l.max_players, 100);
        assert_eq!(l.max_queue, 500);
    }
}
