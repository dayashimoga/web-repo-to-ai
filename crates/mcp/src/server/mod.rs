//! # MCP Server Subsystem
//!
//! Lifecycle management, tool/resource registry, request dispatching, and server execution.

pub mod dispatcher;
pub mod registry;
pub mod service;
pub mod state;

pub use dispatcher::McpDispatcher;
pub use registry::{
    FnResourceHandler, FnToolHandler, PromptHandler, ResourceHandler, ServerRegistry, ToolHandler,
};
pub use service::{McpServer, McpServerBuilder};
pub use state::{ServerLifecycleState, ServerState};
