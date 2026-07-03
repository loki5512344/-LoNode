//! Discord Voice Gateway op-codes and payload types.
//!
//! Reference: <https://discord.com/developers/docs/topics/voice-connections#voice-gateway-opcodes>.

use serde::{Deserialize, Serialize};

/// Voice Gateway op-code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// Client → Server: start a new session.
    Identify,
    /// Client → Server: pick transport protocol & encryption mode.
    SelectProtocol,
    /// Server → Client: contains ssrc, voice server IP/port, available modes.
    Ready,
    /// Client → Server: keep-alive (every `heartbeat_interval` ms).
    Heartbeat,
    /// Server → Client: contains the secret_key used for UDP encryption.
    SessionDescription,
    /// Client ↔ Server: speaking flag.
    Speaking,
    /// Server → Client: heartbeat acknowledged.
    HeartbeatAck,
    /// Server → Client: first message, contains `heartbeat_interval`.
    Hello,
    /// Server → Client: session resumed.
    Resumed,
    /// Server → Client: another user disconnected.
    ClientDisconnect,
}

impl Op {
    /// Convert a raw wire byte into an [`Op`].
    #[must_use]
    pub const fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Identify),
            1 => Some(Self::SelectProtocol),
            2 => Some(Self::Ready),
            3 => Some(Self::Heartbeat),
            4 => Some(Self::SessionDescription),
            5 => Some(Self::Speaking),
            6 => Some(Self::HeartbeatAck),
            8 => Some(Self::Hello),
            9 => Some(Self::Resumed),
            12 => Some(Self::ClientDisconnect),
            _ => None,
        }
    }

    /// Convert an [`Op`] back to its wire byte.
    #[must_use]
    pub const fn to_u8(self) -> u8 {
        match self {
            Self::Identify => 0,
            Self::SelectProtocol => 1,
            Self::Ready => 2,
            Self::Heartbeat => 3,
            Self::SessionDescription => 4,
            Self::Speaking => 5,
            Self::HeartbeatAck => 6,
            Self::Hello => 8,
            Self::Resumed => 9,
            Self::ClientDisconnect => 12,
        }
    }
}

/// Wrapper envelope: `{ "op": N, "d": ... }`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<D> {
    pub op: u8,
    pub d: D,
}

/// `op 0` Identify payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identify {
    pub server_id: String,
    pub user_id: String,
    pub session_id: String,
    pub token: String,
}

/// `op 8` Hello payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hello {
    /// Heartbeat interval in milliseconds.
    pub heartbeat_interval: u32,
}

/// `op 2` Ready payload (subset we care about).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ready {
    pub ssrc: u32,
    pub ip: String,
    pub port: u16,
    pub modes: Vec<String>,
}

/// `op 1` Select Protocol payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectProtocol {
    pub protocol: String,
    pub data: SelectProtocolData,
}

/// Inner `data` field of [`SelectProtocol`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectProtocolData {
    pub address: String,
    pub port: u16,
    pub mode: String,
}

/// `op 4` Session Description payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDescription {
    pub mode: String,
    pub secret_key: [u8; 32],
}

/// `op 3` Heartbeat payload — any integer; Discord echoes it back via ACK.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heartbeat {
    pub t: u64,
}
