//! LoNode binary entry point.
//!
//! Phase 1 behaviour:
//! - Loads `config.toml` (or falls back to defaults).
//! - Reads voice credentials from environment variables
//!   (`VOICE_ENDPOINT`, `VOICE_GUILD_ID`, `VOICE_USER_ID`,
//!   `VOICE_SESSION_ID`, `VOICE_TOKEN`).
//! - If all five are present, runs a single voice session via
//!   [`lonode_core::runner::run_voice_session`].
//! - Otherwise, prints usage and exits cleanly.
//!
//! Phase 2 will add an axum HTTP/WS server that supplies these credentials
//! dynamically per guild.

use lonode_core::config;
use lonode_core::gateway::VoiceConfig;
use lonode_core::runner;

use anyhow::Result;

const ENV_ENDPOINT: &str = "VOICE_ENDPOINT";
const ENV_GUILD: &str = "VOICE_GUILD_ID";
const ENV_USER: &str = "VOICE_USER_ID";
const ENV_SESSION: &str = "VOICE_SESSION_ID";
const ENV_TOKEN: &str = "VOICE_TOKEN";

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cfg = config::load("config.toml")?;
    tracing::info!(host = %cfg.server.host, port = cfg.server.port, "lonode starting (phase 1)");

    let Some(voice) = read_voice_env() else {
        eprintln!("Voice credentials not set in environment.");
        eprintln!("Required: {ENV_ENDPOINT}, {ENV_GUILD}, {ENV_USER}, {ENV_SESSION}, {ENV_TOKEN}");
        eprintln!("Phase 2 will add an HTTP/WS server that supplies these per guild.");
        return Ok(());
    };

    tokio::select! {
        res = runner::run_voice_session(voice) => {
            if let Err(e) = res {
                tracing::error!(error = %e, "voice session ended with error");
            }
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("ctrl-c received, shutting down");
        }
    }
    Ok(())
}

/// Initialize `tracing_subscriber` with `RUST_LOG` env filter, defaulting to `info`.
fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("lonode=info,warn"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

/// Read the five required voice env vars. Returns `None` if any is missing.
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
