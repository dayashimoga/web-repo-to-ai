//! # In-Memory Channel Transport
//!
//! Provides bidirectional, thread-safe message exchange using asynchronous Tokio MPSC channels,
//! enabling fast, deterministic in-memory integration testing and embedded orchestration.

use async_trait::async_trait;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::Mutex;

use crate::error::McpError;
use crate::protocol::jsonrpc::{JsonRpcMessage, JsonRpcNotification, JsonRpcResponse};
use crate::transport::trait_def::Transport;

/// In-memory transport powered by Tokio channels.
pub struct ChannelTransport {
    incoming_rx: Mutex<Receiver<JsonRpcMessage>>,
    outgoing_tx: Sender<JsonRpcMessage>,
}

impl ChannelTransport {
    /// Creates a new channel transport from receiver and sender channels.
    #[must_use]
    pub fn new(incoming_rx: Receiver<JsonRpcMessage>, outgoing_tx: Sender<JsonRpcMessage>) -> Self {
        Self {
            incoming_rx: Mutex::new(incoming_rx),
            outgoing_tx,
        }
    }

    /// Creates a paired client/server channel transport pair with the specified buffer capacity.
    ///
    /// Returns `(server_transport, client_transport)`.
    #[must_use]
    pub fn pair(capacity: usize) -> (Self, Self) {
        let (server_tx, client_rx) = channel(capacity);
        let (client_tx, server_rx) = channel(capacity);

        let server_transport = Self::new(server_rx, server_tx);
        let client_transport = Self::new(client_rx, client_tx);

        (server_transport, client_transport)
    }

    /// Directly writes a message into the transport's outgoing channel.
    pub async fn send_raw(&self, message: JsonRpcMessage) -> Result<(), McpError> {
        self.outgoing_tx.send(message).await.map_err(|_| {
            McpError::Transport("Failed to send message: receiver dropped".to_string())
        })
    }
}

#[async_trait]
impl Transport for ChannelTransport {
    async fn read_message(&mut self) -> Result<Option<JsonRpcMessage>, McpError> {
        let mut rx = self.incoming_rx.lock().await;
        Ok(rx.recv().await)
    }

    async fn send_response(&self, response: &JsonRpcResponse) -> Result<(), McpError> {
        self.send_raw(JsonRpcMessage::Response(response.clone()))
            .await
    }

    async fn send_notification(&self, notification: &JsonRpcNotification) -> Result<(), McpError> {
        self.send_raw(JsonRpcMessage::Notification(notification.clone()))
            .await
    }
}
