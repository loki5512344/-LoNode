# TODO

## Workspace структура

```
lonode/
├── lonode/                    # Бинарник, собирает всё вместе
├── lonode-core/               # Config, общие типы
├── lonode-gateway/            # Discord Voice Gateway WebSocket
├── lonode-udp/                # RTP + XSalsa20-Poly1305 encryption
├── lonode-audio/              # FrameSource trait + Opus encoder
├── lonode-runtime/            # Player manager + plugin loader + runner
├── lonode-api/                # axum REST + WebSocket (Lavalink v4)
├── lonode-plugin-api/         # Публичные traits для плагинов (.so)
├── lonode-sources-builtin/    # Radio, SoundCloud, Bandcamp, Twitch, Vimeo, YouTube
└── lonode-source-spotify/     # Spotify (YouTube matching pattern)
```

---

## Phase 1 — Discord Voice UDP (сейчас)

- [x] Подключение к Discord Voice Gateway (WSS)
  - [x] Отправить `Identify` payload (op 0)
  - [x] Получить `READY` (op 2) — ssrc, ip, port
  - [x] Отправить `Select Protocol` (op 1) — UDP, XSalsa20-Poly1305
  - [x] Получить `Session Description` (op 4) — secret_key
  - [x] Heartbeat loop (op 3 / op 8)
- [x] UDP-сокет
  - [x] IP Discovery (отправить пустой пакет, получить внешний IP/port)
  - [x] Собирать RTP-пакеты (header + зашифрованный payload)
  - [x] Шифрование XSalsa20-Poly1305 (`xsalsa20poly1305` крейт)
  - [x] Отправка Opus-фреймов каждые 20 мс (точный таймер через `tokio::time::interval`)
  - [x] Keep-alive UDP пакеты
- [x] Аудио пайплайн
  - [x] Trait `FrameSource` — единый интерфейс для PCM-фреймов
  - [ ] Буфер фреймов (кольцевой, ~1 сек ahead)
  - [ ] Symphonia: декод PCM из байтов
  - [x] Opus энкодинг PCM → opus frame (48kHz, stereo, 20ms)
  - [x] Silence frames когда буфер пустой (5 штук перед паузой — требование Discord)

## Phase 2 — Player + API

- [x] Player Manager
  - [x] Структура `Guild` (voice state, queue, current track)
  - [x] Команды: play, stop, pause, resume, skip, seek, volume
  - [x] Очередь треков
- [x] axum WebSocket
  - [x] Lavalink v4 протокол (events: TrackStart, TrackEnd, TrackException)
  - [x] Аутентификация по паролю (header `Authorization`)
- [x] axum REST
  - [x] `GET /v4/info`
  - [x] `GET /v4/stats`
  - [ ] `PATCH /v4/sessions/{sessionId}`
  - [x] `GET/POST/PATCH/DELETE /v4/sessions/{sessionId}/players/{guildId}`

## Phase 3 — Источники

- [x] Интернет-радио
  - [x] Icecast / Shoutcast (chunked HTTP stream)
  - [ ] HLS (`m3u8` парсинг + сегменты)
  - [x] ICY metadata (название трека из стрима)
- [x] YouTube
  - [ ] `rusty-ytdl` интеграция
  - [ ] Поиск по запросу
  - [ ] Плейлисты
- [x] SoundCloud
  - [x] URL detection + API resolve (нужен client_id)
  - [ ] Playlist support
- [x] Bandcamp (stub)
- [x] Twitch (stub)
- [x] Vimeo (stub)
- [x] Spotify
  - [x] Spotify API client (track metadata)
  - [x] YouTube matching pattern (resolve → search YouTube → play)
  - [ ] Playlist/album support

## Phase 4 — Plugin System

- [x] Trait `AudioSource` как публичный интерфейс плагина
  ```rust
  pub trait AudioSource: Send + Sync {
      fn name(&self) -> &str;
      fn supports(&self, url: &str) -> bool;
      async fn resolve(&self, url: &str) -> Result<TrackInfo>;
      async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead>>;
  }
  ```
- [x] Встроенные источники (trait-based, компилируются вместе)
  - [x] Radio (`lonode-sources/radio`)
  - [x] YouTube (`lonode-sources/youtube`) — stub
- [x] Динамические плагины (`.so` через `libloading`)
  - [x] Загрузка из папки `plugins/` при старте
  - [x] Единая точка входа `lonode_plugin_init() -> *mut dyn AudioSource`
  - [ ] Горячая перезагрузка (опционально, позже)
- [x] Реестр источников — при `play` перебирает все плагины, первый у кого `supports()` = true берёт задачу

## Phase 5 — Config

- [x] `config.toml`
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
- [ ] Hot reload конфига (SIGHUP)
- [x] Логи (`tracing` крейт, уровень через config)
- [x] Graceful shutdown (ctrl-c)
- [x] Docker / `Dockerfile` (в `docs/deployment/`)
- [x] `docs/api.md`
- [x] `docs/plugin-dev.md` — гайд по написанию плагинов

## Phase 6 — Plugins (Lavalink-эквиваленты)

Реализация плагинов по аналогии с Lavalink plugin ecosystem. Каждый плагин —
отдельный crate в workspace, регистрируется через `PluginRegistry`.

### YouTube Plugin (`lonode-source-youtube`)
- [x] URL detection (youtube.com/watch, youtu.be/)
- [ ] `rusty-ytdl` интеграция — real video download + audio extraction
- [ ] Поиск по запросу (search query → track)
- [ ] Плейлисты (youtube.com/playlist)
- [ ] Stream as AsyncRead (compressed audio bytes)

### LavaSrc Plugin (`lonode-source-lavasrc`)
Lavalink LavaSrc эквивалент — Spotify, Apple Music, Deezer.
- [x] Spotify (уже в `lonode-source-spotify` — YouTube matching pattern)
- [ ] Apple Music API resolver (needs developer token)
  - [ ] Track URL detection (music.apple.com/track/)
  - [ ] Catalog search
  - [ ] YouTube matching for playback (Apple Music has no public streaming)
- [ ] Deezer resolver (native playback — Deezer has public streaming API)
  - [ ] Track URL detection (deezer.com/track/)
  - [ ] API resolve → stream URL
  - [ ] Direct stream (no YouTube fallback needed)

### SponsorBlock Plugin (`lonode-plugin-sponsorblock`)
- [ ] SponsorBlock API client (sponsor.ajay.app API)
  - [ ] Get segments for YouTube video ID
  - [ ] Skip sponsor segments during playback
  - [ ] Return chapter information (YouTube video chapters)
- [ ] Integration with player (auto-skip on segment boundary)

### Google Cloud TTS Plugin (`lonode-plugin-tts-google`)
- [ ] Google Cloud Text-to-Speech API client
  - [ ] Synthesize text → MP3/Opus bytes
  - [ ] Voice selection (language, gender, name)
  - [ ] SSML support
- [ ] AudioSource impl — generate audio on-the-fly from text
- [ ] Config: `GOOGLE_APPLICATION_CREDENTIALS` env var

### План интеграции
1. YouTube (foundation — нужен для Spotify/Apple Music matching)
2. SponsorBlock (depends on YouTube)
3. LavaSrc: Apple Music + Deezer
4. Google Cloud TTS (standalone)

