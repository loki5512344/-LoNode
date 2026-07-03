//! `lonode-source-yandex-music` — Yandex Music resolver for LoNode.
//!
//! Uses the unofficial Yandex Music API (`api.music.yandex.net`) to resolve
//! track metadata and stream URLs. Tracks require a user `access_token` for
//! full streaming; without one only 30-second previews are available.
//!
//! Public API:
//! - [`YandexMusicSource`] — `AudioSource` impl for Yandex Music URLs.
//! - [`YandexMusicCredentials`] — access_token + user_id.
//! - [`extract_track_id`] — parse `music.yandex.com/track/` URLs.

pub mod client;
pub mod resolver;

pub use client::{YandexMusicClient, YandexMusicError, YandexMusicTrack};
pub use resolver::{extract_track_id, YandexMusicCredentials, YandexMusicSource};
