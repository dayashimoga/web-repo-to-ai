//! # MCP Error Taxonomy
//!
//! Strongly-typed error domain for the Model Context Protocol (MCP) server,
//! providing mapping to standard JSON-RPC 2.0 error codes and RFC-compliant envelopes.

use thiserror::Error;

/// Standard JSON-RPC 2.0 error codes.
pub mod jsonrpc_codes {
    /// Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text.
    pub const PARSE_ERROR: i32 = -32700;
    /// The JSON sent is not a valid Request object.
    pub const INVALID_REQUEST: i32 = -32600;
    /// The method does not exist / is not available.
    pub const METHOD_NOT_FOUND: i32 = -32601;
    /// Invalid method parameter(s).
    pub const INVALID_PARAMS: i32 = -32602;
    /// Internal JSON-RPC error.
    pub const INTERNAL_ERROR: i32 = -32603;

    /// Server error range start.
    pub const SERVER_ERROR_START: i32 = -32000;
    /// Protocol state error (e.g. request received prior to initialization).
    pub const PROTOCOL_STATE_ERROR: i32 = -32001;
    /// Tool execution failed.
    pub const TOOL_EXECUTION_ERROR: i32 = -32002;
    /// Resource resolution or retrieval failed.
    pub const RESOURCE_ERROR: i32 = -32003;
    /// Transport error (e.g. broken pipe or I/O failure).
    pub const TRANSPORT_ERROR: i32 = -32004;
}

/// Comprehensive error type for MCP server operations.
#[derive(Debug, Error)]
pub enum McpError {
    /// Invalid JSON was received by the server.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// The JSON sent is not a valid Request object.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// The method does not exist or is not available.
    #[error("Method not found: {0}")]
    MethodNotFound(String),

    /// Invalid method parameter(s).
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Internal JSON-RPC error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Request attempted before server initialization or after shutdown.
    #[error("Protocol state violation: {0}")]
    ProtocolStateViolation(String),

    /// Tool invocation or execution failed.
    #[error("Tool execution failed: {0}")]
    ToolError(String),

    /// Resource lookup or retrieval failed.
    #[error("Resource error: {0}")]
    ResourceError(String),

    /// Transport communication or stream error.
    #[error("Transport error: {0}")]
    Transport(String),

    /// Low-level I/O error.
    #[error("I/O error: {0}")]
    Io(String),

    /// JSON serialization or deserialization failure.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

impl McpError {
    /// Returns the corresponding JSON-RPC 2.0 error code.
    #[must_use]
    pub fn error_code(&self) -> i32 {
        match self {
            Self::ParseError(_) => jsonrpc_codes::PARSE_ERROR,
            Self::InvalidRequest(_) => jsonrpc_codes::INVALID_REQUEST,
            Self::MethodNotFound(_) => jsonrpc_codes::METHOD_NOT_FOUND,
            Self::InvalidParams(_) => jsonrpc_codes::INVALID_PARAMS,
            Self::Internal(_) => jsonrpc_codes::INTERNAL_ERROR,
            Self::ProtocolStateViolation(_) => jsonrpc_codes::PROTOCOL_STATE_ERROR,
            Self::ToolError(_) => jsonrpc_codes::TOOL_EXECUTION_ERROR,
            Self::ResourceError(_) => jsonrpc_codes::RESOURCE_ERROR,
            Self::Transport(_) => jsonrpc_codes::TRANSPORT_ERROR,
            Self::Io(_) => jsonrpc_codes::INTERNAL_ERROR,
            Self::Serialization(_) => jsonrpc_codes::PARSE_ERROR,
        }
    }
}

impl From<serde_json::Error> for McpError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for McpError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}
