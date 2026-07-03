//! `lonode-source-youtube` — YouTube audio source for LoNode.
//!
//! Uses [`rusty_ytdl`] to download videos and extract audio. The audio stream
//! (typically Opus/WebM) is returned as an [`AsyncRead`] for downstream PCM
//! decoding by `lonode-audio`.
//!
//! Public API:
//! - [`YoutubeSource`] — `AudioSource` impl for YouTube URLs.
//! - [`extract_video_id`] — parse a YouTube URL to its 11-char video ID.

pub mod reader;
pub mod resolver;

pub use reader::YtdlReader;
pub use resolver::{extract_video_id, YoutubeSource};
