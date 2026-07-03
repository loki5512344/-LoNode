//! Plugin registry tests.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use lonode_runtime::plugins::PluginRegistry;
use std::sync::Arc;
use tokio::io::{empty, AsyncRead};

struct Dummy;

#[async_trait]
impl AudioSource for Dummy {
    fn name(&self) -> &str {
        "dummy"
    }
    fn supports(&self, url: &str) -> bool {
        url.starts_with("dummy://")
    }
    async fn resolve(&self, _: &str) -> std::result::Result<TrackInfo, PluginError> {
        Ok(TrackInfo::default())
    }
    async fn stream(
        &self,
        _: &str,
    ) -> std::result::Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Ok(Box::new(empty()))
    }
}

#[tokio::test]
async fn register_builtin_and_find() {
    let reg = PluginRegistry::new();
    reg.register_builtin(Arc::new(Dummy)).await;
    assert_eq!(reg.len().await, 1);
    assert!(reg.find_for("dummy://x").await.is_some());
    assert!(reg.find_for("http://x").await.is_none());
}

#[tokio::test]
async fn load_dir_missing_returns_zero() {
    let reg = PluginRegistry::new();
    let n = reg.load_dir("/nonexistent/plugins").await.unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn source_names_returns_all() {
    let reg = PluginRegistry::new();
    reg.register_builtin(Arc::new(Dummy)).await;
    let names = reg.source_names().await;
    assert_eq!(names, vec!["dummy".to_string()]);
}
