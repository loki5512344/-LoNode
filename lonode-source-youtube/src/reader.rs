//! Adapter: rusty_ytdl's `Stream` trait → `tokio::io::AsyncRead`.

use rusty_ytdl::stream::Stream as YtdlStream;
use std::io::Cursor;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

/// Wraps a `rusty_ytdl::stream::Stream` into an `AsyncRead` by draining all
/// chunks into a `Cursor<Vec<u8>>`.
///
/// For large videos this buffers everything in memory before returning. A
/// streaming variant would require a custom `AsyncRead` impl with a background
/// task — tracked as a TODO.
pub struct YtdlReader {
    cursor: Cursor<Vec<u8>>,
}

impl YtdlReader {
    /// Drain `stream` into a buffer and return a reader over it.
    ///
    /// # Errors
    /// Returns an error if any chunk read fails.
    pub async fn from_stream(stream: Box<dyn YtdlStream + Send + Sync>) -> std::io::Result<Self> {
        let mut buf = Vec::new();
        let s = stream;
        loop {
            match s.chunk().await {
                Ok(Some(bytes)) => buf.extend_from_slice(&bytes),
                Ok(None) => break,
                Err(e) => return Err(std::io::Error::other(e.to_string())),
            }
        }
        Ok(Self {
            cursor: Cursor::new(buf),
        })
    }
}

impl AsyncRead for YtdlReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.cursor).poll_read(cx, buf)
    }
}
