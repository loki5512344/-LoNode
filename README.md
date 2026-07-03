# LoNode

Self-hosted Discord audio node. Drop-in Lavalink alternative written in Rust.

## Features

- Discord Voice UDP (Opus, XSalsa20-Poly1305 encryption)
- WebSocket + REST API (Lavalink-compatible) — Phase 2
- Internet radio (Icecast / Shoutcast / HLS) — Phase 3
- Plugin system (`.so` dynamic sources) — Phase 4
- Single binary, single node

## Workspace

```
lonode/
├── lonode/             # Binary — wires everything together
├── lonode-core/        # Voice UDP, audio pipeline, runner
├── lonode-sources/     # Built-in sources (radio, YouTube) — Phase 3
└── lonode-plugin-api/  # Public trait contract for plugins
```

## Stack

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `tokio-tungstenite` | Voice Gateway WebSocket |
| `axum` | REST + WebSocket (Phase 2) |
| `symphonia` | Audio decoding (Phase 3) |
| `opus` | Opus encoding |
| `xsalsa20poly1305` | UDP packet encryption |
| `async-trait` | Plugin trait (dyn-compatible) |

## Architecture

```
Bot (any language)
      │  WebSocket / REST
      ▼
┌─────────────────────┐
│      LoNode         │
│  ┌───────────────┐  │
│  │  lonode bin   │  │  entry point + tracing init
│  └──────┬────────┘  │
│         │           │
│  ┌──────▼────────┐  │
│  │  lonode-core  │  │  gateway + udp + audio + runner
│  └──────┬────────┘  │
│         │           │
│  ┌──────▼────────┐  │
│  │   sources     │  │  radio / youtube / .so plugins
│  └───────────────┘  │
└─────────────────────┘
      │  Discord Voice UDP
      ▼
 Discord servers
```

## Quick Start

```bash
git clone https://github.com/loki5512344/lonode
cd lonode
cp config.example.toml config.toml
cargo run -p lonode
```

## Config

```toml
[server]
host = "0.0.0.0"
port = 2333
password = "youshallnotpass"
```

Phase 5 will extend this with `[audio]`, `[sources]`, and `[limits]` sections.

## API

Compatible with Lavalink v4 WebSocket protocol.
REST endpoints documented in `docs/api.md` (Phase 2).

## Plugin Development

See `docs/plugin-dev.md` (Phase 4). Plugins implement
`lonode_plugin_api::AudioSource` and are loaded from `./plugins/`.

## License

MIT
