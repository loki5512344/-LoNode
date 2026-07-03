# Plugin Development

LoNode supports dynamic audio source plugins (`.so` files on Linux). A plugin
is a shared library that exports a single C entry point.

## ABI Contract

```c
// Returns a heap-allocated trait object. LoNode takes ownership and will
// reclaim it via `Box::from_raw`.
extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource;
```

## Minimal Plugin

`Cargo.toml`:
```toml
[package]
name = "my-source"
version = "0.1.0"
edition = "2021"

[lib]
name = "my_source"
crate-type = ["cdylib"]

[dependencies]
lonode-plugin-api = { path = "../lonode-plugin-api" }
async-trait = "0.1"
tokio = { version = "1", default-features = false, features = ["io-util"] }
```

`src/lib.rs`:
```rust
use async_trait::async_trait;
use lonode_plugin_api::{AudioSource, PluginError, TrackInfo};
use tokio::io::{empty, AsyncRead};

pub struct MySource;

#[async_trait]
impl AudioSource for MySource {
    fn name(&self) -> &str { "my-source" }

    fn supports(&self, url: &str) -> bool {
        url.starts_with("my://")
    }

    async fn resolve(&self, url: &str) -> Result<TrackInfo, PluginError> {
        Ok(TrackInfo {
            title: "My Track".into(),
            author: "My Source".into(),
            duration_ms: 0,
            url: url.to_string(),
        })
    }

    async fn stream(&self, _url: &str) -> Result<Box<dyn AsyncRead + Send + Unpin>, PluginError> {
        Ok(Box::new(empty()))
    }
}

#[no_mangle]
pub extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource {
    Box::into_raw(Box::new(MySource))
}
```

## Building & Installing

```bash
cd my-source
cargo build --release
cp target/release/libmy_source.so /path/to/lonode/plugins/
```

LoNode scans `plugins_dir` (default `./plugins/`) at startup and loads every
`.so` file. The plugin's `name()` is logged; `supports()` is queried when a
`play` command comes in with a URL.

## ABI Stability Warning

Plugins **must** be compiled against the same `lonode-plugin-api` version as
the LoNode binary. Trait object layouts are not stable across versions.
When upgrading LoNode, recompile plugins against the new version.

## Hot Reload

Not yet implemented (planned). For now, restart LoNode to pick up plugin
changes.
```
