//! UDP transport for Discord voice: socket setup, IP discovery, sending.
//!
//! Public API: [`VoiceUdp`] — owns a [`tokio::net::UdpSocket`], the
//! negotiated SSRC, and a [`VoiceCrypto`] for per-packet encryption.

pub mod crypto;
pub mod rtp;

pub use crypto::VoiceCrypto;
pub use rtp::{RtpHeader, RtpPacket, PAYLOAD_TYPE};

use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// Size of the IP-discovery packet (74 bytes per Discord spec).
const IP_DISCOVERY_PACKET_LEN: usize = 74;
/// Offset of the IP address inside the IP-discovery packet.
const IP_OFFSET: usize = 4;
/// Offset of the port inside the IP-discovery packet.
const PORT_OFFSET: usize = 72;

/// Voice UDP sender. Owns socket + crypto state + sequence/timestamp counters.
pub struct VoiceUdp {
    socket: UdpSocket,
    dest: SocketAddr,
    ssrc: u32,
    crypto: VoiceCrypto,
    sequence: u16,
    timestamp: u32,
}

impl VoiceUdp {
    /// Bind a new UDP socket to `local_addr` (e.g. `"0.0.0.0:0"` for ephemeral).
    /// The `secret_key` is initialized to zeros; call
    /// [`set_secret_key`](Self::set_secret_key) once `op 4` arrives.
    ///
    /// # Errors
    /// Returns an error if the socket cannot be bound.
    pub async fn bind(local_addr: &str, ssrc: u32) -> Result<Self> {
        let socket = UdpSocket::bind(local_addr).await?;
        Ok(Self {
            socket,
            dest: "0.0.0.0:0".parse()?,
            ssrc,
            crypto: VoiceCrypto::new(&[0u8; 32]),
            sequence: 0,
            timestamp: 0,
        })
    }

    /// Install the real `secret_key` (from `op 4` Session Description).
    /// Must be called before [`send_opus`](Self::send_opus) is invoked.
    pub fn set_secret_key(&mut self, secret_key: [u8; 32]) {
        self.crypto = VoiceCrypto::new(&secret_key);
    }

    /// Perform Discord's IP discovery: send the SSRC, receive our external
    /// IP+port. Must be called before [`send_opus`](Self::send_opus).
    ///
    /// # Errors
    /// Returns an error if the discovery exchange fails or yields an
    /// unparseable address.
    pub async fn discover_ip(&mut self, server: SocketAddr) -> Result<SocketAddr> {
        self.dest = server;
        let mut packet = [0u8; IP_DISCOVERY_PACKET_LEN];
        packet[..4].copy_from_slice(&self.ssrc.to_be_bytes());
        self.socket.send_to(&packet, server).await?;

        let mut buf = [0u8; IP_DISCOVERY_PACKET_LEN];
        let (n, _) = self.socket.recv_from(&mut buf).await?;
        if n < IP_DISCOVERY_PACKET_LEN {
            anyhow::bail!("ip discovery response too short: {n} bytes");
        }
        let ip_str = std::str::from_utf8(&buf[IP_OFFSET..PORT_OFFSET])?
            .trim_end_matches('\0')
            .to_string();
        let port = u16::from_be_bytes([buf[PORT_OFFSET], buf[PORT_OFFSET + 1]]);
        Ok(format!("{ip_str}:{port}").parse()?)
    }

    /// Encrypt + send a single Opus frame as an RTP packet.
    /// Sequence/timestamp advance automatically (20 ms @ 48 kHz = 960 samples).
    ///
    /// # Errors
    /// Returns an error if encryption or the underlying `send_to` fails.
    pub async fn send_opus(&mut self, opus_frame: &[u8]) -> Result<()> {
        let header = RtpHeader {
            sequence: self.sequence,
            timestamp: self.timestamp,
            ssrc: self.ssrc,
        };
        let header_bytes = header.to_bytes();
        let payload = self.crypto.encrypt(&header_bytes, opus_frame)?;
        let packet = RtpPacket { header, payload };
        self.socket.send_to(&packet.to_bytes(), self.dest).await?;
        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(960);
        Ok(())
    }

    /// Send a UDP keep-alive packet (Discord expects the bytes `\x00\x00\x00\x00`
    /// every ~5 s while silent).
    ///
    /// # Errors
    /// Returns an error if the underlying `send_to` fails.
    pub async fn send_keepalive(&self) -> Result<()> {
        self.socket.send_to(&[0u8; 4], self.dest).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_offsets_are_consistent() {
        assert_eq!(PORT_OFFSET - IP_OFFSET, 68); // 64-byte IP field + 4 bytes
        assert_eq!(IP_DISCOVERY_PACKET_LEN, PORT_OFFSET + 2);
    }
}
