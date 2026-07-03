# LoNode API

Lavalink v4 compatible REST + WebSocket API.

## Authentication

All endpoints require the `Authorization` header with the configured password:

```
Authorization: youshallnotpass
```

Requests without (or with wrong) auth return `401 Unauthorized`.

## REST Endpoints

### `GET /v4/info`

Returns node version and enabled sources.

```json
{
  "version": "0.1.0",
  "build_time": "",
  "git": "",
  "node_region": "local",
  "enabled_sources": ["radio", "youtube"]
}
```

### `GET /v4/stats`

Returns runtime statistics.

```json
{
  "players": 0,
  "playing_players": 0,
  "uptime": 42,
  "memory": { "reserved": 0, "used": 0 }
}
```

### `GET /v4/sessions/{sessionId}/players/{guildId}`

Returns the current state of a guild's player. Returns `404` if no player
exists for the guild.

```json
{
  "guild_id": "123",
  "track": { "id": "...", "title": "...", "author": "...", "duration_ms": 0, "url": "" },
  "volume": 100,
  "paused": false
}
```

### `PATCH /v4/sessions/{sessionId}/players/{guildId}`

Updates a guild's player. Creates the player if it doesn't exist.

Request body (all fields optional):
```json
{
  "encoded_track": "http://stream.example.com/mp3",
  "volume": 80,
  "paused": false
}
```

If `encoded_track` is an `http(s)://` URL, LoNode checks that a registered
source supports it. Returns `400 Bad Request` if no source claims the URL.

## WebSocket

### `GET /v4/websocket`

Upgrades to a WebSocket. On connect the server sends a `ready` op with a
session id:

```json
{ "op": "Ready", "data": { "sessionId": "lonode-..." } }
```

Client messages (subset of Lavalink v4):

```json
{ "op": "play",    "guildId": "123" }
{ "op": "stop",    "guildId": "123" }
{ "op": "pause",   "guildId": "123" }
{ "op": "resume",  "guildId": "123" }
```

Server events:

```json
{ "op": "Event", "guildId": "123", "type": "TrackStartEvent", "data": {} }
{ "op": "Event", "guildId": "123", "type": "TrackEndEvent",   "data": {} }
```
```
