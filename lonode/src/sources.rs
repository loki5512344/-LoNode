//! Source registry construction — wires all built-in + dynamic sources.

use lonode_core::config;
use lonode_runtime::plugins::PluginRegistry;
use std::sync::Arc;

/// Build the source registry from config. Specific sources first, radio LAST
/// (fallback for any HTTP URL not claimed by a specific source).
pub async fn build_sources(cfg: &config::Config) -> PluginRegistry {
    let reg = PluginRegistry::new();

    if cfg.sources.youtube {
        reg.register_builtin(Arc::new(lonode_source_youtube::YoutubeSource::new()))
            .await;
        tracing::info!("registered source: youtube");
    }

    if cfg.spotify.enabled() {
        let creds = lonode_source_spotify::SpotifyCredentials {
            client_id: cfg.spotify.client_id.clone(),
            client_secret: cfg.spotify.client_secret.clone(),
        };
        reg.register_builtin(Arc::new(
            lonode_source_spotify::SpotifyResolver::with_credentials(creds),
        ))
        .await;
        tracing::info!("registered source: spotify");
    } else {
        tracing::info!("spotify disabled (set [spotify] client_id + client_secret in config)");
    }

    if cfg.apple_music.enabled() {
        let creds = lonode_source_apple_music::AppleMusicCredentials {
            developer_token: cfg.apple_music.developer_token.clone(),
            user_token: if cfg.apple_music.user_token.is_empty() {
                None
            } else {
                Some(cfg.apple_music.user_token.clone())
            },
        };
        reg.register_builtin(Arc::new(
            lonode_source_apple_music::AppleMusicSource::with_credentials(creds),
        ))
        .await;
        tracing::info!("registered source: applemusic");
    } else {
        tracing::info!("apple music disabled (set [apple_music] developer_token in config)");
    }

    register_yandex(&reg, cfg).await;
    register_builtin_platforms(&reg).await;

    if cfg.sources.radio {
        reg.register_builtin(Arc::new(lonode_sources_builtin::RadioSource::new()))
            .await;
        tracing::info!("registered source: radio (fallback)");
    }

    match reg.load_dir(&cfg.sources.plugins_dir).await {
        Ok(n) => tracing::info!(n, "loaded dynamic plugins"),
        Err(e) => tracing::warn!(error = %e, "failed to scan plugins dir"),
    }
    reg
}

async fn register_yandex(reg: &PluginRegistry, cfg: &config::Config) {
    if cfg.yandex_music.enabled() {
        let creds = lonode_source_yandex_music::YandexMusicCredentials {
            access_token: cfg.yandex_music.access_token.clone(),
            user_id: cfg.yandex_music.user_id.clone(),
        };
        reg.register_builtin(Arc::new(
            lonode_source_yandex_music::YandexMusicSource::with_credentials(creds),
        ))
        .await;
        tracing::info!("registered source: yandexmusic (full streaming)");
    } else {
        reg.register_builtin(Arc::new(
            lonode_source_yandex_music::YandexMusicSource::new(),
        ))
        .await;
        tracing::info!(
            "registered source: yandexmusic (preview only — set [yandex_music] access_token for full tracks)"
        );
    }
}

async fn register_builtin_platforms(reg: &PluginRegistry) {
    reg.register_builtin(Arc::new(lonode_source_deezer::DeezerSource::new()))
        .await;
    tracing::info!("registered source: deezer");
    reg.register_builtin(Arc::new(lonode_sources_builtin::SoundCloudSource::new()))
        .await;
    tracing::info!("registered source: soundcloud (disabled without client_id)");
    reg.register_builtin(Arc::new(lonode_sources_builtin::BandcampSource::new()))
        .await;
    reg.register_builtin(Arc::new(lonode_sources_builtin::TwitchSource::new()))
        .await;
    reg.register_builtin(Arc::new(lonode_sources_builtin::VimeoSource::new()))
        .await;
    tracing::info!("registered sources: bandcamp, twitch, vimeo (stubs)");
}
