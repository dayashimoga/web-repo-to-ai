//! # MCP Protocol Layer
//!
//! JSON-RPC 2.0 primitives and official Model Context Protocol (MCP) data structures.

pub mod jsonrpc;
pub mod types;

pub use jsonrpc::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
    JSONRPC_VERSION,
};
pub use types::*;
