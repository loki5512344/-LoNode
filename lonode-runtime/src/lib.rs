//! `lonode-runtime` — voice session orchestration, player state, plugin loader.
//!
//! Public API:
//! - [`player`] — `Track`, `Queue`, `GuildPlayer`, `PlayerManager`.
//! - [`plugins`] — `PluginRegistry`, `LoadedPlugin`, `load_plugin`.
//! - [`runner`] — `run_voice_session` end-to-end.

pub mod player;
pub mod plugins;
pub mod runner;

/// Crate-level error type.
pub type Result<T> = anyhow::Result<T>;
