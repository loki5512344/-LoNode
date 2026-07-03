//! RTP header construction for Discord voice packets.
//!
//! Discord uses a fixed 12-byte RTP header (no CSRC, no extensions).
//! Reference: <https://datatracker.ietf.org/doc/html/rfc3550#section-5.1>.

/// Discord's static payload type for Opus audio.
pub const PAYLOAD_TYPE: u8 = 0x78;

/// 12-byte RTP header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RtpHeader {
    /// Per-packet sequence counter (wraps at u16::MAX).
    pub sequence: u16,
    /// Per-sample timestamp in 48 kHz units (wraps at u32::MAX).
    pub timestamp: u32,
    /// Synchronization Source identifier (per Discord voice session).
    pub ssrc: u32,
}

impl RtpHeader {
    /// Serialize to 12 big-endian bytes.
    #[must_use]
    pub fn to_bytes(self) -> [u8; 12] {
        let mut buf = [0u8; 12];
        buf[0] = 0x80; // V=2, P=0, X=0, CC=0
        buf[1] = PAYLOAD_TYPE;
        buf[2..4].copy_from_slice(&self.sequence.to_be_bytes());
        buf[4..8].copy_from_slice(&self.timestamp.to_be_bytes());
        buf[8..12].copy_from_slice(&self.ssrc.to_be_bytes());
        buf
    }

    /// Parse a 12-byte slice into an [`RtpHeader`].
    ///
    /// # Errors
    /// Returns an error if the slice is shorter than 12 bytes or if the
    /// version field isn't `2`.
    pub fn from_bytes(buf: &[u8]) -> crate::Result<Self> {
        if buf.len() < 12 {
            anyhow::bail!("rtp header too short: {} bytes", buf.len());
        }
        if (buf[0] >> 6) != 2 {
            anyhow::bail!("invalid rtp version: {}", buf[0] >> 6);
        }
        Ok(Self {
            sequence: u16::from_be_bytes([buf[2], buf[3]]),
            timestamp: u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            ssrc: u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]),
        })
    }
}

/// A full RTP packet: header + (already encrypted) payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RtpPacket {
    pub header: RtpHeader,
    pub payload: Vec<u8>,
}

impl RtpPacket {
    /// Serialize header + payload into a single `Vec`.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let header = self.header.to_bytes();
        let mut buf = Vec::with_capacity(12 + self.payload.len());
        buf.extend_from_slice(&header);
        buf.extend_from_slice(&self.payload);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let h = RtpHeader {
            sequence: 0x1234,
            timestamp: 0xDEADBEEF,
            ssrc: 0xCAFEBABE,
        };
        let bytes = h.to_bytes();
        assert_eq!(bytes[0], 0x80);
        assert_eq!(bytes[1], PAYLOAD_TYPE);
        let back = RtpHeader::from_bytes(&bytes).unwrap();
        assert_eq!(back, h);
    }

    #[test]
    fn header_rejects_short_buffer() {
        assert!(RtpHeader::from_bytes(&[0u8; 11]).is_err());
    }

    #[test]
    fn packet_concatenates_header_and_payload() {
        let pkt = RtpPacket {
            header: RtpHeader {
                sequence: 1,
                timestamp: 960,
                ssrc: 5,
            },
            payload: vec![0xAA, 0xBB],
        };
        let bytes = pkt.to_bytes();
        assert_eq!(bytes.len(), 14);
        assert_eq!(&bytes[12..], &[0xAA, 0xBB]);
    }
}
