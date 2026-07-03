//! Dynamic plugin system for LoNode.
//!
//! - [`loader`] — loads `.so` files and calls `lonode_plugin_init`.
//! - [`registry`] — combines built-in sources with dynamic plugins.
//!
//! Plugin ABI contract:
//! ```c
//! extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource;
//! ```
//! The returned pointer must be allocated via `Box::into_raw(Box::new(...))`
//! and will be reclaimed by LoNode via `Box::from_raw`.

pub mod loader;
pub mod registry;

pub use loader::LoadedPlugin;
pub use registry::PluginRegistry;
