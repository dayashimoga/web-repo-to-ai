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
//! - **`tools`**: Universal conversion tools (`convert_url`, `convert_repo`, `detect_source`, `optimize_context`).
//! - **`resources`**: Resource providers for converted documents (`cdm://`), packages, manifests, and local artifacts (`file://`, `artifact://`).

pub mod error;
pub mod protocol;
pub mod resources;
pub mod server;
pub mod tools;
pub mod transport;

pub use error::{jsonrpc_codes, McpError};
pub use protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest,
    JsonRpcResponse, ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
    ListToolsResult, LoggingLevel, Prompt, PromptArgument, ReadResourceParams, ReadResourceResult,
    RequestId, Resource, ResourceContents, ResourceTemplate, ServerCapabilities, ServerInfo, Tool,
    ToolContent, JSONRPC_VERSION, LATEST_PROTOCOL_VERSION,
};
pub use resources::{
    decode_base64, encode_base64, register_default_resources, CdmResourceProvider, DocumentSummary,
    FileResourceProvider, McpDocumentStore,
};
pub use server::{
    DynamicResourceProvider, FnResourceHandler, FnToolHandler, McpDispatcher, McpServer,
    McpServerBuilder, PromptHandler, ResourceHandler, ServerLifecycleState, ServerRegistry,
    ServerState, ToolHandler,
};
pub use tools::{
    detect_source_tool, export_cdm_to_string, optimize_context_tool, register_default_tools,
    register_default_tools_with_store, repo_tool, url_tool, ConvertRepoHandler, ConvertRepoParams,
    ConvertUrlHandler, ConvertUrlParams, DetectSourceHandler, DetectSourceParams,
    OptimizeContextHandler, OptimizeContextParams,
};
pub use transport::{ChannelTransport, StdioTransport, Transport};
