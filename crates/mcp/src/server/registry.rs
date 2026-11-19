//! # MCP Tool and Resource Registry
//!
//! Provides thread-safe registries for MCP tools, resources, and prompt templates,
//! supporting extensible dynamic handlers and typed parameter schemas.

use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::McpError;
use crate::protocol::types::{CallToolResult, Prompt, ReadResourceResult, Resource, Tool};

/// Trait implemented by executable MCP tool handlers.
#[async_trait]
pub trait ToolHandler: Send + Sync {
    /// Executes the tool with the provided arguments.
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError>;
}

/// Closure-backed tool handler.
pub struct FnToolHandler<F> {
    func: F,
}

impl<F> FnToolHandler<F> {
    /// Wraps an async closure as a `ToolHandler`.
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F, Fut> ToolHandler for FnToolHandler<F>
where
    F: Fn(Option<Value>) -> Fut + Send + Sync,
    Fut: Future<Output = Result<CallToolResult, McpError>> + Send,
{
    async fn call(&self, arguments: Option<Value>) -> Result<CallToolResult, McpError> {
        (self.func)(arguments).await
    }
}

/// Trait implemented by MCP resource content providers.
#[async_trait]
pub trait ResourceHandler: Send + Sync {
    /// Reads and returns the content for the specified canonical URI.
    async fn read(&self, uri: &str) -> Result<ReadResourceResult, McpError>;
}

/// Closure-backed resource handler.
pub struct FnResourceHandler<F> {
    func: F,
}

impl<F> FnResourceHandler<F> {
    /// Wraps an async closure as a `ResourceHandler`.
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

#[async_trait]
impl<F, Fut> ResourceHandler for FnResourceHandler<F>
where
    F: Fn(String) -> Fut + Send + Sync,
    Fut: Future<Output = Result<ReadResourceResult, McpError>> + Send,
{
    async fn read(&self, uri: &str) -> Result<ReadResourceResult, McpError> {
        (self.func)(uri.to_owned()).await
    }
}

/// Trait implemented by prompt template providers.
#[async_trait]
pub trait PromptHandler: Send + Sync {
    /// Evaluates prompt arguments and returns rendered prompt payload.
    async fn get(&self, arguments: Option<Value>) -> Result<Value, McpError>;
}

/// Registry storing all registered tools, resources, and prompt definitions.
#[derive(Default)]
pub struct ServerRegistry {
    tools: HashMap<String, (Tool, Arc<dyn ToolHandler>)>,
    resources: HashMap<String, (Resource, Arc<dyn ResourceHandler>)>,
    prompts: HashMap<String, (Prompt, Arc<dyn PromptHandler>)>,
}

impl ServerRegistry {
    /// Creates an empty server registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a tool with its execution handler.
    pub fn register_tool(&mut self, tool: Tool, handler: Arc<dyn ToolHandler>) {
        self.tools.insert(tool.name.clone(), (tool, handler));
    }

    /// Registers a resource descriptor with its reading handler.
    pub fn register_resource(&mut self, resource: Resource, handler: Arc<dyn ResourceHandler>) {
        self.resources
            .insert(resource.uri.clone(), (resource, handler));
    }

    /// Registers a prompt template with its evaluation handler.
    pub fn register_prompt(&mut self, prompt: Prompt, handler: Arc<dyn PromptHandler>) {
        self.prompts.insert(prompt.name.clone(), (prompt, handler));
    }

    /// Returns list of registered tool descriptors.
    #[must_use]
    pub fn list_tools(&self) -> Vec<Tool> {
        self.tools.values().map(|(t, _)| t.clone()).collect()
    }

    /// Retrieves tool handler by name.
    #[must_use]
    pub fn get_tool(&self, name: &str) -> Option<(Tool, Arc<dyn ToolHandler>)> {
        self.tools.get(name).cloned()
    }

    /// Returns list of registered resource descriptors.
    #[must_use]
    pub fn list_resources(&self) -> Vec<Resource> {
        self.resources.values().map(|(r, _)| r.clone()).collect()
    }

    /// Retrieves resource handler by URI.
    #[must_use]
    pub fn get_resource(&self, uri: &str) -> Option<(Resource, Arc<dyn ResourceHandler>)> {
        self.resources.get(uri).cloned()
    }

    /// Returns list of registered prompt descriptors.
    #[must_use]
    pub fn list_prompts(&self) -> Vec<Prompt> {
        self.prompts.values().map(|(p, _)| p.clone()).collect()
    }

    /// Retrieves prompt handler by name.
    #[must_use]
    pub fn get_prompt(&self, name: &str) -> Option<(Prompt, Arc<dyn PromptHandler>)> {
        self.prompts.get(name).cloned()
    }
}
