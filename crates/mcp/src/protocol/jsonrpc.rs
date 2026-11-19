//! # JSON-RPC 2.0 Protocol Types
//!
//! Implementation of the JSON-RPC 2.0 specification for the Model Context Protocol,
//! supporting string and integer IDs, structured requests, responses, notifications, and errors.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::fmt;

use crate::error::McpError;

/// JSON-RPC 2.0 protocol version string.
pub const JSONRPC_VERSION: &str = "2.0";

/// JSON-RPC 2.0 Request ID, supporting integer or string identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RequestId {
    /// Integer request identifier.
    Number(i64),
    /// String request identifier.
    String(String),
}

impl fmt::Display for RequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(n) => write!(f, "{n}"),
            Self::String(s) => write!(f, "{s}"),
        }
    }
}

impl Serialize for RequestId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Number(n) => serializer.serialize_i64(*n),
            Self::String(s) => serializer.serialize_str(s),
        }
    }
}

impl<'de> Deserialize<'de> for RequestId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RequestIdVisitor;

        impl serde::de::Visitor<'_> for RequestIdVisitor {
            type Value = RequestId;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a string or an integer request id")
            }

            fn visit_i64<E>(self, value: i64) -> Result<RequestId, E>
            where
                E: serde::de::Error,
            {
                Ok(RequestId::Number(value))
            }

            fn visit_u64<E>(self, value: u64) -> Result<RequestId, E>
            where
                E: serde::de::Error,
            {
                if let Ok(signed) = i64::try_from(value) {
                    Ok(RequestId::Number(signed))
                } else {
                    Err(E::custom("integer request id out of i64 range"))
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<RequestId, E>
            where
                E: serde::de::Error,
            {
                Ok(RequestId::String(value.to_owned()))
            }

            fn visit_string<E>(self, value: String) -> Result<RequestId, E>
            where
                E: serde::de::Error,
            {
                Ok(RequestId::String(value))
            }
        }

        deserializer.deserialize_any(RequestIdVisitor)
    }
}

/// JSON-RPC 2.0 Error object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i32,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional diagnostic data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Creates a new JSON-RPC error.
    #[must_use]
    pub fn new(code: i32, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            code,
            message: message.into(),
            data,
        }
    }

    /// Creates a JSON-RPC error from an `McpError`.
    #[must_use]
    pub fn from_mcp_error(err: &McpError, data: Option<Value>) -> Self {
        Self {
            code: err.error_code(),
            message: err.to_string(),
            data,
        }
    }
}

/// JSON-RPC 2.0 Request object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Request identifier (optional for notifications, required for calls).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RequestId>,
    /// Target RPC method.
    pub method: String,
    /// Optional parameter payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    /// Creates a new request with an ID.
    #[must_use]
    pub fn new(id: RequestId, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id: Some(id),
            method: method.into(),
            params,
        }
    }

    /// Returns true if this request is a notification (no ID).
    #[must_use]
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 Response object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Request identifier matching the incoming request.
    pub id: RequestId,
    /// Result payload (mutually exclusive with error).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error payload (mutually exclusive with result).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Constructs a successful JSON-RPC response.
    #[must_use]
    pub fn success(id: RequestId, result: Value) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Constructs an error JSON-RPC response.
    #[must_use]
    pub fn error(id: RequestId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            id,
            result: None,
            error: Some(error),
        }
    }
}

/// JSON-RPC 2.0 Notification object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    /// JSON-RPC protocol version (must be "2.0").
    pub jsonrpc: String,
    /// Notification method.
    pub method: String,
    /// Optional parameter payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    /// Constructs a new notification.
    #[must_use]
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: JSONRPC_VERSION.to_owned(),
            method: method.into(),
            params,
        }
    }
}

/// Unified JSON-RPC message envelope representing any incoming or outgoing message.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    /// Standard request expecting a response.
    Request(JsonRpcRequest),
    /// Response matching a previous request.
    Response(JsonRpcResponse),
    /// One-way notification without response.
    Notification(JsonRpcNotification),
}

impl JsonRpcMessage {
    /// Parses a JSON-RPC message from a JSON string.
    pub fn parse(s: &str) -> Result<Self, McpError> {
        let val: Value =
            serde_json::from_str(s).map_err(|e| McpError::ParseError(e.to_string()))?;
        if !val.is_object() {
            return Err(McpError::InvalidRequest(
                "Message must be a JSON object".to_string(),
            ));
        }

        let obj = val
            .as_object()
            .ok_or_else(|| McpError::InvalidRequest("Message must be a JSON object".to_string()))?;

        // Verify jsonrpc == "2.0"
        match obj.get("jsonrpc").and_then(Value::as_str) {
            Some(v) if v == JSONRPC_VERSION => {}
            Some(other) => {
                return Err(McpError::InvalidRequest(format!(
                    "Unsupported jsonrpc version: '{other}'. Expected '{JSONRPC_VERSION}'"
                )))
            }
            None => {
                return Err(McpError::InvalidRequest(
                    "Missing 'jsonrpc' version field in message".to_string(),
                ))
            }
        }

        // Determine variant
        if obj.contains_key("method") {
            if obj.contains_key("id") {
                let req: JsonRpcRequest = serde_json::from_value(val)?;
                Ok(Self::Request(req))
            } else {
                let notif: JsonRpcNotification = serde_json::from_value(val)?;
                Ok(Self::Notification(notif))
            }
        } else if obj.contains_key("id")
            && (obj.contains_key("result") || obj.contains_key("error"))
        {
            let res: JsonRpcResponse = serde_json::from_value(val)?;
            Ok(Self::Response(res))
        } else {
            Err(McpError::InvalidRequest(
                "Message must contain either 'method' or 'result'/'error' with 'id'".to_string(),
            ))
        }
    }
}
