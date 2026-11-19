//! # web-repo-to-ai-mcp
//!
//! Model Context Protocol (MCP) server implementation for `web-repo-to-ai`,
//! exposing universal webpage and GitHub repository conversion capabilities to AI agents
//! over standard JSON-RPC 2.0 stdio and transport abstractions.
//!
//! # Architecture
//!
//! - **`protocol`**: Strongly-typed JSON-RPC 2.0 framing and official MCP (2024-11-05) data types.
//! - **`transport`**: Abstract asynchronous transports including production `StdioTransport` and testing `ChannelTransport`.
//! - **`server`**: Lifecycle state machine, tool/resource registries, and high-performance async request dispatcher.

pub mod error;
pub mod protocol;
pub mod server;
pub mod transport;

pub use error::{jsonrpc_codes, McpError};
pub use protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, ListPromptsResult, ListResourcesResult, ListToolsResult, LoggingLevel, Prompt,
    PromptArgument, ReadResourceParams, ReadResourceResult, RequestId, Resource, ResourceContents,
    ServerCapabilities, ServerInfo, Tool, ToolContent, JSONRPC_VERSION, LATEST_PROTOCOL_VERSION,
};
pub use server::{
    FnResourceHandler, FnToolHandler, McpDispatcher, McpServer, McpServerBuilder, PromptHandler,
    ResourceHandler, ServerLifecycleState, ServerRegistry, ServerState, ToolHandler,
};
pub use transport::{ChannelTransport, StdioTransport, Transport};
