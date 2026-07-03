//! YouTube resolver tests.

use lonode_plugin_api::AudioSource;
use lonode_source_youtube::{extract_video_id, YoutubeSource};

#[test]
fn detects_watch_urls() {
    assert!(YoutubeSource::is_youtube_url(
        "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
    ));
    assert!(YoutubeSource::is_youtube_url(
        "https://youtu.be/dQw4w9WgXcQ"
    ));
    assert!(YoutubeSource::is_youtube_url(
        "https://youtube.com/shorts/abc123"
    ));
}

#[test]
fn rejects_non_youtube_urls() {
    assert!(!YoutubeSource::is_youtube_url("https://example.com"));
    assert!(!YoutubeSource::is_youtube_url(
        "https://soundcloud.com/artist/track"
    ));
}

#[test]
fn extracts_video_id_from_watch_url() {
    assert_eq!(
        extract_video_id("https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=42"),
        Some("dQw4w9WgXcQ".to_string())
    );
}

#[test]
fn extracts_video_id_from_short_url() {
    assert_eq!(
        extract_video_id("https://youtu.be/dQw4w9WgXcQ?si=abc"),
        Some("dQw4w9WgXcQ".to_string())
    );
}

#[test]
fn extracts_video_id_from_shorts() {
    assert_eq!(
        extract_video_id("https://youtube.com/shorts/abc123defg"),
        Some("abc123defg".to_string())
    );
}

#[test]
fn returns_none_for_non_youtube_url() {
    assert_eq!(extract_video_id("https://example.com"), None);
}

#[test]
fn name_is_youtube() {
    assert_eq!(YoutubeSource::new().name(), "youtube");
}

#[test]
fn supports_watch_urls() {
    let y = YoutubeSource::new();
    assert!(y.supports("https://www.youtube.com/watch?v=dQw4w9WgXcQ"));
    assert!(!y.supports("https://example.com"));
}
