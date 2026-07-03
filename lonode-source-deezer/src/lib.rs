//! `lonode-source-deezer` — Deezer audio source for LoNode.
//!
//! Uses the [Deezer API](https://developers.deezer.com/api) to resolve track
//! metadata and stream URLs. Unlike Spotify/Apple Music, Deezer has a public
//! streaming API (with a valid `track.preview` URL — 30-second previews are
//! free; full tracks require a Deezer Premium subscription + SaaS token).
//!
//! Public API:
//! - [`DeezerSource`] — `AudioSource` impl for Deezer URLs.
//! - [`extract_track_id`] — parse `deezer.com/track/` URLs.

pub mod client;
pub mod resolver;

pub use client::{DeezerClient, DeezerError, DeezerTrack};
pub use resolver::{extract_track_id, DeezerSource};
