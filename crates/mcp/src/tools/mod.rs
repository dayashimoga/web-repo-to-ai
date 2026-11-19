//! # Built-in MCP Conversion Tools
//!
//! Production Model Context Protocol tools for webpage conversion (`convert_url`),
//! repository and directory conversion (`convert_repo`), source archetype detection (`detect_source`),
//! and token context optimization (`optimize_context`).

pub mod convert_repo;
pub mod convert_url;
pub mod detect_source;
pub mod export;
pub mod optimize_context;

use std::sync::Arc;

pub use convert_repo::{tool_definition as repo_tool, ConvertRepoHandler, ConvertRepoParams};
pub use convert_url::{tool_definition as url_tool, ConvertUrlHandler, ConvertUrlParams};
pub use detect_source::{
    tool_definition as detect_source_tool, DetectSourceHandler, DetectSourceParams,
};
pub use export::export_cdm_to_string;
pub use optimize_context::{
    tool_definition as optimize_context_tool, OptimizeContextHandler, OptimizeContextParams,
};

use crate::resources::McpDocumentStore;
use crate::server::registry::ServerRegistry;

/// Registers all default production conversion tools backed by an optional document store.
pub fn register_default_tools_with_store(
    registry: &mut ServerRegistry,
    store: Option<Arc<McpDocumentStore>>,
) {
    let mut url_handler = ConvertUrlHandler::new();
    let mut repo_handler = ConvertRepoHandler::new();
    if let Some(ref s) = store {
        url_handler = url_handler.with_store(s.clone());
        repo_handler = repo_handler.with_store(s.clone());
    }

    registry.register_tool(url_tool(), Arc::new(url_handler));
    registry.register_tool(repo_tool(), Arc::new(repo_handler));
    registry.register_tool(detect_source_tool(), Arc::new(DetectSourceHandler::new()));
    registry.register_tool(
        optimize_context_tool(),
        Arc::new(OptimizeContextHandler::new()),
    );
}

/// Registers all default production conversion tools into the provided registry.
pub fn register_default_tools(registry: &mut ServerRegistry) {
    register_default_tools_with_store(registry, None);
}
