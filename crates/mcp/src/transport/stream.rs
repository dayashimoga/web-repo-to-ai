//! # Generic Stream Transport
//!
//! Provides asynchronous newline-delimited JSON-RPC 2.0 message framing over any
//! byte streams implementing Tokio's `AsyncRead` and `AsyncWrite`.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader, Lines};
use tokio::sync::Mutex;
use tracing::{debug, error, trace};

use crate::error::McpError;
use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcResponse};
use crate::transport::trait_def::Transport;

/// Stream-based transport framing JSON-RPC 2.0 messages over asynchronous I/O streams.
pub struct StreamTransport<R, W> {
    reader: Mutex<Lines<BufReader<R>>>,
    writer: Mutex<W>,
}

impl<R: AsyncRead + Unpin + Send + 'static, W: AsyncWrite + Unpin + Send + 'static>
    StreamTransport<R, W>
{
    /// Constructs a new `StreamTransport` from read and write streams.
    pub fn new(read_stream: R, write_stream: W) -> Self {
        let reader = BufReader::new(read_stream).lines();
        Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(write_stream),
        }
    }
}

#[async_trait]
impl<R: AsyncRead + Unpin + Send + 'static, W: AsyncWrite + Unpin + Send + 'static> Transport
    for StreamTransport<R, W>
{
    async fn read_message(&mut self) -> Result<Option<JsonRpcMessage>, McpError> {
        let mut reader = self.reader.lock().await;

        loop {
            match reader.next_line().await {
                Ok(Some(line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        // Skip empty keepalive or blank lines
                        continue;
                    }

                    trace!(target: "mcp::stream", direction = "in", payload = %trimmed);
                    match JsonRpcMessage::parse(trimmed) {
                        Ok(msg) => return Ok(Some(msg)),
                        Err(e) => {
                            error!(target: "mcp::stream", error = %e, raw = %trimmed, "Malformed incoming JSON-RPC message");
                            return Err(e);
                        }
                    }
                }
                Ok(None) => {
                    debug!(target: "mcp::stream", "Stream reached EOF");
                    return Ok(None);
                }
                Err(e) => {
                    error!(target: "mcp::stream", error = %e, "Failed to read line from stream");
                    return Err(McpError::Io(e.to_string()));
                }
            }
        }
    }

    async fn send_response(&self, response: &JsonRpcResponse) -> Result<(), McpError> {
        let mut json_str =
            serde_json::to_string(response).map_err(|e| McpError::Serialization(e.to_string()))?;
        json_str.push('\n');
        trace!(target: "mcp::stream", direction = "out", payload = %json_str.trim_end());

        let mut writer = self.writer.lock().await;
        writer
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        Ok(())
    }

    async fn send_notification(&self, notification: &JsonRpcNotification) -> Result<(), McpError> {
        let mut json_str = serde_json::to_string(notification)
            .map_err(|e| McpError::Serialization(e.to_string()))?;
        json_str.push('\n');
        trace!(target: "mcp::stream", direction = "out", payload = %json_str.trim_end());

        let mut writer = self.writer.lock().await;
        writer
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        Ok(())
    }
}
