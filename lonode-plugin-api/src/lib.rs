//! `lonode-plugin-api` — public contract for LoNode audio sources.
//!
//! Both built-in sources (radio, YouTube — added in Phase 3) and dynamic
//! `.so` plugins (Phase 4) implement the [`AudioSource`] trait defined here.
//!
//! # Modules
//! - [`trait_`] — re-exports [`AudioSource`] (named `trait_` because `trait`
//!   is a reserved word; the type is exported at crate root).
//! - [`types`] — [`TrackInfo`] and [`PluginError`].

mod r#trait;
mod types;

pub use r#trait::AudioSource;
pub use types::{PluginError, TrackInfo};
