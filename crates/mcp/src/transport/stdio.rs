//! # Standard I/O (stdio) Transport
//!
//! Production-grade Model Context Protocol (MCP) transport communicating over `stdin` and `stdout`.
//! Adheres strictly to the JSON-RPC newline-delimited framing contract, reserving `stdout` exclusively
//! for protocol messages and directing all diagnostic logging and traces to `stderr`.

use async_trait::async_trait;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin, Stdout};
use tokio::sync::Mutex;
use tracing::{debug, error, trace};

use crate::error::McpError;
use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcResponse};
use crate::transport::trait_def::Transport;

/// Standard I/O transport for MCP desktop AI clients (Claude Desktop, Antigravity, Cursor).
pub struct StdioTransport {
    reader: Mutex<tokio::io::Lines<BufReader<Stdin>>>,
    writer: Mutex<Stdout>,
}

impl StdioTransport {
    /// Constructs a new `StdioTransport` bound to standard input and standard output.
    #[must_use]
    pub fn new() -> Self {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        let reader = BufReader::new(stdin).lines();
        Self {
            reader: Mutex::new(reader),
            writer: Mutex::new(stdout),
        }
    }

    /// Initializes tracing logger directing all events strictly to standard error (`stderr`).
    ///
    /// This prevents any pollution of the JSON-RPC standard output stream.
    pub fn init_stdio_tracing() {
        let subscriber = tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
            )
            .finish();

        let _ = tracing::subscriber::set_global_default(subscriber);
    }
}

impl Default for StdioTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for StdioTransport {
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

                    trace!(target: "mcp::stdio", direction = "in", payload = %trimmed);
                    match JsonRpcMessage::parse(trimmed) {
                        Ok(msg) => return Ok(Some(msg)),
                        Err(e) => {
                            error!(target: "mcp::stdio", error = %e, raw = %trimmed, "Malformed incoming JSON-RPC message");
                            return Err(e);
                        }
                    }
                }
                Ok(None) => {
                    debug!(target: "mcp::stdio", "Standard input reached EOF");
                    return Ok(None);
                }
                Err(e) => {
                    error!(target: "mcp::stdio", error = %e, "Failed to read line from stdin");
                    return Err(McpError::Io(e.to_string()));
                }
            }
        }
    }

    async fn send_response(&self, response: &JsonRpcResponse) -> Result<(), McpError> {
        let json_str =
            serde_json::to_string(response).map_err(|e| McpError::Serialization(e.to_string()))?;
        trace!(target: "mcp::stdio", direction = "out", payload = %json_str);

        let mut writer = self.writer.lock().await;
        writer
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;

        Ok(())
    }

    async fn send_notification(&self, notification: &JsonRpcNotification) -> Result<(), McpError> {
        let json_str = serde_json::to_string(notification)
            .map_err(|e| McpError::Serialization(e.to_string()))?;
        trace!(target: "mcp::stdio", direction = "out", payload = %json_str);

        let mut writer = self.writer.lock().await;
        writer
            .write_all(json_str.as_bytes())
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .write_all(b"\n")
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;
        writer
            .flush()
            .await
            .map_err(|e| McpError::Io(e.to_string()))?;

        Ok(())
    }
}
