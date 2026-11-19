//! # Model Context Protocol (MCP) Data Types
//!
//! Aligned with the official Model Context Protocol Specification (version 2024-11-05),
//! defining types for initialization, capability negotiation, tool definitions,
//! resource descriptors, and structured content responses.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::protocol::jsonrpc::RequestId;

/// Supported Model Context Protocol versions.
pub const LATEST_PROTOCOL_VERSION: &str = "2024-11-05";
/// Recognized protocol versions accepted during initialization negotiation.
pub const SUPPORTED_PROTOCOL_VERSIONS: &[&str] = &["2024-11-05"];

/// Metadata describing a client connecting to the MCP server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientInfo {
    /// Client implementation name (e.g. "Claude Desktop", "Antigravity").
    pub name: String,
    /// Client software version.
    pub version: String,
}

/// Metadata describing the MCP server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfo {
    /// Server implementation name.
    pub name: String,
    /// Server software version.
    pub version: String,
}

/// Server capabilities declared during initialization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    /// Experimental capabilities dictionary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    /// Logging capability declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<LoggingCapability>,
    /// Prompts capability declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<PromptsCapability>,
    /// Resources capability declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourcesCapability>,
    /// Tools capability declaration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolsCapability>,
}

/// Declaration of server tools support.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCapability {
    /// Whether the server emits `notifications/tools/list_changed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Declaration of server resources support.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcesCapability {
    /// Whether the server supports resource subscriptions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    /// Whether the server emits `notifications/resources/list_changed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Declaration of server prompts support.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsCapability {
    /// Whether the server emits `notifications/prompts/list_changed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Declaration of server logging support.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingCapability {}

/// Client capabilities declared during initialization.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    /// Experimental capabilities dictionary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    /// Roots capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootsCapability>,
    /// Sampling capability.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<SamplingCapability>,
}

/// Roots capability in client declaration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RootsCapability {
    /// Whether client emits list_changed notifications.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

/// Sampling capability in client declaration.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingCapability {}

/// Parameters sent in the `initialize` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    /// Requested protocol version.
    pub protocol_version: String,
    /// Client declared capabilities.
    pub capabilities: ClientCapabilities,
    /// Information identifying the client.
    pub client_info: ClientInfo,
}

/// Response returned to the `initialize` request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    /// Negotiated protocol version.
    pub protocol_version: String,
    /// Server declared capabilities.
    pub capabilities: ServerCapabilities,
    /// Information identifying the server.
    pub server_info: ServerInfo,
    /// Optional server instruction text for AI model guidance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

/// Declaration of an available Tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Unique name of the tool.
    pub name: String,
    /// Human-readable description of what the tool does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema object describing valid arguments.
    pub input_schema: Value,
}

impl Tool {
    /// Creates a new tool descriptor.
    #[must_use]
    pub fn new(name: impl Into<String>, description: Option<String>, input_schema: Value) -> Self {
        Self {
            name: name.into(),
            description,
            input_schema,
        }
    }
}

/// Result returned from `tools/list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    /// Available tools list.
    pub tools: Vec<Tool>,
    /// Pagination cursor if more tools exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters sent in `tools/call`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolParams {
    /// Name of tool to execute.
    pub name: String,
    /// Tool arguments object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
}

/// Content item inside a tool call response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ToolContent {
    /// Plaintext content.
    Text {
        /// Text payload.
        text: String,
    },
    /// Base64-encoded image content.
    Image {
        /// Base64 data string.
        data: String,
        /// Media MIME type.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Embedded resource content.
    Resource {
        /// Resource contents.
        resource: ResourceContents,
    },
}

impl ToolContent {
    /// Creates a plaintext content item.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }
}

/// Result returned from `tools/call`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    /// Array of content items returned by the tool.
    pub content: Vec<ToolContent>,
    /// True if tool execution produced an error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    /// Creates a successful tool result containing plain text.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(text)],
            is_error: None,
        }
    }

    /// Creates an error tool result.
    #[must_use]
    pub fn error(error_message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::text(error_message)],
            is_error: Some(true),
        }
    }
}

/// Declaration of an available Resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// Canonical URI identifying the resource.
    pub uri: String,
    /// Human-readable name.
    pub name: String,
    /// Optional resource description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Result returned from `resources/list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    /// Available resources list.
    pub resources: Vec<Resource>,
    /// Pagination cursor if more resources exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Parameters sent in `resources/read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceParams {
    /// Target resource URI.
    pub uri: String,
}

/// Content payload of a resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    /// Canonical resource URI.
    pub uri: String,
    /// Content MIME type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// UTF-8 text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64 binary content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// Result returned from `resources/read`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResult {
    /// Array of resource content fragments.
    pub contents: Vec<ResourceContents>,
}

/// Declaration of a prompt argument.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptArgument {
    /// Argument name.
    pub name: String,
    /// Description of argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether argument is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
}

/// Declaration of an available Prompt template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    /// Unique prompt name.
    pub name: String,
    /// Description of prompt behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Declared arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Vec<PromptArgument>>,
}

/// Result returned from `prompts/list`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    /// Available prompts list.
    pub prompts: Vec<Prompt>,
    /// Pagination cursor if more prompts exist.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<String>,
}

/// Severity level for protocol logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingLevel {
    /// Debug-level diagnostic messages.
    Debug,
    /// Informational notices.
    Info,
    /// Normal but significant conditions.
    Notice,
    /// Warning conditions.
    Warning,
    /// Error conditions.
    Error,
    /// Critical conditions.
    Critical,
    /// Action must be taken immediately.
    Alert,
    /// System is unusable.
    Emergency,
}

/// Parameters sent in `logging/setLevel`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLevelParams {
    /// Target logging level threshold.
    pub level: LoggingLevel,
}

/// Parameters in `notifications/message` (logging notification).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoggingMessageParams {
    /// Severity level.
    pub level: LoggingLevel,
    /// Optional logger name or subsystem tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logger: Option<String>,
    /// Log payload data.
    pub data: Value,
}

/// Parameters in `notifications/cancelled`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelledNotificationParams {
    /// ID of the request being cancelled.
    pub request_id: RequestId,
    /// Optional reason for cancellation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}
