//! Apple Music resolver tests.

use lonode_plugin_api::AudioSource;
use lonode_source_apple_music::{extract_track_id, AppleMusicSource};

#[test]
fn disabled_without_credentials() {
    let s = AppleMusicSource::new();
    assert!(!s.supports("https://music.apple.com/us/album/song/123"));
}

#[test]
fn detects_apple_music_urls() {
    assert!(AppleMusicSource::is_apple_music_url(
        "https://music.apple.com/us/album/song-name/123?i=456"
    ));
    assert!(!AppleMusicSource::is_apple_music_url(
        "https://youtube.com/watch?v=x"
    ));
}

#[test]
fn extracts_track_id_from_query_param() {
    assert_eq!(
        extract_track_id("https://music.apple.com/us/album/name/123?i=456"),
        Some("456")
    );
}

#[test]
fn extracts_track_id_from_path() {
    assert_eq!(
        extract_track_id("https://music.apple.com/us/album/name/123456"),
        Some("123456")
    );
}

#[test]
fn name_is_applemusic() {
    assert_eq!(AppleMusicSource::new().name(), "applemusic");
}
