//! `lonode-core` — core library of LoNode.
//!
//! Exposes seven modules:
//! - [`config`] — TOML configuration parsing.
//! - [`gateway`] — Discord Voice Gateway (WebSocket) wire types and actor.
//! - [`udp`] — RTP packet construction and XSalsa20-Poly1305 encryption.
//! - [`audio`] — `FrameSource` trait and Opus encoder wrapper.
//! - [`player`] — per-guild player state and registry.
//! - [`api`] — axum REST + WebSocket handlers (Lavalink v4 compatible).
//! - [`runner`] — orchestrates a full voice session end-to-end.

pub mod api;
pub mod audio;
pub mod config;
pub mod gateway;
pub mod player;
pub mod runner;
pub mod udp;

/// Crate-level error type.
pub type Result<T> = anyhow::Result<T>;
