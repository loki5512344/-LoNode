//! HTTP/WS API: axum router, auth middleware, REST + WS handlers.

pub mod handlers;
pub mod state;

pub use state::{require_auth, AppState};

use crate::config::Config;
use crate::player::PlayerManager;
use crate::plugins::PluginRegistry;
use crate::Result;
use axum::middleware::from_fn;
use axum::routing::get;
use axum::Extension;
use axum::Router;
use std::net::SocketAddr;

/// Build the axum router with all v4 routes and auth middleware.
pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/v4/info", get(handlers::info))
        .route("/v4/stats", get(handlers::stats))
        .route("/v4/websocket", get(handlers::ws_handler))
        .route(
            "/v4/sessions/:session_id/players/:guild_id",
            get(handlers::get_player).patch(handlers::patch_player),
        )
        .layer(from_fn(require_auth))
        .with_state(state.clone())
        .layer(Extension(state))
}

/// Bind the HTTP server and serve until shutdown.
///
/// # Errors
/// Returns an error if the socket cannot be bound.
pub async fn serve(config: &Config, players: PlayerManager, sources: PluginRegistry) -> Result<()> {
    let state = AppState::new(config.clone(), players, sources);
    let app = build_router(state);
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port).parse()?;
    tracing::info!(%addr, "http api listening");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_builds_without_panicking() {
        let cfg = Config::default();
        let players = PlayerManager::new(cfg.limits.clone());
        let sources = PluginRegistry::new();
        let state = AppState::new(cfg, players, sources);
        let _router = build_router(state);
    }
}
