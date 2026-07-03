//! Spotify resolver tests.

use lonode_plugin_api::AudioSource;
use lonode_source_spotify::{SpotifyClient, SpotifyCredentials, SpotifyResolver};

#[test]
fn disabled_without_credentials() {
    let s = SpotifyResolver::new();
    assert!(!s.supports("https://open.spotify.com/track/abc"));
}

#[test]
fn detects_spotify_urls() {
    assert!(SpotifyResolver::is_spotify_url(
        "https://open.spotify.com/track/abc123"
    ));
    assert!(SpotifyResolver::is_spotify_url("spotify:track:abc123"));
    assert!(!SpotifyResolver::is_spotify_url(
        "https://youtube.com/watch?v=x"
    ));
}

#[test]
fn extracts_track_id_from_url() {
    assert_eq!(
        SpotifyResolver::extract_track_id("https://open.spotify.com/track/abc123?si=xyz"),
        Some("abc123")
    );
    assert_eq!(
        SpotifyResolver::extract_track_id("spotify:track:abc123"),
        Some("abc123")
    );
    assert_eq!(
        SpotifyResolver::extract_track_id("https://example.com"),
        None
    );
}

#[test]
fn name_is_spotify() {
    assert_eq!(SpotifyResolver::new().name(), "spotify");
}

#[test]
fn client_constructs_with_creds() {
    let creds = SpotifyCredentials {
        client_id: "id".into(),
        client_secret: "secret".into(),
    };
    let _c = SpotifyClient::new(creds);
}
