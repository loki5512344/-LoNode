//! Yandex Music resolver tests.

use lonode_plugin_api::AudioSource;
use lonode_source_yandex_music::{extract_track_id, YandexMusicClient, YandexMusicSource};

#[test]
fn detects_yandex_music_urls() {
    assert!(YandexMusicSource::is_yandex_music_url(
        "https://music.yandex.com/track/123456"
    ));
    assert!(YandexMusicSource::is_yandex_music_url(
        "https://music.yandex.ru/track/123456"
    ));
    assert!(!YandexMusicSource::is_yandex_music_url(
        "https://example.com"
    ));
}

#[test]
fn extracts_track_id() {
    assert_eq!(
        extract_track_id("https://music.yandex.com/track/123456?from=app"),
        Some("123456")
    );
    assert_eq!(
        extract_track_id("https://music.yandex.ru/track/789"),
        Some("789")
    );
}

#[test]
fn returns_none_for_non_yandex_url() {
    assert_eq!(extract_track_id("https://example.com"), None);
}

#[test]
fn name_is_yandexmusic() {
    assert_eq!(YandexMusicSource::new().name(), "yandexmusic");
}

#[test]
fn client_without_token_cannot_stream_full() {
    let c = YandexMusicClient::new();
    assert!(!c.has_token());
}

#[test]
fn client_with_token_can_stream_full() {
    let c = YandexMusicClient::with_token("ya.123".into(), "42".into());
    assert!(c.has_token());
}
