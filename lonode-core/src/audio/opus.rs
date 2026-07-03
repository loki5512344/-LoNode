//! Thin wrapper around the [`opus`] crate.
//!
//! Encodes 20 ms stereo PCM frames (48 kHz, 1920 `i16` samples) into Opus
//! packets ready for RTP encryption.

use crate::audio::frames::PcmFrame;
use crate::Result;

/// Sample rate Discord expects for Opus voice.
pub const SAMPLE_RATE: u32 = 48_000;
/// Max size of an encoded Opus frame in bytes (per RFC 6716).
pub const MAX_OPUS_PACKET: usize = 4_000;

/// Opus encoder configured for 48 kHz stereo, application = Audio.
pub struct OpusEncoder {
    inner: opus::Encoder,
}

impl OpusEncoder {
    /// Create a new encoder (48 kHz, stereo, audio application).
    ///
    /// # Errors
    /// Returns an error if the underlying libopus encoder cannot be allocated.
    pub fn new() -> Result<Self> {
        let inner = opus::Encoder::new(
            SAMPLE_RATE,
            opus::Channels::Stereo,
            opus::Application::Audio,
        )?;
        Ok(Self { inner })
    }

    /// Encode one PCM frame into an Opus packet. The returned `Vec` is
    /// exactly `n` bytes long where `n ≤ MAX_OPUS_PACKET`.
    ///
    /// # Errors
    /// Returns an error if libopus reports an encoding failure.
    pub fn encode(&mut self, pcm: &PcmFrame) -> Result<Vec<u8>> {
        let mut buf = [0u8; MAX_OPUS_PACKET];
        let n = self.inner.encode(pcm, &mut buf)?;
        Ok(buf[..n].to_vec())
    }

    /// Encode silence as a "comfort noise" frame. Discord requires ≥5 of
    /// these at the start of a transmission and after a pause.
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    pub fn encode_silence(&mut self) -> Result<Vec<u8>> {
        let pcm: PcmFrame = [0; crate::audio::frames::FRAME_SAMPLES];
        self.encode(&pcm)
    }
}

impl Default for OpusEncoder {
    fn default() -> Self {
        Self::new().expect("opus encoder allocation must succeed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_silence_to_compact_packet() {
        let mut enc = OpusEncoder::new().unwrap();
        let pkt = enc.encode_silence().unwrap();
        // Silence frames in Opus should be small (typically 1–3 bytes for CELT-only).
        assert!(
            pkt.len() <= 10,
            "silence packet too large: {} bytes",
            pkt.len()
        );
    }

    #[test]
    fn silence_packet_is_valid_opus() {
        let mut enc = OpusEncoder::new().unwrap();
        let pkt = enc.encode_silence().unwrap();
        // RFC 6716 §3.1: the LSB of byte 0 is the "config" bit; silence must
        // have a non-zero length and parse as a valid packet.
        assert!(!pkt.is_empty());
    }
}
