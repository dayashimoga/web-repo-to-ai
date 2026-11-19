//! # MCP Server & Server Builder
//!
//! Provides the primary `McpServer` runtime orchestrating state, tool registries,
//! and transport dispatch loops, alongside a fluent `McpServerBuilder`.

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use crate::error::McpError;
use crate::protocol::jsonrpc::JsonRpcMessage;
use crate::protocol::types::{Prompt, Resource, ServerInfo, Tool};
use crate::resources::McpDocumentStore;
use crate::server::dispatcher::McpDispatcher;
use crate::server::registry::{
    DynamicResourceProvider, PromptHandler, ResourceHandler, ServerRegistry, ToolHandler,
};
use crate::server::state::ServerState;
use crate::transport::stdio::StdioTransport;
use crate::transport::trait_def::Transport;

/// Asynchronous Model Context Protocol (MCP) server instance.
pub struct McpServer {
    state: Arc<RwLock<ServerState>>,
    registry: Arc<RwLock<ServerRegistry>>,
    dispatcher: McpDispatcher,
    document_store: Arc<McpDocumentStore>,
}

impl McpServer {
    /// Creates a server builder.
    #[must_use]
    pub fn builder() -> McpServerBuilder {
        McpServerBuilder::new()
    }

    /// Returns a read-only handle to the server's runtime state.
    #[must_use]
    pub fn state(&self) -> Arc<RwLock<ServerState>> {
        self.state.clone()
    }

    /// Returns a handle to the server's tool and resource registry.
    #[must_use]
    pub fn registry(&self) -> Arc<RwLock<ServerRegistry>> {
        self.registry.clone()
    }

    /// Returns a handle to the session's in-memory document store.
    #[must_use]
    pub fn document_store(&self) -> Arc<McpDocumentStore> {
        self.document_store.clone()
    }

    /// Runs the server event loop using the provided transport implementation.
    ///
    /// Loops continuously reading messages from the transport until EOF or stream termination.
    pub async fn run<T: Transport>(self, mut transport: T) -> Result<(), McpError> {
        info!(target: "mcp::server", "Starting MCP server message processing loop");

        loop {
            match transport.read_message().await {
                Ok(Some(JsonRpcMessage::Request(req))) => {
                    if let Some(resp) = self.dispatcher.dispatch_request(req).await {
                        transport.send_response(&resp).await?;
                    }
                }
                Ok(Some(JsonRpcMessage::Notification(notif))) => {
                    self.dispatcher.dispatch_notification(notif).await;
                }
                Ok(Some(JsonRpcMessage::Response(res))) => {
                    debug!(target: "mcp::server", id = %res.id, "Ignoring unsolicited response message");
                }
                Ok(None) => {
                    info!(target: "mcp::server", "Transport stream reached end of file; terminating server loop");
                    break;
                }
                Err(e) => {
                    error!(target: "mcp::server", error = %e, "Transport decode error in event loop");
                    // On fatal I/O errors, break loop
                    if matches!(e, McpError::Io(_)) {
                        return Err(e);
                    }
                }
            }
        }

        let mut state = self.state.write().await;
        state.transition_to_terminated();
        info!(target: "mcp::server", "MCP server shutdown cleanly");
        Ok(())
    }

    /// Runs the server using standard I/O (stdio) transport, configuring tracing to `stderr`.
    pub async fn run_stdio(self) -> Result<(), McpError> {
        StdioTransport::init_stdio_tracing();
        let transport = StdioTransport::new();
        self.run(transport).await
    }
}

/// Fluent builder for assembling and configuring an `McpServer`.
pub struct McpServerBuilder {
    name: String,
    version: String,
    instructions: Option<String>,
    registry: ServerRegistry,
    document_store: Arc<McpDocumentStore>,
}

impl Default for McpServerBuilder {
    fn default() -> Self {
        Self {
            name: "web-repo-to-ai-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            instructions: Some(
                "Universal URL & GitHub Repository to AI-Ready Context Converter MCP Server"
                    .to_string(),
            ),
            registry: ServerRegistry::new(),
            document_store: Arc::new(McpDocumentStore::new()),
        }
    }
}

impl McpServerBuilder {
    /// Creates a new builder with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the server implementation name.
    #[must_use]
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the server implementation version.
    #[must_use]
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Sets optional instructions for AI clients describing server capabilities.
    #[must_use]
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Attaches an existing in-memory document store.
    #[must_use]
    pub fn with_document_store(mut self, store: Arc<McpDocumentStore>) -> Self {
        self.document_store = store;
        self
    }

    /// Automatically registers all default conversion tools (`convert_url`, `convert_repo`, `detect_source`, `optimize_context`).
    #[must_use]
    pub fn with_default_tools(mut self) -> Self {
        crate::tools::register_default_tools_with_store(
            &mut self.registry,
            Some(self.document_store.clone()),
        );
        self
    }

    /// Automatically registers all default resource providers (`cdm://`, `file://`, `artifact://`).
    #[must_use]
    pub fn with_default_resources(mut self) -> Self {
        crate::resources::register_default_resources(
            &mut self.registry,
            self.document_store.clone(),
        );
        self
    }

    /// Automatically registers both default conversion tools and resource providers.
    #[must_use]
    pub fn with_defaults(self) -> Self {
        self.with_default_tools().with_default_resources()
    }

    /// Registers a tool with its execution handler.
    #[must_use]
    pub fn register_tool(mut self, tool: Tool, handler: Arc<dyn ToolHandler>) -> Self {
        self.registry.register_tool(tool, handler);
        self
    }

    /// Registers a static resource with its content reading handler.
    #[must_use]
    pub fn register_resource(
        mut self,
        resource: Resource,
        handler: Arc<dyn ResourceHandler>,
    ) -> Self {
        self.registry.register_resource(resource, handler);
        self
    }

    /// Registers a dynamic resource provider.
    #[must_use]
    pub fn register_resource_provider(
        mut self,
        provider: Arc<dyn DynamicResourceProvider>,
    ) -> Self {
        self.registry.register_resource_provider(provider);
        self
    }

    /// Registers a prompt template with its evaluation handler.
    #[must_use]
    pub fn register_prompt(mut self, prompt: Prompt, handler: Arc<dyn PromptHandler>) -> Self {
        self.registry.register_prompt(prompt, handler);
        self
    }

    /// Finalizes construction into an executable `McpServer`.
    #[must_use]
    pub fn build(self) -> McpServer {
        let server_info = ServerInfo {
            name: self.name,
            version: self.version,
        };

        let state = Arc::new(RwLock::new(ServerState::new()));
        let registry = Arc::new(RwLock::new(self.registry));
        let dispatcher = McpDispatcher::new(
            state.clone(),
            registry.clone(),
            server_info,
            self.instructions,
        );

        McpServer {
            state,
            registry,
            dispatcher,
            document_store: self.document_store,
        }
    }
}
