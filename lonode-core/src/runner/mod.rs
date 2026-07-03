//! Voice session orchestrator.
//!
//! Wires together the three Phase-1 subsystems:
//! 1. [`gateway`] — Voice Gateway WebSocket (Ready, SessionDescription, …).
//! 2. [`udp`] — IP discovery + encrypted Opus sending.
//! 3. [`audio`] — PCM source → Opus encoder.
//!
//! Flow:
//! ```text
//! connect(WS) → Ready → UDP bind + IP discovery → SelectProtocol cmd
//!            → SessionDescription → set_secret_key → 5 silence frames
//!            → stream silence until gateway closes.
//! ```

use crate::audio::{AudioPipeline, SilenceSource};
use crate::gateway::{connect, GatewayCommand, GatewayEvent, VoiceConfig};
use crate::udp::VoiceUdp;
use crate::Result;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::time::interval;

/// 20 ms frame cadence (48 kHz × 20 ms = 960 samples).
const FRAME_INTERVAL: Duration = Duration::from_millis(20);
/// Discord's encryption mode for Phase 1.
const CRYPTO_MODE: &str = "xsalsa20_poly1305";

/// Run a single voice session end-to-end. Blocks until the gateway closes
/// or an unrecoverable error occurs.
///
/// # Errors
/// Returns an error if any subsystem fails critically (WS, UDP, Opus).
pub async fn run_voice_session(cfg: VoiceConfig) -> Result<()> {
    let mut handle = connect(cfg)?;
    let mut udp: Option<VoiceUdp> = None;

    while let Some(event) = handle.events.recv().await {
        match event {
            GatewayEvent::Ready {
                ssrc,
                ip,
                port,
                modes: _,
            } => {
                let server = parse_socket_addr(&ip, port)?;
                let mut sock = VoiceUdp::bind("0.0.0.0:0", ssrc).await?;
                let local = sock.discover_ip(server).await?;
                handle
                    .commands
                    .send(GatewayCommand::SelectProtocol {
                        address: local.ip().to_string(),
                        port: local.port(),
                        mode: CRYPTO_MODE.to_string(),
                    })
                    .await?;
                udp = Some(sock);
            }
            GatewayEvent::SessionDescription { secret_key } => {
                let Some(mut sock) = udp.take() else {
                    tracing::warn!("SessionDescription before Ready — ignored");
                    continue;
                };
                sock.set_secret_key(secret_key);
                stream_silence(&mut sock).await?;
            }
            GatewayEvent::Closed(reason) => {
                tracing::info!(%reason, "voice session ended");
                break;
            }
        }
    }
    Ok(())
}

/// Send the 5 leading silence frames, then stream silence at 20 ms cadence
/// until the socket errors. Will be replaced by real `AudioSource` plumbing
/// in Phase 2.
async fn stream_silence(sock: &mut VoiceUdp) -> Result<()> {
    let mut pipeline = AudioPipeline::new(SilenceSource)?;
    for _ in 0..AudioPipeline::<SilenceSource>::SILENCE_PADDING {
        sock.send_opus(&pipeline.encode_silence()?).await?;
    }
    let mut ticker = interval(FRAME_INTERVAL);
    loop {
        ticker.tick().await;
        let frame = match pipeline.next_opus_frame() {
            Some(Ok(b)) => b,
            Some(Err(e)) => return Err(e),
            None => return Ok(()),
        };
        sock.send_opus(&frame).await?;
    }
}

fn parse_socket_addr(ip: &str, port: u16) -> Result<SocketAddr> {
    Ok(format!("{ip}:{port}").parse()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ipv4_socket_addr() {
        let addr = parse_socket_addr("127.0.0.1", 5004).unwrap();
        assert_eq!(addr.port(), 5004);
        assert!(addr.is_ipv4());
    }

    #[test]
    fn rejects_garbage_ip() {
        assert!(parse_socket_addr("not_an_ip", 5004).is_err());
    }
}
