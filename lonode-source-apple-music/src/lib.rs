//! `lonode-source-apple-music` — Apple Music resolver for LoNode.
//!
//! Apple Music has no public audio streaming API. Like Spotify, we resolve
//! track metadata via the [Apple Music API](https://api.music.apple.com/)
//! and defer playback to the YouTube source (search `"{artist} - {title}"`).
//!
//! Public API:
//! - [`AppleMusicSource`] — `AudioSource` impl.
//! - [`extract_track_id`] — parse `music.apple.com/track/` URLs.

pub mod client;
pub mod resolver;

pub use client::{AppleMusicClient, AppleMusicCredentials, AppleMusicError};
pub use resolver::{extract_track_id, AppleMusicSource, AppleMusicTrack};
