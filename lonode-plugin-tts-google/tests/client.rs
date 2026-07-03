//! Google Cloud TTS client tests.

use lonode_plugin_tts_google::{AudioEncoding, Gender, TtsClient, TtsRequest, Voice};

#[test]
fn voice_builds_with_defaults() {
    let v = Voice::new("en-US");
    assert_eq!(v.language_code, "en-US");
    assert!(v.name.is_none());
    assert!(v.ssml_gender.is_none());
}

#[test]
fn voice_chains_builders() {
    let v = Voice::new("en-US")
        .with_name("en-US-Wavenet-D")
        .with_gender(Gender::Male);
    assert_eq!(v.name.as_deref(), Some("en-US-Wavenet-D"));
    assert_eq!(v.ssml_gender, Some(Gender::Male));
}

#[test]
fn tts_request_defaults_to_mp3() {
    let r = TtsRequest::text("hello", "en-US");
    assert_eq!(r.encoding, AudioEncoding::Mp3);
    assert!(!r.is_ssml);
    assert_eq!(r.rate, 1.0);
}

#[test]
fn tts_request_ssml_flag() {
    let r = TtsRequest::text("<speak>hi</speak>", "en-US").ssml();
    assert!(r.is_ssml);
}

#[test]
fn client_constructs_with_api_key() {
    let _c = TtsClient::new("test-key");
}
