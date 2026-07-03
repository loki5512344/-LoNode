//! LoNode binary entry point.
//!
//! - Loads `config.toml` (or falls back to defaults).
//! - Builds a `PluginRegistry` from config (see [`sources::build_sources`]).
//! - Starts the axum HTTP/WS API server (Lavalink v4 compatible).
//! - Optionally also starts one voice session from env vars (dev/test mode).

mod sources;

use anyhow::Result;
use lonode_core::config;
use lonode_gateway::VoiceConfig;
use lonode_runtime::runner;

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

    let players = lonode_runtime::player::PlayerManager::new(cfg.limits.clone());
    let registry = sources::build_sources(&cfg).await;

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
        res = lonode_api::serve(&cfg, players, registry) => {
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
