//! Player manager and per-guild state.
//!
//! Public API:
//! - [`Track`], [`TrackId`] — queue entries (defined here).
//! - [`Queue`] — per-guild FIFO queue (defined here).
//! - [`GuildPlayer`] — transport state (play/pause/skip/volume).
//! - [`PlayerManager`] — shared registry indexed by guild id.

pub mod manager;
pub mod state;

pub use manager::PlayerManager;
pub use state::{GuildPlayer, PlayState};

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Opaque track identifier (base64-encoded track data in Lavalink v4).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrackId(pub String);

/// Metadata for a queued track. Mirrors Lavalink v4 `Track.info`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub title: String,
    pub author: String,
    /// Duration in milliseconds (`0` for live streams).
    pub duration_ms: u64,
    pub url: String,
}

impl Track {
    #[must_use]
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: TrackId(id.into()),
            title: title.into(),
            author: String::new(),
            duration_ms: 0,
            url: String::new(),
        }
    }
}

/// Per-guild play queue. FIFO with optional repeat.
#[derive(Debug, Default)]
pub struct Queue {
    tracks: VecDeque<Track>,
    current: Option<Track>,
}

impl Queue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn current(&self) -> Option<&Track> {
        self.current.as_ref()
    }

    pub fn push(&mut self, track: Track) {
        self.tracks.push_back(track);
    }

    /// Advance to the next track: pops the front of `tracks` into `current`.
    /// Returns the new current track, or `None` if the queue is exhausted.
    pub fn advance(&mut self) -> Option<&Track> {
        self.current = self.tracks.pop_front();
        self.current.as_ref()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty() && self.current.is_none()
    }

    pub fn clear(&mut self) {
        self.tracks.clear();
    }
}
