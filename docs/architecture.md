# LoNode Architecture

## Workspace Layout

```
lonode/
├── lonode/                    # Binary — entry point, tracing, source wiring
├── lonode-core/               # Config (TOML), shared types
├── lonode-gateway/            # Discord Voice Gateway WebSocket client
├── lonode-udp/                # RTP packets + XSalsa20-Poly1305 encryption
├── lonode-audio/              # FrameSource trait + Opus encoder
├── lonode-runtime/            # Player manager + plugin loader + voice runner
├── lonode-api/                # axum REST + WebSocket (Lavalink v4)
├── lonode-plugin-api/         # Public trait contract for .so plugins
├── lonode-sources-builtin/    # Radio, SoundCloud, Bandcamp, Twitch, Vimeo, YouTube
└── lonode-source-spotify/     # Spotify resolver (YouTube matching pattern)
```

## Dependency Graph

```
                    lonode (binary)
                   ┌───┴───────────────────┐
                   │                       │
              lonode-api              lonode-runtime
              │       │              │      │      │
        lonode-core  lonode-runtime  │      │      │
              │       │          ┌───┘      │      │
              │       │     lonode-gateway  │      │
              │       │          │     lonode-udp │
              │       │          │          │     │
              │       │     lonode-audio ◄──┘     │
              │       │                            │
              │       └──► lonode-plugin-api ◄────┘
              │                  ▲
              │                  │
        lonode-sources-builtin ──┤
        lonode-source-spotify ───┘
```

## Source Resolution Flow

When a `PATCH /v4/sessions/{sid}/players/{gid}` request arrives with an
`encoded_track` URL:

1. **URL validation**: if the track is an `http(s)://` URL, LoNode queries
   the `PluginRegistry` for a source that `supports()` it.
2. **Source lookup**: sources are checked in registration order:
   - Platform sources first (youtube, soundcloud, spotify, bandcamp, twitch, vimeo)
   - Radio LAST (fallback for any HTTP URL not claimed by a specific source)
3. **No source found** → `400 Bad Request`
4. **Source found** → track is added to the guild's queue, player state updated

## Spotify Handling

Spotify doesn't allow direct audio streaming. LoNode follows the Lavalink
pattern:

1. `SpotifyResolver.supports(url)` returns `true` for `open.spotify.com/track/`
   URLs (only when `SPOTIFY_CLIENT_ID` + `SPOTIFY_CLIENT_SECRET` env vars are set).
2. `resolve(url)` queries the Spotify Web API for track metadata (title, artist, duration).
3. `stream(url)` returns an error — the runtime layer is responsible for taking
   the resolved metadata, building a YouTube search query (`"{artist} - {title}"`),
   and handing it to the YouTube source for actual playback.

This keeps Spotify legal (no ToS violations) while still enabling playback.

## Plugin System

Dynamic `.so` plugins are loaded from `config.sources.plugins_dir` (default
`./plugins/`). Each plugin exports:

```c
extern "C" fn lonode_plugin_init() -> *mut dyn AudioSource;
```

The loader (`lonode-runtime::plugins::loader`) calls this function, wraps the
returned pointer in `Box::from_raw`, and keeps the `Library` alive for as long
as the source is registered. See `docs/plugin-dev.md` for the full guide.

## Configuration

See `config.example.toml`. Sections:

- `[server]` — host, port, password
- `[audio]` — buffer_ms, opus_bitrate
- `[sources]` — youtube, radio, plugins_dir
- `[limits]` — max_players, max_queue

## Phase Status

| Phase | Status |
|-------|--------|
| 1 — Discord Voice UDP | ✅ Complete |
| 2 — Player + API | ✅ Complete |
| 3 — Sources | ✅ Radio + Spotify; stubs for YouTube/SoundCloud/Bandcamp/Twitch/Vimeo |
| 4 — Plugin System | ✅ Complete |
| 5 — Config + Ops | 🚧 Partial (config done; hot reload, Docker, systemd pending) |
