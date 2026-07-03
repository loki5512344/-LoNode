//! Shared application state + authorization middleware.
//!
//! Lavalink uses `Authorization: <password>`. The middleware reads
//! `AppState` from `axum::Extension` so it composes with `from_fn`.

use crate::config::Config;
use crate::player::PlayerManager;
use crate::plugins::PluginRegistry;
use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum::Extension;
use std::sync::Arc;

/// Clonable handle passed to every axum handler via `State`.
#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    pub config: Config,
    pub players: PlayerManager,
    pub sources: PluginRegistry,
    pub started_at: std::time::Instant,
}

impl AppState {
    #[must_use]
    pub fn new(config: Config, players: PlayerManager, sources: PluginRegistry) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                config,
                players,
                sources,
                started_at: std::time::Instant::now(),
            }),
        }
    }

    #[must_use]
    pub fn config(&self) -> &Config {
        &self.inner.config
    }

    #[must_use]
    pub fn players(&self) -> &PlayerManager {
        &self.inner.players
    }

    #[must_use]
    pub fn sources(&self) -> &PluginRegistry {
        &self.inner.sources
    }

    #[must_use]
    pub fn uptime_seconds(&self) -> u64 {
        self.inner.started_at.elapsed().as_secs()
    }

    /// Returns `true` if the request's `Authorization` header matches the
    /// configured password (Lavalink uses plain `Authorization: <password>`).
    #[must_use]
    pub fn check_auth(&self, header: Option<&str>) -> bool {
        header.is_some_and(|h| h == self.inner.config.server.password)
    }
}

/// Reject requests whose `Authorization` header doesn't match the password.
pub async fn require_auth(
    Extension(state): Extension<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let auth = req
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());
    if state.check_auth(auth) {
        Ok(next.run(req).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_accepts_correct_password() {
        let s = make_state();
        assert!(s.check_auth(Some("youshallnotpass")));
    }

    #[test]
    fn auth_rejects_wrong_password() {
        let s = make_state();
        assert!(!s.check_auth(Some("nope")));
        assert!(!s.check_auth(None));
    }

    fn make_state() -> AppState {
        let cfg = Config::default();
        let players = PlayerManager::new(cfg.limits.clone());
        let sources = PluginRegistry::new();
        AppState::new(cfg, players, sources)
    }
}
