//! # MCP Server Lifecycle State Machine Tests

use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use web_repo_to_ai_mcp::error::jsonrpc_codes;
use web_repo_to_ai_mcp::protocol::jsonrpc::{JsonRpcNotification, JsonRpcRequest, RequestId};
use web_repo_to_ai_mcp::protocol::types::{
    ClientCapabilities, ClientInfo, InitializeParams, InitializeResult, ServerInfo,
    LATEST_PROTOCOL_VERSION,
};
use web_repo_to_ai_mcp::server::dispatcher::McpDispatcher;
use web_repo_to_ai_mcp::server::registry::ServerRegistry;
use web_repo_to_ai_mcp::server::state::{ServerLifecycleState, ServerState};

fn setup_dispatcher() -> (McpDispatcher, Arc<RwLock<ServerState>>) {
    let state = Arc::new(RwLock::new(ServerState::new()));
    let registry = Arc::new(RwLock::new(ServerRegistry::new()));
    let server_info = ServerInfo {
        name: "test-mcp-server".to_string(),
        version: "0.1.0".to_string(),
    };
    let dispatcher = McpDispatcher::new(
        state.clone(),
        registry,
        server_info,
        Some("Test server guidance".to_string()),
    );
    (dispatcher, state)
}

#[tokio::test]
async fn test_uninitialized_state_rejects_tools_list() {
    let (dispatcher, state) = setup_dispatcher();
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Uninitialized
    );

    let req = JsonRpcRequest::new(RequestId::Number(1), "tools/list", None);
    let resp = dispatcher
        .dispatch_request(req)
        .await
        .expect("response expected");

    assert_eq!(resp.id, RequestId::Number(1));
    assert!(resp.error.is_some());
    let err = resp.error.unwrap();
    assert_eq!(err.code, jsonrpc_codes::PROTOCOL_STATE_ERROR);
    assert!(err.message.contains("requires 'Initialized' state"));
}

#[tokio::test]
async fn test_ping_succeeds_in_uninitialized_state() {
    let (dispatcher, _) = setup_dispatcher();

    let req = JsonRpcRequest::new(RequestId::Number(99), "ping", None);
    let resp = dispatcher
        .dispatch_request(req)
        .await
        .expect("response expected");

    assert_eq!(resp.id, RequestId::Number(99));
    assert!(resp.error.is_none());
    assert_eq!(resp.result, Some(json!({})));
}

#[tokio::test]
async fn test_full_initialization_lifecycle_handshake() {
    let (dispatcher, state) = setup_dispatcher();

    // Step 1: Send 'initialize'
    let init_params = InitializeParams {
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "TestClient".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    let init_req = JsonRpcRequest::new(
        RequestId::Number(1),
        "initialize",
        Some(serde_json::to_value(init_params).unwrap()),
    );

    let init_resp = dispatcher
        .dispatch_request(init_req)
        .await
        .expect("response expected");
    assert_eq!(init_resp.id, RequestId::Number(1));
    assert!(init_resp.error.is_none());

    let result: InitializeResult =
        serde_json::from_value(init_resp.result.unwrap()).expect("parse InitializeResult");
    assert_eq!(result.protocol_version, LATEST_PROTOCOL_VERSION);
    assert_eq!(result.server_info.name, "test-mcp-server");
    assert_eq!(
        result.instructions,
        Some("Test server guidance".to_string())
    );
    assert!(result.capabilities.tools.is_some());
    assert!(result.capabilities.resources.is_some());

    // State is now Initializing
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Initializing
    );

    // Step 2: Attempting repeated initialize while Initializing must be rejected
    let second_init_req = JsonRpcRequest::new(
        RequestId::Number(2),
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "c", "version": "1"}
        })),
    );
    let second_resp = dispatcher.dispatch_request(second_init_req).await.unwrap();
    assert_eq!(
        second_resp.error.unwrap().code,
        jsonrpc_codes::PROTOCOL_STATE_ERROR
    );

    // Step 3: Client sends 'notifications/initialized'
    let notif = JsonRpcNotification::new("notifications/initialized", Some(json!({})));
    dispatcher.dispatch_notification(notif).await;

    // State is now Initialized
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Initialized
    );

    // Step 4: Queries now succeed
    let tools_req = JsonRpcRequest::new(RequestId::Number(3), "tools/list", None);
    let tools_resp = dispatcher.dispatch_request(tools_req).await.unwrap();
    assert!(tools_resp.error.is_none());
    assert_eq!(tools_resp.result.unwrap()["tools"], json!([]));

    let res_req = JsonRpcRequest::new(RequestId::Number(4), "resources/list", None);
    let res_resp = dispatcher.dispatch_request(res_req).await.unwrap();
    assert!(res_resp.error.is_none());
    assert_eq!(res_resp.result.unwrap()["resources"], json!([]));

    let prompt_req = JsonRpcRequest::new(RequestId::Number(5), "prompts/list", None);
    let prompt_resp = dispatcher.dispatch_request(prompt_req).await.unwrap();
    assert!(prompt_resp.error.is_none());
    assert_eq!(prompt_resp.result.unwrap()["prompts"], json!([]));
}

#[tokio::test]
async fn test_logging_set_level() {
    let (dispatcher, state) = setup_dispatcher();

    let req = JsonRpcRequest::new(
        RequestId::Number(10),
        "logging/setLevel",
        Some(json!({ "level": "debug" })),
    );

    let resp = dispatcher.dispatch_request(req).await.expect("response");
    assert!(resp.error.is_none());

    let active_level = state.read().await.log_level;
    assert_eq!(
        active_level,
        web_repo_to_ai_mcp::protocol::types::LoggingLevel::Debug
    );
}
