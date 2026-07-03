//! `FrameSource` trait and built-in frame producers.
//!
//! Discord expects 20 ms Opus frames at 48 kHz stereo. PCM frames here are
//! 1920 `i16` samples per channel-pair (= 960 stereo samples × 2 channels).

/// PCM samples per 20 ms frame at 48 kHz (per channel).
pub const SAMPLES_PER_FRAME: usize = 960;
/// Total `i16` samples in one 20 ms stereo frame (960 × 2 channels).
pub const FRAME_SAMPLES: usize = SAMPLES_PER_FRAME * 2;

/// One 20 ms PCM stereo frame, 1920 `i16` samples.
pub type PcmFrame = [i16; FRAME_SAMPLES];

/// Uniform interface for any source of 20 ms PCM frames.
///
/// This is distinct from `lonode_plugin_api::AudioSource` (which resolves
/// URLs to byte streams): `FrameSource` sits at the Opus-encoder boundary
/// and yields ready-to-encode PCM.
pub trait FrameSource: Send {
    /// Return the next PCM frame, or `None` at end-of-stream.
    fn next_frame(&mut self) -> Option<PcmFrame>;
}

/// Produces infinite silence. Used during Discord's mandatory 5-frame
/// pre-speak padding and as a fallback when no source is queued.
#[derive(Debug, Default, Clone, Copy)]
pub struct SilenceSource;

impl FrameSource for SilenceSource {
    fn next_frame(&mut self) -> Option<PcmFrame> {
        Some([0; FRAME_SAMPLES])
    }
}

/// Plays a pre-decoded buffer of `i16` stereo samples to exhaustion.
/// Useful for testing the Opus pipeline without touching the filesystem.
#[derive(Debug, Clone)]
pub struct PcmSource {
    samples: Vec<i16>,
    cursor: usize,
}

impl PcmSource {
    /// Construct from an existing `Vec<i16>` (will be consumed in
    /// [`FRAME_SAMPLES`]-sized chunks).
    #[must_use]
    pub fn new(samples: Vec<i16>) -> Self {
        Self { samples, cursor: 0 }
    }

    /// Number of complete frames still available.
    #[must_use]
    pub fn remaining_frames(&self) -> usize {
        (self.samples.len().saturating_sub(self.cursor)) / FRAME_SAMPLES
    }
}

impl FrameSource for PcmSource {
    fn next_frame(&mut self) -> Option<PcmFrame> {
        if self.cursor + FRAME_SAMPLES > self.samples.len() {
            return None;
        }
        let mut frame = [0i16; FRAME_SAMPLES];
        frame.copy_from_slice(&self.samples[self.cursor..self.cursor + FRAME_SAMPLES]);
        self.cursor += FRAME_SAMPLES;
        Some(frame)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn silence_never_ends() {
        let mut s = SilenceSource;
        for _ in 0..1_000 {
            assert!(s.next_frame().is_some());
        }
    }

    #[test]
    fn pcm_source_returns_none_when_short() {
        let mut s = PcmSource::new(vec![0; 10]);
        assert!(s.next_frame().is_none());
    }

    #[test]
    fn pcm_source_counts_remaining() {
        // 3 full frames worth of samples.
        let s = PcmSource::new(vec![1; FRAME_SAMPLES * 3]);
        assert_eq!(s.remaining_frames(), 3);
    }
}
