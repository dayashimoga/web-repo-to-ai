//! # Standard I/O (stdio) Transport
//!
//! Production-grade Model Context Protocol (MCP) transport communicating over `stdin` and `stdout`.
//! Adheres strictly to the JSON-RPC newline-delimited framing contract, reserving `stdout` exclusively
//! for protocol messages and directing all diagnostic logging and traces to `stderr`.

use async_trait::async_trait;

use crate::error::McpError;
use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcResponse};
use crate::transport::stream::StreamTransport;
use crate::transport::trait_def::Transport;

/// Standard I/O transport for MCP desktop AI clients (Claude Desktop, Antigravity, Cursor).
pub struct StdioTransport {
    inner: StreamTransport<tokio::io::Stdin, tokio::io::Stdout>,
}

impl StdioTransport {
    /// Constructs a new `StdioTransport` bound to standard input and standard output.
    #[must_use]
    pub fn new() -> Self {
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        Self {
            inner: StreamTransport::new(stdin, stdout),
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
        self.inner.read_message().await
    }

    async fn send_response(&self, response: &JsonRpcResponse) -> Result<(), McpError> {
        self.inner.send_response(response).await
    }

    async fn send_notification(&self, notification: &JsonRpcNotification) -> Result<(), McpError> {
        self.inner.send_notification(notification).await
    }
}
