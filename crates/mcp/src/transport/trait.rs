//! # Transport Layer Abstraction
//!
//! Defines the `Transport` trait for exchanging JSON-RPC 2.0 messages
//! over stdio, in-memory channels, or network streams.

use async_trait::async_trait;

use crate::error::McpError;
use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcResponse};

/// Asynchronous transport interface for the Model Context Protocol.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Reads the next incoming JSON-RPC message from the input stream.
    ///
    /// Returns:
    /// - `Ok(Some(message))` when a valid message is received.
    /// - `Ok(None)` when the transport stream reaches end-of-file (EOF) or disconnects.
    /// - `Err(McpError)` if an I/O or protocol decoding error occurs.
    async fn read_message(&mut self) -> Result<Option<JsonRpcMessage>, McpError>;

    /// Sends a JSON-RPC response message to the client.
    async fn send_response(&self, response: &JsonRpcResponse) -> Result<(), McpError>;

    /// Sends a JSON-RPC notification message to the client.
    async fn send_notification(&self, notification: &JsonRpcNotification) -> Result<(), McpError>;
}
