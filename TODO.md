# TODO

## Workspace структура

```
lonode/
├── lonode-core/       # Voice UDP, Player, API
├── lonode-sources/    # Встроенные парсеры (radio, youtube)
├── lonode-plugin-api/ # Публичные traits для плагинов (.so)
└── lonode/            # Бинарник, собирает всё вместе
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

- [ ] Player Manager
  - [ ] Структура `Guild` (voice state, queue, current track)
  - [ ] Команды: play, stop, pause, resume, skip, seek, volume
  - [ ] Очередь треков
- [ ] axum WebSocket
  - [ ] Lavalink v4 протокол (events: TrackStart, TrackEnd, TrackException)
  - [ ] Аутентификация по паролю (header `Authorization`)
- [ ] axum REST
  - [ ] `GET /v4/info`
  - [ ] `GET /v4/stats`
  - [ ] `PATCH /v4/sessions/{sessionId}`
  - [ ] `GET/POST/PATCH/DELETE /v4/sessions/{sessionId}/players/{guildId}`

## Phase 3 — Источники

- [ ] Интернет-радио
  - [ ] Icecast / Shoutcast (chunked HTTP stream)
  - [ ] HLS (`m3u8` парсинг + сегменты)
  - [ ] ICY metadata (название трека из стрима)
- [ ] YouTube
  - [ ] `rusty-ytdl` интеграция
  - [ ] Поиск по запросу
  - [ ] Плейлисты

## Phase 4 — Plugin System

- [ ] Trait `AudioSource` как публичный интерфейс плагина
  ```rust
  pub trait AudioSource: Send + Sync {
      fn name(&self) -> &str;
      fn supports(&self, url: &str) -> bool;
      async fn resolve(&self, url: &str) -> Result<TrackInfo>;
      async fn stream(&self, url: &str) -> Result<Box<dyn AsyncRead>>;
  }
  ```
- [ ] Встроенные источники (trait-based, компилируются вместе)
  - [ ] Radio (`lonode-sources/radio`)
  - [ ] YouTube (`lonode-sources/youtube`)
- [ ] Динамические плагины (`.so` через `libloading`)
  - [ ] Загрузка из папки `plugins/` при старте
  - [ ] Единая точка входа `lonode_plugin_init() -> Box<dyn AudioSource>`
  - [ ] Горячая перезагрузка (опционально, позже)
- [ ] Реестр источников — при `play` перебирает все плагины, первый у кого `supports()` = true берёт задачу

## Phase 5 — Config

- [ ] `config.toml`
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
- [ ] Логи (`tracing` крейт, уровень через config)
- [ ] Graceful shutdown
- [ ] Docker / `Dockerfile`
- [ ] `docs/api.md`
- [ ] `docs/plugin-dev.md` — гайд по написанию плагинов
