//! Integration tests for built-in sources (YouTube tests are in lonode-source-youtube).

use lonode_plugin_api::AudioSource;
use lonode_sources_builtin::{
    BandcampSource, RadioSource, SoundCloudSource, TwitchSource, VimeoSource,
};

#[test]
fn soundcloud_disabled_without_client_id() {
    let s = SoundCloudSource::new();
    assert!(!s.supports("https://soundcloud.com/artist/track"));
}

#[test]
fn soundcloud_detects_urls() {
    assert!(SoundCloudSource::is_soundcloud_url(
        "https://soundcloud.com/artist/track"
    ));
    assert!(!SoundCloudSource::is_soundcloud_url(
        "https://youtube.com/watch?v=x"
    ));
}

#[test]
fn soundcloud_name() {
    assert_eq!(SoundCloudSource::new().name(), "soundcloud");
}

#[test]
fn radio_rejects_known_platforms() {
    let r = RadioSource::new();
    assert!(!r.supports("https://soundcloud.com/a/t"));
    assert!(!r.supports("https://open.spotify.com/track/abc"));
    assert!(!r.supports("https://youtube.com/watch?v=x"));
    assert!(!r.supports("https://music.apple.com/us/album/x/123"));
    assert!(!r.supports("https://www.deezer.com/track/123"));
}

#[test]
fn radio_accepts_generic_http() {
    let r = RadioSource::new();
    assert!(r.supports("http://stream.example.com/mp3"));
}

#[test]
fn bandcamp_detects_urls() {
    assert!(BandcampSource::is_bandcamp_url(
        "https://artist.bandcamp.com/track/song"
    ));
    assert!(!BandcampSource::is_bandcamp_url("https://example.com"));
}

#[test]
fn twitch_detects_urls() {
    assert!(TwitchSource::is_twitch_url("https://twitch.tv/videos/123"));
    assert!(!TwitchSource::is_twitch_url("https://example.com"));
}

#[test]
fn vimeo_detects_urls() {
    assert!(VimeoSource::is_vimeo_url("https://vimeo.com/123456"));
    assert!(!VimeoSource::is_vimeo_url("https://example.com"));
}

#[test]
fn all_stubs_disabled() {
    assert!(!BandcampSource::new().supports("https://artist.bandcamp.com/track/x"));
    assert!(!TwitchSource::new().supports("https://twitch.tv/videos/1"));
    assert!(!VimeoSource::new().supports("https://vimeo.com/1"));
}
