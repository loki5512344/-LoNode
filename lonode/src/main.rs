//! LoNode binary entry point.
//!
//! - Loads `config.toml` (or falls back to defaults).
//! - Builds a `PluginRegistry`: registers built-in sources (radio, youtube,
//!   soundcloud, bandcamp, twitch, vimeo) + Spotify resolver, then loads
//!   any `.so` files from `config.sources.plugins_dir`.
//! - Starts the axum HTTP/WS API server (Lavalink v4 compatible).
//! - Optionally also starts one voice session from env vars (dev/test mode).

use anyhow::Result;
use lonode_core::config;
use lonode_gateway::VoiceConfig;
use lonode_runtime::plugins::PluginRegistry;
use lonode_runtime::runner;
use lonode_source_spotify::{SpotifyCredentials, SpotifyResolver};
use std::sync::Arc;

const ENV_ENDPOINT: &str = "VOICE_ENDPOINT";
const ENV_GUILD: &str = "VOICE_GUILD_ID";
const ENV_USER: &str = "VOICE_USER_ID";
const ENV_SESSION: &str = "VOICE_SESSION_ID";
const ENV_TOKEN: &str = "VOICE_TOKEN";
const ENV_SPOTIFY_ID: &str = "SPOTIFY_CLIENT_ID";
const ENV_SPOTIFY_SECRET: &str = "SPOTIFY_CLIENT_SECRET";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cfg = config::load("config.toml")?;
    tracing::info!(host = %cfg.server.host, port = cfg.server.port, "lonode starting");

    let players = lonode_runtime::player::PlayerManager::new(cfg.limits.clone());
    let sources = build_sources(&cfg).await;

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
        res = lonode_api::serve(&cfg, players, sources) => {
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

/// Build the source registry: built-ins first, then Spotify, then dynamic plugins.
async fn build_sources(cfg: &config::Config) -> PluginRegistry {
    let reg = PluginRegistry::new();

    // Specific sources first (highest priority).
    if cfg.sources.youtube {
        reg.register_builtin(Arc::new(lonode_sources_builtin::YoutubeSource::new()))
            .await;
        tracing::info!("registered source: youtube (stub)");
    }
    // SoundCloud, Bandcamp, Twitch, Vimeo are stubs but registered so their
    // names appear in /v4/info and the ABI is exercised. They return
    // supports()=false until implemented, so they never shadow other sources.
    reg.register_builtin(Arc::new(lonode_sources_builtin::SoundCloudSource::new()))
        .await;
    reg.register_builtin(Arc::new(lonode_sources_builtin::BandcampSource::new()))
        .await;
    reg.register_builtin(Arc::new(lonode_sources_builtin::TwitchSource::new()))
        .await;
    reg.register_builtin(Arc::new(lonode_sources_builtin::VimeoSource::new()))
        .await;
    tracing::info!("registered sources: soundcloud, bandcamp, twitch, vimeo (stubs)");

    // Spotify resolver (env-var driven — needs client_id + secret).
    if let (Ok(id), Ok(secret)) = (
        std::env::var(ENV_SPOTIFY_ID),
        std::env::var(ENV_SPOTIFY_SECRET),
    ) {
        let creds = SpotifyCredentials {
            client_id: id,
            client_secret: secret,
        };
        reg.register_builtin(Arc::new(SpotifyResolver::with_credentials(creds)))
            .await;
        tracing::info!("registered source: spotify");
    } else {
        tracing::info!(
            "spotify disabled (set SPOTIFY_CLIENT_ID + SPOTIFY_CLIENT_SECRET to enable)"
        );
    }

    // Radio LAST — fallback for any HTTP URL not claimed by a specific source.
    if cfg.sources.radio {
        reg.register_builtin(Arc::new(lonode_sources_builtin::RadioSource::new()))
            .await;
        tracing::info!("registered source: radio (fallback)");
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
