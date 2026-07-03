//! WebSocket handler — Lavalink v4 protocol.
//!
//! On connect the client sends a `ready` op; we respond with `ready` op
//! containing the session id. The client then sends `play`, `pause`,
//! `stop`, etc. and the server emits `TrackStartEvent`, `TrackEndEvent`.

use crate::api::state::AppState;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};

/// `op` field on incoming WS messages.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ClientOp {
    Play,
    Stop,
    Pause,
    Resume,
}

#[derive(Debug, Deserialize)]
struct ClientMessage {
    op: ClientOp,
    #[serde(default)]
    guild_id: String,
}

/// `op` field on outgoing WS messages.
#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
enum ServerOp {
    Ready,
    Event,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ServerMessage<T: Serialize> {
    op: ServerOp,
    #[serde(skip_serializing_if = "Option::is_none")]
    guild_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    r#type: Option<String>,
    data: T,
}

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| run_ws(socket, state))
}

async fn run_ws(mut socket: WebSocket, state: AppState) {
    let session_id = format!("lonode-{}", uuid_like());
    let ready = ServerMessage {
        op: ServerOp::Ready,
        guild_id: None,
        r#type: None,
        data: serde_json::json!({ "sessionId": session_id }),
    };
    if send_json(&mut socket, &ready).await.is_err() {
        return;
    }

    while let Some(Ok(msg)) = futures_util::StreamExt::next(&mut socket).await {
        if let Message::Text(text) = msg {
            if handle_text(&state, &text, &mut socket).await.is_err() {
                break;
            }
        }
    }
}

async fn handle_text(state: &AppState, text: &str, socket: &mut WebSocket) -> Result<(), ()> {
    let Ok(msg): std::result::Result<ClientMessage, _> = serde_json::from_str(text) else {
        return Ok(());
    };
    match msg.op {
        ClientOp::Play => {
            let _ = state.players().get_or_create(&msg.guild_id).await;
            let _ = state
                .players()
                .with_player(&msg.guild_id, |p| {
                    p.play(crate::player::Track::new("n/a", "unknown"));
                })
                .await;
            emit_event(socket, &msg.guild_id, "TrackStartEvent").await;
        }
        ClientOp::Stop => {
            let _ = state
                .players()
                .with_player(&msg.guild_id, |p| p.stop())
                .await;
            emit_event(socket, &msg.guild_id, "TrackEndEvent").await;
        }
        ClientOp::Pause => {
            let _ = state
                .players()
                .with_player(&msg.guild_id, |p| p.pause())
                .await;
        }
        ClientOp::Resume => {
            let _ = state
                .players()
                .with_player(&msg.guild_id, |p| p.resume())
                .await;
        }
    }
    Ok(())
}

async fn emit_event(socket: &mut WebSocket, guild_id: &str, event_type: &str) {
    let msg = ServerMessage {
        op: ServerOp::Event,
        guild_id: Some(guild_id.to_string()),
        r#type: Some(event_type.to_string()),
        data: serde_json::json!({}),
    };
    let _ = send_json(socket, &msg).await;
}

async fn send_json<T: Serialize>(socket: &mut WebSocket, value: &T) -> Result<(), ()> {
    let json = serde_json::to_string(value).map_err(|_| ())?;
    socket.send(Message::Text(json)).await.map_err(|_| ())
}

/// Cheap pseudo-session-id without pulling in `uuid`.
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{nanos:x}")
}
