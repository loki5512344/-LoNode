//! Dynamic `.so` plugin loading via `libloading`.
//!
//! Each plugin must export:
//! ```c
//! extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource;
//! ```

use anyhow::Result;
use libloading::{Library, Symbol};
use lonode_plugin_api::AudioSource;
use std::path::Path;
use std::sync::Arc;

/// A loaded plugin: the source instance + the library it came from.
/// Keeping `_lib` alive prevents the vtable / function pointers from being
/// unloaded while the source is still in use.
pub struct LoadedPlugin {
    pub source: Box<dyn AudioSource>,
    /// Library handle; must stay alive while `source` is in use.
    pub lib: Arc<Library>,
}

/// Load a single `.so` plugin from `path`.
///
/// # Safety
/// Calls `lonode_plugin_init` which is `extern "C"`. The plugin must be
/// compiled against the same `lonode-plugin-api` version (ABI is not stable
/// across versions).
///
/// # Errors
/// Returns an error if the file cannot be opened, the symbol is missing,
/// or the init function returns a null pointer.
pub fn load_plugin(path: &Path) -> Result<LoadedPlugin> {
    let lib = unsafe { Library::new(path) }?;
    let lib = Arc::new(lib);
    let init: Symbol<unsafe extern "C" fn() -> *mut dyn AudioSource> =
        unsafe { lib.get(b"lonode_plugin_init") }?;
    let raw = unsafe { init() };
    if raw.is_null() {
        anyhow::bail!("lonode_plugin_init returned null for {}", path.display());
    }
    let source = unsafe { Box::from_raw(raw) };
    Ok(LoadedPlugin { source, lib })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_nonexistent_returns_error() {
        let res = load_plugin(Path::new("/nonexistent/plugin.so"));
        assert!(res.is_err());
    }

    #[test]
    fn load_regular_file_returns_error() {
        // A file that exists but isn't a shared library.
        let tmp = std::env::temp_dir().join("lonode_not_a_plugin.txt");
        std::fs::write(&tmp, "not a plugin").unwrap();
        let res = load_plugin(&tmp);
        assert!(res.is_err());
        let _ = std::fs::remove_file(&tmp);
    }
}
