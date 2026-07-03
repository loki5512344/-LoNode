//! Audio pipeline: turn a [`FrameSource`] into Opus packets.
//!
//! Public API:
//! - [`FrameSource`] trait — anything that yields 20 ms PCM frames.
//! - [`AudioPipeline`] — pairs a source with an [`OpusEncoder`].
//! - [`AudioPipeline::next_opus_frame`] — pull one Opus packet.

pub mod frames;
pub mod opus;

pub use frames::{
    FrameSource, PcmFrame, PcmSource, SilenceSource, FRAME_SAMPLES, SAMPLES_PER_FRAME,
};
pub use opus::{OpusEncoder, MAX_OPUS_PACKET, SAMPLE_RATE};

use crate::Result;

/// Couples a [`FrameSource`] with an [`OpusEncoder`].
///
/// The caller is responsible for the 20 ms ticker and for sending the
/// leading/trailing 5 silence frames Discord requires (use
/// [`encode_silence`](Self::encode_silence) for those).
pub struct AudioPipeline<S: FrameSource> {
    source: S,
    encoder: OpusEncoder,
}

impl<S: FrameSource> AudioPipeline<S> {
    /// Create a pipeline around `source` with a fresh Opus encoder.
    ///
    /// # Errors
    /// Returns an error if the Opus encoder cannot be allocated.
    pub fn new(source: S) -> Result<Self> {
        Ok(Self {
            source,
            encoder: OpusEncoder::new()?,
        })
    }

    /// Pull the next PCM frame from the source, encode it to Opus, and
    /// return the packet. `None` means end-of-stream.
    ///
    /// # Errors
    /// Returns an error if Opus encoding fails.
    pub fn next_opus_frame(&mut self) -> Option<Result<Vec<u8>>> {
        let pcm = self.source.next_frame()?;
        Some(self.encoder.encode(&pcm))
    }

    /// Encode a silence frame (used for Discord's 5-frame pre-speak padding
    /// and trailing padding before pause).
    ///
    /// # Errors
    /// Returns an error if encoding fails.
    pub fn encode_silence(&mut self) -> Result<Vec<u8>> {
        self.encoder.encode_silence()
    }

    /// Number of leading silence frames Discord requires before real audio.
    pub const SILENCE_PADDING: usize = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_drains_pcm_source() {
        let samples: Vec<i16> = (0..(FRAME_SAMPLES as i16 * 3)).collect();
        let mut p = AudioPipeline::new(PcmSource::new(samples)).unwrap();
        let mut count = 0;
        while p.next_opus_frame().is_some() {
            count += 1;
        }
        assert_eq!(count, 3);
    }

    #[test]
    fn silence_padding_constant_is_five() {
        assert_eq!(AudioPipeline::<SilenceSource>::SILENCE_PADDING, 5);
    }
}
