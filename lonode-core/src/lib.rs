//! `lonode-core` — shared foundation for all LoNode crates.
//!
//! Contains only cross-cutting concerns:
//! - [`config`] — TOML configuration types and loader.
//! - [`Result`] — crate-wide error alias.
//!
//! Voice gateway lives in `lonode-gateway`, UDP in `lonode-udp`, audio
//! pipeline in `lonode-audio`, REST/WS API in `lonode-api`, orchestration
//! in `lonode-runtime`.

pub mod config;

/// Crate-level error type.
pub type Result<T> = anyhow::Result<T>;
