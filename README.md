# LoNode

Self-hosted Discord audio node. Drop-in Lavalink alternative written in Rust.

## Features

- **Discord Voice UDP** — Opus, XSalsa20-Poly1305 encryption
- **Lavalink v4 API** — REST + WebSocket, password auth
- **Sources**:
  - Radio (Icecast/Shoutcast with ICY metadata)
  - Spotify (via YouTube matching — Lavalink pattern)
  - SoundCloud (API resolve — needs client_id)
  - Bandcamp, Twitch, Vimeo, YouTube (stubs — rusty-ytdl planned)
- **Plugin system** — load `.so` files implementing `AudioSource`
- Single binary, single node

## Workspace

```
lonode/
├── lonode/                    # Binary
├── lonode-core/               # Config, shared types
├── lonode-gateway/            # Discord Voice Gateway WebSocket
├── lonode-udp/                # RTP + encryption
├── lonode-audio/              # FrameSource + Opus encoder
├── lonode-runtime/            # Player + plugin loader + runner
├── lonode-api/                # axum REST + WebSocket
├── lonode-plugin-api/         # Plugin trait contract
├── lonode-sources-builtin/    # Radio, SoundCloud, Bandcamp, Twitch, Vimeo, YouTube
└── lonode-source-spotify/     # Spotify resolver
```

## Stack

| Crate | Purpose |
|-------|---------|
| `tokio` | Async runtime |
| `tokio-tungstenite` | Voice Gateway WebSocket |
| `axum` | REST + WebSocket API |
| `opus` | Opus encoding |
| `xsalsa20poly1305` | UDP packet encryption |
| `reqwest` | HTTP streams (radio, SoundCloud, Spotify API) |
| `libloading` | Dynamic `.so` plugin loading |
| `async-trait` | Plugin trait (dyn-compatible) |

## Quick Start

```bash
git clone https://github.com/loki5512344/-LoNode
cd LoNode
cp config.example.toml config.toml
cargo run -p lonode
```

## Configuration

```toml
[server]
host = "0.0.0.0"
port = 2333
password = "youshallnotpass"

[audio]
buffer_ms = 1000
opus_bitrate = 128000

[sources]
youtube = true
radio = true
plugins_dir = "./plugins"

[limits]
max_players = 100
max_queue = 500
```

### Spotify (optional)

Set env vars to enable Spotify URL resolution:

```bash
export SPOTIFY_CLIENT_ID=your_client_id
export SPOTIFY_CLIENT_SECRET=your_client_secret
```

Spotify URLs are resolved to metadata via the Spotify Web API, then matched
to YouTube for playback (Lavalink pattern — Spotify doesn't allow direct
streaming).

## API

See `docs/api.md` for the full REST + WebSocket reference.

Quick example:

```bash
# Get node info
curl -H "Authorization: youshallnotpass" http://localhost:2333/v4/info

# Play a radio stream
curl -X PATCH \
  -H "Authorization: youshallnotpass" \
  -H "Content-Type: application/json" \
  -d '{"encoded_track":"http://stream.example.com/mp3"}' \
  http://localhost:2333/v4/sessions/s1/players/123
```

## Plugin Development

See `docs/plugin-dev.md`. Plugins implement `lonode_plugin_api::AudioSource`
and are loaded from `./plugins/`. An example plugin is in
`examples/plugin-example/`.

## Documentation

- `docs/api.md` — REST + WebSocket API reference
- `docs/architecture.md` — internal design and dependency graph
- `docs/plugin-dev.md` — how to write `.so` plugins
- `docs/deployment/` — Dockerfile, systemd unit

## License

MIT
