//! `lonode-source-spotify` — Spotify resolver for LoNode.
//!
//! Spotify doesn't allow direct audio streaming via its API. The standard
//! pattern (used by Lavalink + spotify-source plugin) is:
//! 1. Query Spotify API for track metadata (title, artist, duration).
//! 2. Search YouTube for the same `{artist} - {title}`.
//! 3. Hand off to the YouTube source for actual playback.
//!
//! This crate does step 1 and returns a [`SpotifyTrack`] with a suggested
//! YouTube search query. The runtime layer wires the YouTube source for
//! step 2–3.

pub mod client;
pub mod resolver;

pub use client::{SpotifyClient, SpotifyCredentials};
pub use resolver::{SpotifyResolver, SpotifyTrack};
