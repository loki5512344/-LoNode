//! Voice Gateway actor: owns the WebSocket connection, drives heartbeats,
//! and translates wire messages into [`GatewayEvent`]s.

use super::{GatewayCommand, GatewayEvent, VoiceConfig};
use crate::gateway::op::{
    Envelope, Heartbeat, Hello, Identify, Op, Ready, SelectProtocol, SelectProtocolData,
    SessionDescription,
};
use crate::Result;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{Duration, MissedTickBehavior};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

/// Main actor loop. Owns the WebSocket until [`GatewayCommand::Close`] or
/// the server disconnects.
pub async fn run(
    cfg: VoiceConfig,
    events: mpsc::Sender<GatewayEvent>,
    mut commands: mpsc::Receiver<GatewayCommand>,
) -> Result<()> {
    let endpoint = format!("{}/?v=4", cfg.endpoint);
    tracing::info!(%endpoint, "voice gateway connecting");
    let (ws, _) = connect_async(&endpoint).await?;
    let (mut write, mut read) = ws.split();

    let interval_ms = wait_hello(&mut read).await?;
    let mut ticker = tokio::time::interval(Duration::from_millis(u64::from(interval_ms)));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Delay);

    let identify = Envelope {
        op: Op::Identify.to_u8(),
        d: Identify {
            server_id: cfg.guild_id,
            user_id: cfg.user_id,
            session_id: cfg.session_id,
            token: cfg.token,
        },
    };
    write
        .send(Message::Text(serde_json::to_string(&identify)?))
        .await?;

    let mut nonce: u64 = 0;
    loop {
        tokio::select! {
            msg = read.next() => match msg {
                Some(Ok(Message::Text(t))) => {
                    if let Some(ev) = dispatch(&t)? {
                        if events.send(ev).await.is_err() { break; }
                    }
                }
                Some(Ok(_)) => {}
                Some(Err(e)) => return Err(e.into()),
                None => break,
            },
            _ = ticker.tick() => {
                nonce = nonce.wrapping_add(1);
                let hb = Envelope { op: Op::Heartbeat.to_u8(), d: Heartbeat { t: nonce } };
                write.send(Message::Text(serde_json::to_string(&hb)?)).await?;
            }
            cmd = commands.recv() => match cmd {
                Some(GatewayCommand::SelectProtocol { address, port, mode }) => {
                    let sp = Envelope {
                        op: Op::SelectProtocol.to_u8(),
                        d: SelectProtocol {
                            protocol: "udp".into(),
                            data: SelectProtocolData { address, port, mode },
                        },
                    };
                    write.send(Message::Text(serde_json::to_string(&sp)?)).await?;
                }
                Some(GatewayCommand::Close) | None => break,
            },
        }
    }
    let _ = write.send(Message::Close(None)).await;
    let _ = events
        .send(GatewayEvent::Closed("gateway closed".into()))
        .await;
    Ok(())
}

/// Read messages until Hello (op 8) is received, returning `heartbeat_interval`.
async fn wait_hello<S>(read: &mut S) -> Result<u32>
where
    S: StreamExt<Item = std::result::Result<Message, tokio_tungstenite::tungstenite::Error>>
        + Unpin,
{
    while let Some(msg) = read.next().await {
        if let Message::Text(t) = msg? {
            let env: Envelope<serde_json::Value> = serde_json::from_str(&t)?;
            if Op::from_u8(env.op) == Some(Op::Hello) {
                let hello: Hello = serde_json::from_value(env.d)?;
                return Ok(hello.heartbeat_interval);
            }
        }
    }
    anyhow::bail!("connection closed before Hello")
}

/// Map a text frame into a [`GatewayEvent`] (or `None` if uninteresting).
fn dispatch(text: &str) -> Result<Option<GatewayEvent>> {
    let env: Envelope<serde_json::Value> = serde_json::from_str(text)?;
    let Some(op) = Op::from_u8(env.op) else {
        return Ok(None);
    };
    Ok(match op {
        Op::Ready => {
            let r: Ready = serde_json::from_value(env.d)?;
            Some(GatewayEvent::Ready {
                ssrc: r.ssrc,
                ip: r.ip,
                port: r.port,
                modes: r.modes,
            })
        }
        Op::SessionDescription => {
            let sd: SessionDescription = serde_json::from_value(env.d)?;
            Some(GatewayEvent::SessionDescription {
                secret_key: sd.secret_key,
            })
        }
        _ => None,
    })
}
