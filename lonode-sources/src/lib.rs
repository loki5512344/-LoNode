//! `lonode-sources` — built-in audio sources for LoNode.
//!
//! Phase 3 will add `radio` (Icecast/Shoutcast/HLS) and `youtube`
//! (rusty-ytdl) modules here. Each module exports a type implementing
//! `lonode_plugin_api::AudioSource`.
//!
//! The crate is currently empty by design — adding empty modules would
//! violate the "no dead code" rule.
