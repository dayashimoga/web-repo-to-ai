//! # MCP Server Lifecycle & State Management
//!
//! Enforces the state machine prescribed by the Model Context Protocol:
//! `Uninitialized` -> `Initializing` -> `Initialized` -> `ShuttingDown` -> `Terminated`.

use std::fmt;

use crate::error::McpError;
use crate::protocol::types::{ClientCapabilities, ClientInfo, LoggingLevel};

/// Lifecycle phases of an active MCP server connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerLifecycleState {
    /// Initial state upon connection before client `initialize` request.
    #[default]
    Uninitialized,
    /// `initialize` request has been answered, awaiting `notifications/initialized`.
    Initializing,
    /// Connection is fully established; tools, resources, and prompts are accessible.
    Initialized,
    /// Server is undergoing graceful shutdown.
    ShuttingDown,
    /// Connection terminated.
    Terminated,
}

impl fmt::Display for ServerLifecycleState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uninitialized => write!(f, "Uninitialized"),
            Self::Initializing => write!(f, "Initializing"),
            Self::Initialized => write!(f, "Initialized"),
            Self::ShuttingDown => write!(f, "ShuttingDown"),
            Self::Terminated => write!(f, "Terminated"),
        }
    }
}

/// Runtime state maintained across an active MCP session.
#[derive(Debug, Clone)]
pub struct ServerState {
    /// Current lifecycle phase.
    pub lifecycle: ServerLifecycleState,
    /// Negotiated protocol version.
    pub protocol_version: Option<String>,
    /// Information about the connected client.
    pub client_info: Option<ClientInfo>,
    /// Capabilities declared by the client.
    pub client_capabilities: Option<ClientCapabilities>,
    /// Active logging severity threshold.
    pub log_level: LoggingLevel,
}

impl Default for ServerState {
    fn default() -> Self {
        Self {
            lifecycle: ServerLifecycleState::Uninitialized,
            protocol_version: None,
            client_info: None,
            client_capabilities: None,
            log_level: LoggingLevel::Info,
        }
    }
}

impl ServerState {
    /// Constructs a new clean server state.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Verifies that the server has reached the `Initialized` state, returning an error if not.
    pub fn ensure_initialized(&self) -> Result<(), McpError> {
        if self.lifecycle == ServerLifecycleState::Initialized {
            Ok(())
        } else {
            Err(McpError::ProtocolStateViolation(format!(
                "Server is in '{}' state; method requires 'Initialized' state",
                self.lifecycle
            )))
        }
    }

    /// Transitions to `Initializing` upon valid `initialize` request.
    pub fn transition_to_initializing(
        &mut self,
        version: String,
        client_info: ClientInfo,
        client_capabilities: ClientCapabilities,
    ) -> Result<(), McpError> {
        if self.lifecycle != ServerLifecycleState::Uninitialized {
            return Err(McpError::ProtocolStateViolation(format!(
                "Cannot initialize server: current state is '{}'",
                self.lifecycle
            )));
        }

        self.lifecycle = ServerLifecycleState::Initializing;
        self.protocol_version = Some(version);
        self.client_info = Some(client_info);
        self.client_capabilities = Some(client_capabilities);
        Ok(())
    }

    /// Transitions to `Initialized` upon receiving `notifications/initialized`.
    pub fn transition_to_initialized(&mut self) -> Result<(), McpError> {
        if self.lifecycle != ServerLifecycleState::Initializing {
            return Err(McpError::ProtocolStateViolation(format!(
                "Received 'notifications/initialized' while in '{}' state",
                self.lifecycle
            )));
        }

        self.lifecycle = ServerLifecycleState::Initialized;
        Ok(())
    }

    /// Transitions to `ShuttingDown`.
    pub fn transition_to_shutdown(&mut self) {
        self.lifecycle = ServerLifecycleState::ShuttingDown;
    }

    /// Transitions to `Terminated`.
    pub fn transition_to_terminated(&mut self) {
        self.lifecycle = ServerLifecycleState::Terminated;
    }
}
