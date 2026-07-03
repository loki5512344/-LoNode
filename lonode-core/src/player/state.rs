//! Per-guild player state (transport controls).

use super::Queue;
use super::Track;
use std::time::Duration;

/// Playback status for one guild.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Paused,
    Playing,
}

/// Per-guild player. Owns queue + transport state.
#[derive(Debug)]
pub struct GuildPlayer {
    queue: Queue,
    state: PlayState,
    volume: u16,
    position: Duration,
}

impl Default for GuildPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl GuildPlayer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
            state: PlayState::Stopped,
            volume: 100,
            position: Duration::ZERO,
        }
    }

    #[must_use]
    pub fn current_track(&self) -> Option<&Track> {
        self.queue.current()
    }

    /// Append a track. If stopped, the track becomes current and playback starts.
    pub fn play(&mut self, track: Track) {
        if self.state == PlayState::Stopped && self.queue.current().is_none() {
            self.queue.advance();
            self.queue.push(track);
            self.queue.advance();
        } else {
            self.queue.push(track);
        }
        self.state = PlayState::Playing;
    }

    pub fn pause(&mut self) {
        if self.state == PlayState::Playing {
            self.state = PlayState::Paused;
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlayState::Paused {
            self.state = PlayState::Playing;
        }
    }

    pub fn stop(&mut self) {
        self.queue.clear();
        self.state = PlayState::Stopped;
        self.position = Duration::ZERO;
    }

    pub fn skip(&mut self) -> bool {
        let next = self.queue.advance().is_some();
        if !next {
            self.state = PlayState::Stopped;
        }
        self.position = Duration::ZERO;
        next
    }

    /// Set volume (0–1000, where 100 = 100%). Out-of-range values are clamped.
    pub fn set_volume(&mut self, volume: u16) {
        self.volume = volume.min(1_000);
    }

    #[must_use]
    pub const fn volume(&self) -> u16 {
        self.volume
    }

    #[must_use]
    pub const fn state(&self) -> PlayState {
        self.state
    }

    #[must_use]
    pub const fn position(&self) -> Duration {
        self.position
    }

    #[must_use]
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }
}
