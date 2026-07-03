//! Player manager: registry of per-guild players.

use super::state::GuildPlayer;
use crate::config::LimitsConfig;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Registry of all active guild players. Shared between REST and WS handlers.
#[derive(Debug, Clone)]
pub struct PlayerManager {
    inner: Arc<RwLock<HashMap<String, GuildPlayer>>>,
    limits: LimitsConfig,
}

impl PlayerManager {
    #[must_use]
    pub fn new(limits: LimitsConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
            limits,
        }
    }

    /// Returns `true` if a player exists for `guild_id`.
    pub async fn exists(&self, guild_id: &str) -> bool {
        self.inner.read().await.contains_key(guild_id)
    }

    /// Get-or-create a player for `guild_id`. Returns an error if adding a
    /// new guild would exceed `limits.max_players`.
    pub async fn get_or_create(&self, guild_id: &str) -> anyhow::Result<()> {
        let mut map = self.inner.write().await;
        if map.contains_key(guild_id) {
            return Ok(());
        }
        if map.len() as u32 >= self.limits.max_players {
            anyhow::bail!("max_players limit reached ({})", self.limits.max_players);
        }
        map.insert(guild_id.to_string(), GuildPlayer::new());
        Ok(())
    }

    /// Drop a player (called on voice disconnect).
    pub async fn remove(&self, guild_id: &str) {
        self.inner.write().await.remove(guild_id);
    }

    /// Apply a closure to the player for `guild_id`. Returns `None` if no
    /// player exists. The closure runs under a write lock.
    pub async fn with_player<F, R>(&self, guild_id: &str, f: F) -> Option<R>
    where
        F: FnOnce(&mut GuildPlayer) -> R,
    {
        let mut map = self.inner.write().await;
        map.get_mut(guild_id).map(f)
    }

    /// Number of active players.
    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }

    /// `true` if no players are registered.
    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limits() -> LimitsConfig {
        LimitsConfig {
            max_players: 2,
            max_queue: 100,
        }
    }

    #[tokio::test]
    async fn creates_and_counts_players() {
        let mgr = PlayerManager::new(limits());
        mgr.get_or_create("g1").await.unwrap();
        mgr.get_or_create("g2").await.unwrap();
        assert_eq!(mgr.len().await, 2);
        assert!(mgr.exists("g1").await);
    }

    #[tokio::test]
    async fn rejects_when_max_players_reached() {
        let mgr = PlayerManager::new(limits());
        mgr.get_or_create("g1").await.unwrap();
        mgr.get_or_create("g2").await.unwrap();
        assert!(mgr.get_or_create("g3").await.is_err());
    }

    #[tokio::test]
    async fn with_player_mutates_state() {
        let mgr = PlayerManager::new(limits());
        mgr.get_or_create("g1").await.unwrap();
        let vol = mgr
            .with_player("g1", |p| {
                p.set_volume(250);
                p.volume()
            })
            .await;
        assert_eq!(vol, Some(250));
    }
}
