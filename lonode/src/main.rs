//! LoNode binary entry point.
//!
//! - Loads `config.toml` (or falls back to defaults).
//! - Builds a `PluginRegistry`: registers built-in sources (radio, youtube)
//!   and loads any `.so` files from `config.sources.plugins_dir`.
//! - Starts the axum HTTP/WS API server (Lavalink v4 compatible).
//! - Optionally also starts one voice session from env vars (dev/test mode).

use anyhow::Result;
use lonode_core::api;
use lonode_core::config;
use lonode_core::gateway::VoiceConfig;
use lonode_core::player::PlayerManager;
use lonode_core::plugins::PluginRegistry;
use lonode_core::runner;
use std::sync::Arc;

const ENV_ENDPOINT: &str = "VOICE_ENDPOINT";
const ENV_GUILD: &str = "VOICE_GUILD_ID";
const ENV_USER: &str = "VOICE_USER_ID";
const ENV_SESSION: &str = "VOICE_SESSION_ID";
const ENV_TOKEN: &str = "VOICE_TOKEN";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cfg = config::load("config.toml")?;
    tracing::info!(host = %cfg.server.host, port = cfg.server.port, "lonode starting");

    let players = PlayerManager::new(cfg.limits.clone());
    let sources = build_sources(&cfg).await;

    // Optional dev-mode voice session from env vars.
    let voice_task = if let Some(voice) = read_voice_env() {
        let players = players.clone();
        Some(tokio::spawn(async move {
            let _ = players.get_or_create(&voice.guild_id).await;
            if let Err(e) = runner::run_voice_session(voice).await {
                tracing::error!(error = %e, "dev voice session ended");
            }
        }))
    } else {
        tracing::info!("no voice env vars set; running as API-only node");
        None
    };

    tokio::select! {
        res = api::serve(&cfg, players, sources) => {
            if let Err(e) = res {
                tracing::error!(error = %e, "api server ended");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received, shutting down");
        }
    }

    if let Some(t) = voice_task {
        t.abort();
    }
    Ok(())
}

/// Build the source registry: built-ins first, then dynamic plugins.
async fn build_sources(cfg: &config::Config) -> PluginRegistry {
    let reg = PluginRegistry::new();
    if cfg.sources.radio {
        reg.register_builtin(Arc::new(lonode_sources::RadioSource::new()))
            .await;
        tracing::info!("registered built-in source: radio");
    }
    if cfg.sources.youtube {
        reg.register_builtin(Arc::new(lonode_sources::YoutubeSource::new()))
            .await;
        tracing::info!("registered built-in source: youtube (stub)");
    }
    match reg.load_dir(&cfg.sources.plugins_dir).await {
        Ok(n) => tracing::info!(n, "loaded dynamic plugins"),
        Err(e) => tracing::warn!(error = %e, "failed to scan plugins dir"),
    }
    reg
}

fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("lonode=info,warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn read_voice_env() -> Option<VoiceConfig> {
    Some(VoiceConfig {
        endpoint: std::env::var(ENV_ENDPOINT).ok()?,
        guild_id: std::env::var(ENV_GUILD).ok()?,
        user_id: std::env::var(ENV_USER).ok()?,
        session_id: std::env::var(ENV_SESSION).ok()?,
        token: std::env::var(ENV_TOKEN).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_names_are_nonempty() {
        assert!(!ENV_ENDPOINT.is_empty());
        assert!(!ENV_GUILD.is_empty());
        assert!(!ENV_USER.is_empty());
        assert!(!ENV_SESSION.is_empty());
        assert!(!ENV_TOKEN.is_empty());
    }
}
