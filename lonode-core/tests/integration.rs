//! Integration tests: exercise the public API as an external consumer would.
//!
//! These tests verify that `config`, `audio`, `udp`, and `gateway` compose
//! correctly end-to-end (without touching the network).

use lonode_core::audio::{AudioPipeline, OpusEncoder, PcmSource, SilenceSource, FRAME_SAMPLES};
use lonode_core::config;
use lonode_core::gateway::{GatewayCommand, GatewayEvent, Op, VoiceConfig};
use lonode_core::udp::{RtpHeader, VoiceCrypto};

const SAMPLE_TOML: &str = r#"
[server]
host = "127.0.0.1"
port = 2333
password = "youshallnotpass"
"#;

#[test]
fn config_parses_lavalink_defaults() {
    let cfg = config::parse(SAMPLE_TOML).expect("toml parses");
    assert_eq!(cfg.server.host, "127.0.0.1");
    assert_eq!(cfg.server.port, 2333);
    assert_eq!(cfg.server.password, "youshallnotpass");
}

#[test]
fn opus_pipeline_produces_packets_for_real_pcm() {
    // 3 frames worth of progressively-louder samples.
    let samples: Vec<i16> = (0..(FRAME_SAMPLES * 3))
        .map(|i| (i as i16).wrapping_mul(2))
        .collect();
    let mut pipeline = AudioPipeline::new(PcmSource::new(samples)).expect("pipeline builds");
    let mut packet_sizes = Vec::new();
    while let Some(frame) = pipeline.next_opus_frame() {
        packet_sizes.push(frame.expect("encode ok").len());
    }
    assert_eq!(packet_sizes.len(), 3);
    // Opus packets should be non-empty and bounded by MAX_OPUS_PACKET.
    for size in &packet_sizes {
        assert!(*size > 0 && *size <= 4_000);
    }
}

#[test]
fn rtp_packet_round_trips_through_crypto() {
    let key = [0xABu8; 32];
    let crypto = VoiceCrypto::new(&key);
    let header = RtpHeader {
        sequence: 42,
        timestamp: 960,
        ssrc: 0xDEAD_BEEF,
    };
    let header_bytes = header.to_bytes();

    // Encode a silence frame (real Opus output).
    let mut enc = OpusEncoder::new().expect("encoder builds");
    let opus_frame = enc.encode_silence().expect("encode ok");

    // Encrypt as if we were about to send over UDP.
    let ciphertext = crypto
        .encrypt(&header_bytes, &opus_frame)
        .expect("encrypt ok");
    assert_eq!(ciphertext.len(), opus_frame.len() + 16); // +Poly1305 tag

    // Decrypt back.
    let plain = crypto
        .decrypt(&header_bytes, &ciphertext)
        .expect("decrypt ok");
    assert_eq!(plain, opus_frame);
}

#[test]
fn voice_config_carries_all_fields() {
    let cfg = VoiceConfig {
        endpoint: "wss://example.com".into(),
        guild_id: "1".into(),
        user_id: "2".into(),
        session_id: "abc".into(),
        token: "xyz".into(),
    };
    assert_eq!(cfg.endpoint, "wss://example.com");
    assert_eq!(cfg.guild_id, "1");
    assert_eq!(cfg.user_id, "2");
    assert_eq!(cfg.session_id, "abc");
    assert_eq!(cfg.token, "xyz");
}

#[test]
fn gateway_command_select_protocol_is_constructible() {
    let cmd = GatewayCommand::SelectProtocol {
        address: "203.0.113.5".into(),
        port: 50_006,
        mode: "xsalsa20_poly1305".into(),
    };
    let GatewayCommand::SelectProtocol {
        address,
        port,
        mode,
    } = cmd
    else {
        panic!("wrong variant");
    };
    assert_eq!(address, "203.0.113.5");
    assert_eq!(port, 50_006);
    assert_eq!(mode, "xsalsa20_poly1305");
}

#[test]
fn gateway_event_session_description_holds_32_byte_key() {
    let ev = GatewayEvent::SessionDescription {
        secret_key: [7u8; 32],
    };
    let GatewayEvent::SessionDescription { secret_key } = ev else {
        panic!("wrong variant");
    };
    assert_eq!(secret_key.len(), 32);
}

#[test]
fn op_select_protocol_round_trips() {
    assert_eq!(
        Op::from_u8(Op::SelectProtocol.to_u8()),
        Some(Op::SelectProtocol)
    );
}

#[test]
fn silence_source_and_pipeline_agree_on_padding() {
    let mut p = AudioPipeline::new(SilenceSource).expect("pipeline builds");
    for _ in 0..AudioPipeline::<SilenceSource>::SILENCE_PADDING {
        let pkt = p.encode_silence().expect("silence encodes");
        assert!(!pkt.is_empty());
    }
}
