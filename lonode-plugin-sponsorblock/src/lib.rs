//! `lonode-plugin-sponsorblock` — skip sponsor segments in YouTube videos.
//!
//! Uses the [SponsorBlock API](https://sponsor.ajay.app/) to fetch segment
//! timestamps for a given YouTube video ID. The player can use these to
//! auto-skip sponsored content, intros, outros, etc.
//!
//! Public API:
//! - [`SponsorBlockClient`] — HTTP client for the SponsorBlock API.
//! - [`Segment`] — a single sponsor segment (start/end in seconds + category).
//! - [`Category`] — segment category (sponsor, intro, outro, selfpromo, etc.).

pub mod client;
pub mod types;

pub use client::SponsorBlockClient;
pub use types::{Category, Segment, VideoChapters};
