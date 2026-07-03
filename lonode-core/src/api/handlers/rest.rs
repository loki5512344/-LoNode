//! REST endpoints (Lavalink v4 compatible).
//!
//! Endpoints:
//! - `GET  /v4/info` — node version & capabilities.
//! - `GET  /v4/stats` — CPU, memory, uptime, player count.
//! - `GET  /v4/sessions/{sessionId}/players/{guildId}` — read player state.
//! - `PATCH /v4/sessions/{sessionId}/players/{guildId}` — update player.

use crate::api::state::AppState;
use crate::player::Track;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

/// `GET /v4/info` response.
#[derive(Debug, Serialize)]
pub struct InfoResponse {
    pub version: &'static str,
    pub build_time: &'static str,
    pub git: &'static str,
    pub node_region: &'static str,
    pub enabled_sources: &'static [&'static str],
}

/// `GET /v4/stats` response.
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub players: usize,
    pub playing_players: usize,
    pub uptime: u64,
    pub memory: MemoryStats,
}

#[derive(Debug, Serialize)]
pub struct MemoryStats {
    pub reserved: u64,
    pub used: u64,
}

/// `PATCH /v4/sessions/{sessionId}/players/{guildId}` body (subset).
#[derive(Debug, Deserialize)]
pub struct UpdatePlayerRequest {
    pub encoded_track: Option<String>,
    #[serde(default)]
    pub volume: Option<u16>,
    #[serde(default)]
    pub paused: Option<bool>,
}

/// `GET` player response (Lavalink v4 `Player` shape, simplified).
#[derive(Debug, Serialize)]
pub struct PlayerResponse {
    pub guild_id: String,
    pub track: Option<Track>,
    pub volume: u16,
    pub paused: bool,
}

pub async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        version: env!("CARGO_PKG_VERSION"),
        build_time: "",
        git: "",
        node_region: "local",
        enabled_sources: &["radio", "youtube"],
    })
}

pub async fn stats(State(state): State<AppState>) -> Json<StatsResponse> {
    let players = state.players().len().await;
    Json(StatsResponse {
        players,
        playing_players: 0,
        uptime: state.uptime_seconds(),
        memory: MemoryStats {
            reserved: 0,
            used: 0,
        },
    })
}

pub async fn get_player(
    State(state): State<AppState>,
    Path((_session, guild_id)): Path<(String, String)>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    let player = state
        .players()
        .with_player(&guild_id, |p| PlayerResponse {
            guild_id: guild_id.clone(),
            track: p.current_track().cloned(),
            volume: p.volume(),
            paused: matches!(p.state(), crate::player::PlayState::Paused),
        })
        .await;
    player.map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn patch_player(
    State(state): State<AppState>,
    Path((_session, guild_id)): Path<(String, String)>,
    Json(body): Json<UpdatePlayerRequest>,
) -> Result<Json<PlayerResponse>, StatusCode> {
    state
        .players()
        .get_or_create(&guild_id)
        .await
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let resp = state
        .players()
        .with_player(&guild_id, |p| {
            if let Some(vol) = body.volume {
                p.set_volume(vol);
            }
            if let Some(true) = body.paused {
                p.pause();
            }
            if let Some(true) = body.paused.map(|b| !b) {
                p.resume();
            }
            if let Some(track) = body.encoded_track.as_deref() {
                p.play(Track::new(track, track));
            }
            PlayerResponse {
                guild_id: guild_id.clone(),
                track: p.current_track().cloned(),
                volume: p.volume(),
                paused: matches!(p.state(), crate::player::PlayState::Paused),
            }
        })
        .await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(resp))
}
