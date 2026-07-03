//! `lonode-gateway` — Discord Voice Gateway WebSocket client.
//!
//! Public API: [`VoiceConfig`], [`GatewayEvent`], [`GatewayCommand`],
//! [`GatewayHandle`], [`connect`].

pub mod actor;
pub mod op;

pub use actor::run as run_actor;
pub use op::{
    Heartbeat, Hello, Identify, Op, Ready, SelectProtocol, SelectProtocolData, SessionDescription,
};

use anyhow::Result;
use tokio::sync::mpsc;

/// Credentials for one voice WebSocket session.
///
/// The bot obtains these from the main Discord gateway
/// (`VOICE_STATE_UPDATE` + `VOICE_SERVER_UPDATE`) and forwards them to LoNode
/// via the REST API.
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// `wss://` endpoint provided by Discord.
    pub endpoint: String,
    /// Guild ID (a.k.a. `server_id` on the wire).
    pub guild_id: String,
    /// Bot user ID.
    pub user_id: String,
    /// Voice session ID from `VOICE_STATE_UPDATE`.
    pub session_id: String,
    /// Voice token from `VOICE_SERVER_UPDATE`.
    pub token: String,
}

/// Events the actor emits to the orchestrator.
#[derive(Debug, Clone)]
pub enum GatewayEvent {
    /// `op 2` Ready — voice server IP/port + ssrc + available modes.
    Ready {
        ssrc: u32,
        ip: String,
        port: u16,
        modes: Vec<String>,
    },
    /// `op 4` Session Description — contains the `secret_key` for UDP encryption.
    SessionDescription { secret_key: [u8; 32] },
    /// Actor exited (cleanly or with error message).
    Closed(String),
}

/// Commands the orchestrator can send to the actor.
#[derive(Debug, Clone)]
pub enum GatewayCommand {
    /// `op 1` Select Protocol — sent after UDP IP discovery completes.
    SelectProtocol {
        address: String,
        port: u16,
        mode: String,
    },
    /// Tear down the WebSocket cleanly.
    Close,
}

/// Handle returned by [`connect`]. Drop both halves to stop the actor.
pub struct GatewayHandle {
    /// Events flowing from the actor to the orchestrator.
    pub events: mpsc::Receiver<GatewayEvent>,
    /// Commands flowing from the orchestrator to the actor.
    pub commands: mpsc::Sender<GatewayCommand>,
}

/// Spawn the actor task. Returns immediately with a [`GatewayHandle`].
///
/// # Errors
/// Returns an error only if the channel cannot be allocated (effectively never).
pub fn connect(cfg: VoiceConfig) -> Result<GatewayHandle> {
    let (events_tx, events_rx) = mpsc::channel(8);
    let (commands_tx, commands_rx) = mpsc::channel(8);
    tokio::spawn(async move {
        if let Err(e) = actor::run(cfg, events_tx.clone(), commands_rx).await {
            tracing::error!(error = %e, "voice gateway actor failed");
            let _ = events_tx.send(GatewayEvent::Closed(e.to_string())).await;
        }
    });
    Ok(GatewayHandle {
        events: events_rx,
        commands: commands_tx,
    })
}
