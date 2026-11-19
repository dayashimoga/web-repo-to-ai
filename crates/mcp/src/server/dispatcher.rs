//! # MCP Request Dispatcher
//!
//! Routes JSON-RPC 2.0 requests to protocol lifecycle handlers, tool executions,
//! resource readers, and template providers with full error mapping.

use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::jsonrpc_codes;
use crate::protocol::jsonrpc::{
    JsonRpcError, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use crate::protocol::types::{
    CallToolParams, InitializeParams, InitializeResult, ListPromptsResult, ListResourcesResult,
    ListToolsResult, LoggingCapability, PromptsCapability, ReadResourceParams, ResourcesCapability,
    ServerCapabilities, ServerInfo, SetLevelParams, ToolsCapability, LATEST_PROTOCOL_VERSION,
};
use crate::server::registry::ServerRegistry;
use crate::server::state::{ServerLifecycleState, ServerState};

/// Core dispatcher routing incoming JSON-RPC calls against server state and registries.
pub struct McpDispatcher {
    state: Arc<RwLock<ServerState>>,
    registry: Arc<RwLock<ServerRegistry>>,
    server_info: ServerInfo,
    instructions: Option<String>,
}

impl McpDispatcher {
    /// Creates a new dispatcher.
    #[must_use]
    pub fn new(
        state: Arc<RwLock<ServerState>>,
        registry: Arc<RwLock<ServerRegistry>>,
        server_info: ServerInfo,
        instructions: Option<String>,
    ) -> Self {
        Self {
            state,
            registry,
            server_info,
            instructions,
        }
    }

    /// Dispatches an incoming request, returning an optional response.
    pub async fn dispatch_request(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let req_id = match req.id.clone() {
            Some(id) => id,
            None => {
                // If it's a notification without an ID
                self.dispatch_notification(JsonRpcNotification::new(req.method, req.params))
                    .await;
                return None;
            }
        };

        let method = req.method.as_str();
        debug!(target: "mcp::dispatch", method = %method, id = %req_id, "Dispatching MCP request");

        let response = match method {
            "initialize" => self.handle_initialize(req_id, req.params).await,
            "ping" => self.handle_ping(req_id).await,
            "tools/list" => self.handle_tools_list(req_id).await,
            "tools/call" => self.handle_tools_call(req_id, req.params).await,
            "resources/list" => self.handle_resources_list(req_id).await,
            "resources/read" => self.handle_resources_read(req_id, req.params).await,
            "prompts/list" => self.handle_prompts_list(req_id).await,
            "logging/setLevel" => self.handle_logging_set_level(req_id, req.params).await,
            unknown => {
                warn!(target: "mcp::dispatch", method = %unknown, id = %req_id, "Unknown MCP method requested");
                JsonRpcResponse::error(
                    req_id,
                    JsonRpcError::new(
                        jsonrpc_codes::METHOD_NOT_FOUND,
                        format!("Method not found: '{unknown}'"),
                        None,
                    ),
                )
            }
        };

        Some(response)
    }

    /// Dispatches a notification (fire-and-forget message).
    pub async fn dispatch_notification(&self, notif: JsonRpcNotification) {
        let method = notif.method.as_str();
        debug!(target: "mcp::dispatch", method = %method, "Processing MCP notification");

        match method {
            "notifications/initialized" => {
                let mut state = self.state.write().await;
                if let Err(e) = state.transition_to_initialized() {
                    warn!(target: "mcp::dispatch", error = %e, "Invalid transition on notifications/initialized");
                } else {
                    info!(target: "mcp::dispatch", "MCP connection successfully initialized and ready");
                }
            }
            "notifications/cancelled" => {
                info!(target: "mcp::dispatch", params = ?notif.params, "Client cancelled request");
            }
            unknown => {
                debug!(target: "mcp::dispatch", method = %unknown, "Ignoring unrecognized notification");
            }
        }
    }

    async fn handle_initialize(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let params_val = match params {
            Some(v) => v,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        "Missing params in 'initialize' request",
                        None,
                    ),
                );
            }
        };

        let init_params: InitializeParams = match serde_json::from_value(params_val) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        format!("Failed to parse initialize parameters: {e}"),
                        None,
                    ),
                );
            }
        };

        let mut state = self.state.write().await;
        if state.lifecycle != ServerLifecycleState::Uninitialized {
            return JsonRpcResponse::error(
                id,
                JsonRpcError::new(
                    jsonrpc_codes::PROTOCOL_STATE_ERROR,
                    format!(
                        "Server already initialized (current state: '{}')",
                        state.lifecycle
                    ),
                    None,
                ),
            );
        }

        // Accept client's requested version or default to latest
        let negotiated_version = LATEST_PROTOCOL_VERSION.to_string();
        if let Err(e) = state.transition_to_initializing(
            negotiated_version.clone(),
            init_params.client_info,
            init_params.capabilities,
        ) {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let capabilities = ServerCapabilities {
            experimental: None,
            logging: Some(LoggingCapability {}),
            prompts: Some(PromptsCapability {
                list_changed: Some(false),
            }),
            resources: Some(ResourcesCapability {
                subscribe: Some(false),
                list_changed: Some(false),
            }),
            tools: Some(ToolsCapability {
                list_changed: Some(false),
            }),
        };

        let result = InitializeResult {
            protocol_version: negotiated_version,
            capabilities,
            server_info: self.server_info.clone(),
            instructions: self.instructions.clone(),
        };

        match serde_json::to_value(result) {
            Ok(val) => JsonRpcResponse::success(id, val),
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
            ),
        }
    }

    async fn handle_ping(&self, id: RequestId) -> JsonRpcResponse {
        JsonRpcResponse::success(id, json!({}))
    }

    async fn handle_tools_list(&self, id: RequestId) -> JsonRpcResponse {
        let state = self.state.read().await;
        if let Err(e) = state.ensure_initialized() {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let registry = self.registry.read().await;
        let tools = registry.list_tools();
        let result = ListToolsResult {
            tools,
            next_cursor: None,
        };

        match serde_json::to_value(result) {
            Ok(val) => JsonRpcResponse::success(id, val),
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
            ),
        }
    }

    async fn handle_tools_call(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let state = self.state.read().await;
        if let Err(e) = state.ensure_initialized() {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let params_val = match params {
            Some(v) => v,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        "Missing parameters in 'tools/call'",
                        None,
                    ),
                );
            }
        };

        let call_params: CallToolParams = match serde_json::from_value(params_val) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        format!("Failed to parse tools/call parameters: {e}"),
                        None,
                    ),
                );
            }
        };

        let registry = self.registry.read().await;
        let (_, handler) = match registry.get_tool(&call_params.name) {
            Some(t) => t,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::METHOD_NOT_FOUND,
                        format!("Unknown tool: '{}'", call_params.name),
                        None,
                    ),
                );
            }
        };

        match handler.call(call_params.arguments).await {
            Ok(tool_result) => match serde_json::to_value(tool_result) {
                Ok(val) => JsonRpcResponse::success(id, val),
                Err(e) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
                ),
            },
            Err(e) => {
                error!(target: "mcp::dispatch", tool = %call_params.name, error = %e, "Tool execution returned error");
                JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None))
            }
        }
    }

    async fn handle_resources_list(&self, id: RequestId) -> JsonRpcResponse {
        let state = self.state.read().await;
        if let Err(e) = state.ensure_initialized() {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let registry = self.registry.read().await;
        let resources = registry.list_resources();
        let result = ListResourcesResult {
            resources,
            next_cursor: None,
        };

        match serde_json::to_value(result) {
            Ok(val) => JsonRpcResponse::success(id, val),
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
            ),
        }
    }

    async fn handle_resources_read(&self, id: RequestId, params: Option<Value>) -> JsonRpcResponse {
        let state = self.state.read().await;
        if let Err(e) = state.ensure_initialized() {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let params_val = match params {
            Some(v) => v,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        "Missing parameters in 'resources/read'",
                        None,
                    ),
                );
            }
        };

        let read_params: ReadResourceParams = match serde_json::from_value(params_val) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        format!("Failed to parse resources/read parameters: {e}"),
                        None,
                    ),
                );
            }
        };

        let registry = self.registry.read().await;
        let (_, handler) = match registry.get_resource(&read_params.uri) {
            Some(r) => r,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::RESOURCE_ERROR,
                        format!("Resource not found: '{}'", read_params.uri),
                        None,
                    ),
                );
            }
        };

        match handler.read(&read_params.uri).await {
            Ok(res) => match serde_json::to_value(res) {
                Ok(val) => JsonRpcResponse::success(id, val),
                Err(e) => JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
                ),
            },
            Err(e) => JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None)),
        }
    }

    async fn handle_prompts_list(&self, id: RequestId) -> JsonRpcResponse {
        let state = self.state.read().await;
        if let Err(e) = state.ensure_initialized() {
            return JsonRpcResponse::error(id, JsonRpcError::from_mcp_error(&e, None));
        }

        let registry = self.registry.read().await;
        let prompts = registry.list_prompts();
        let result = ListPromptsResult {
            prompts,
            next_cursor: None,
        };

        match serde_json::to_value(result) {
            Ok(val) => JsonRpcResponse::success(id, val),
            Err(e) => JsonRpcResponse::error(
                id,
                JsonRpcError::new(jsonrpc_codes::INTERNAL_ERROR, e.to_string(), None),
            ),
        }
    }

    async fn handle_logging_set_level(
        &self,
        id: RequestId,
        params: Option<Value>,
    ) -> JsonRpcResponse {
        let params_val = match params {
            Some(v) => v,
            None => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        "Missing parameters in 'logging/setLevel'",
                        None,
                    ),
                );
            }
        };

        let level_params: SetLevelParams = match serde_json::from_value(params_val) {
            Ok(p) => p,
            Err(e) => {
                return JsonRpcResponse::error(
                    id,
                    JsonRpcError::new(
                        jsonrpc_codes::INVALID_PARAMS,
                        format!("Failed to parse logging/setLevel parameters: {e}"),
                        None,
                    ),
                );
            }
        };

        let mut state = self.state.write().await;
        state.log_level = level_params.level;
        debug!(target: "mcp::dispatch", level = ?level_params.level, "Updated logging level threshold");
        JsonRpcResponse::success(id, json!({}))
    }
}
