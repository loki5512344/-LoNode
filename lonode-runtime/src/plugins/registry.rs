//! Plugin registry — combines built-in sources with dynamic `.so` plugins.

use super::loader::{load_plugin, LoadedPlugin};
use anyhow::Result;
use lonode_plugin_api::AudioSource;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Holds all sources (built-in + dynamic). Sources are queried in
/// registration order; the first whose `supports()` returns `true` wins.
#[derive(Clone)]
pub struct PluginRegistry {
    inner: Arc<RwLock<Vec<RegistryEntry>>>,
}

struct RegistryEntry {
    source: Arc<dyn AudioSource>,
    /// Library handle kept alive for dynamic plugins (`None` for built-ins).
    _lib: Option<Arc<libloading::Library>>,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PluginRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a built-in source (no library handle to keep alive).
    pub async fn register_builtin(&self, source: Arc<dyn AudioSource>) {
        self.inner
            .write()
            .await
            .push(RegistryEntry { source, _lib: None });
    }

    /// Register a dynamically-loaded plugin (keeps its `.so` alive).
    pub async fn register_plugin(&self, plugin: LoadedPlugin) {
        let lib = plugin.lib.clone();
        self.inner.write().await.push(RegistryEntry {
            source: Arc::from(plugin.source),
            _lib: Some(lib),
        });
    }

    /// Load all `.so` files from `dir`. Returns the number successfully loaded.
    /// Silently skips non-`.so` files; logs warnings for unloadable `.so` files.
    pub async fn load_dir(&self, dir: &str) -> Result<usize> {
        let path = Path::new(dir);
        if !path.is_dir() {
            tracing::debug!(dir, "plugins dir does not exist, skipping");
            return Ok(0);
        }
        let mut count = 0;
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let ep = entry.path();
            if ep.extension().and_then(|e| e.to_str()) == Some("so") {
                match load_plugin(&ep) {
                    Ok(p) => {
                        tracing::info!(path = %ep.display(), name = p.source.name(), "loaded plugin");
                        self.register_plugin(p).await;
                        count += 1;
                    }
                    Err(e) => {
                        tracing::warn!(path = %ep.display(), error = %e, "failed to load plugin")
                    }
                }
            }
        }
        Ok(count)
    }

    /// Find the first source that supports `url`.
    pub async fn find_for(&self, url: &str) -> Option<Arc<dyn AudioSource>> {
        for entry in self.inner.read().await.iter() {
            if entry.source.supports(url) {
                return Some(Arc::clone(&entry.source));
            }
        }
        None
    }

    /// Number of registered sources.
    pub async fn len(&self) -> usize {
        self.inner.read().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.inner.read().await.is_empty()
    }

    /// Names of all registered sources (for `/v4/info` capability reporting).
    pub async fn source_names(&self) -> Vec<String> {
        self.inner
            .read()
            .await
            .iter()
            .map(|e| e.source.name().to_string())
            .collect()
    }
}
