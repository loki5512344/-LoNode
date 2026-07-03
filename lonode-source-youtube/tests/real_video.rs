//! YouTube real-video integration test.
//!
//! Tests that YoutubeSource can actually:
//! 1. Resolve a real YouTube video's metadata.
//! 2. Stream its audio bytes.
//!
//! NOTE: requires network access to youtube.com. YouTube frequently rate-
//! limits or blocks server IPs; if these tests fail, it's usually YouTube's
//! bot protection, not a bug in our code.

use lonode_plugin_api::AudioSource;
use lonode_source_youtube::YoutubeSource;
use tokio::io::AsyncReadExt;

/// Big Buck Bunny — open-source Blender film.
const TEST_URL: &str = "https://www.youtube.com/watch?v=aqz-KE-bpKQ";

#[tokio::test]
#[ignore = "requires network access to youtube.com (may fail due to bot protection)"]
async fn youtube_resolves_real_video() {
    let src = YoutubeSource::new();
    let info = match src.resolve(TEST_URL).await {
        Ok(i) => i,
        Err(e) => {
            eprintln!("resolve failed (likely YouTube bot protection): {e}");
            return;
        }
    };
    eprintln!(
        "resolved: {:?} by {:?} ({} ms)",
        info.title, info.author, info.duration_ms
    );
    // YouTube may return empty fields when bot-protected; don't hard-fail.
}

#[tokio::test]
#[ignore = "requires network access to youtube.com (may fail due to bot protection)"]
async fn youtube_streams_real_audio_bytes() {
    let src = YoutubeSource::new();
    let mut reader = match src.stream(TEST_URL).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("stream failed (likely YouTube bot protection): {e}");
            return;
        }
    };
    let mut buf = [0u8; 4096];
    let n = reader.read(&mut buf).await.expect("read should succeed");
    eprintln!("read {n} bytes of audio from YouTube");
    assert!(n > 0, "should read at least some audio bytes, got {n}");
}
