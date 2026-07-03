//! HTTP handlers: REST endpoints + WebSocket.

pub mod rest;
pub mod ws;

pub use rest::{get_player, info, patch_player, stats};
pub use ws::ws_handler;
