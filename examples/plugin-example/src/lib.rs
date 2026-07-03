//! Example LoNode plugin: a stub source that claims `example://` URLs.
//!
//! Build with `cargo build --release` and copy the resulting `.so` to
//! LoNode's `plugins/` directory.

use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::{empty, AsyncRead};

pub struct ExampleSource;

#[async_trait]
impl AudioSource for ExampleSource {
    fn name(&self) -> &str {
        "example"
    }

    fn supports(&self, url: &str) -> bool {
        url.starts_with("example://")
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        Ok(TrackInfo {
            title: "Example Track".into(),
            author: "Example Plugin".into(),
            duration_ms: 0,
            url: url.to_string(),
        })
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        // Real plugins return a byte stream of compressed audio here.
        Ok(Box::new(empty()))
    }
}

/// C entry point called by LoNode's plugin loader.
#[no_mangle]
pub extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource {
    Box::into_raw(Box::new(ExampleSource))
}
