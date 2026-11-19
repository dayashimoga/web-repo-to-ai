//! # MCP Server End-to-End Dispatch Integration Tests

use serde_json::json;
use std::sync::Arc;
use web_repo_to_ai_mcp::error::{jsonrpc_codes, McpError};
use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, RequestId,
};
use web_repo_to_ai_mcp::protocol::types::{
    CallToolResult, InitializeResult, ListResourcesResult, ListToolsResult, Prompt,
    ReadResourceResult, Resource, ResourceContents, Tool, LATEST_PROTOCOL_VERSION,
};
use web_repo_to_ai_mcp::server::registry::{FnResourceHandler, FnToolHandler, PromptHandler};
use web_repo_to_ai_mcp::transport::channel::ChannelTransport;
use web_repo_to_ai_mcp::transport::trait_def::Transport;
use web_repo_to_ai_mcp::McpServer;

struct DummyPromptHandler;

#[async_trait::async_trait]
impl PromptHandler for DummyPromptHandler {
    async fn get(
        &self,
        _arguments: Option<serde_json::Value>,
    ) -> Result<serde_json::Value, McpError> {
        Ok(json!({"prompt": "Hello prompt"}))
    }
}

#[tokio::test]
async fn test_end_to_end_server_dispatch_lifecycle() {
    let (server_transport, mut client_transport) = ChannelTransport::pair(32);

    // Define test tool
    let echo_tool = Tool::new(
        "echo_tool",
        Some("Echoes input text back".to_string()),
        json!({
            "type": "object",
            "properties": {
                "message": { "type": "string" }
            },
            "required": ["message"]
        }),
    );
    let echo_handler = Arc::new(FnToolHandler::new(
        |args: Option<serde_json::Value>| async move {
            let msg = args
                .and_then(|a| a.get("message").and_then(|m| m.as_str().map(String::from)))
                .unwrap_or_else(|| "default echo".to_string());
            Ok(CallToolResult::text(format!("echo: {msg}")))
        },
    ));

    // Define test resource
    let readme_resource = Resource {
        uri: "repo://readme".to_string(),
        name: "README".to_string(),
        description: Some("Project README document".to_string()),
        mime_type: Some("text/markdown".to_string()),
    };
    let readme_handler = Arc::new(FnResourceHandler::new(|uri| async move {
        Ok(ReadResourceResult {
            contents: vec![ResourceContents {
                uri,
                mime_type: Some("text/markdown".to_string()),
                text: Some("# Test Project README\n\nContent here.".to_string()),
                blob: None,
            }],
        })
    }));

    // Define test prompt
    let test_prompt = Prompt {
        name: "test_prompt".to_string(),
        description: Some("A test prompt".to_string()),
        arguments: None,
    };

    // Build McpServer
    let server = McpServer::builder()
        .name("integration-test-server")
        .version("0.1.0")
        .instructions("Test guidance")
        .register_tool(echo_tool, echo_handler)
        .register_resource(readme_resource, readme_handler)
        .register_prompt(test_prompt, Arc::new(DummyPromptHandler))
        .build();

    // Spawn server in background
    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    // 1. Client sends 'initialize'
    let init_req = JsonRpcRequest::new(
        RequestId::Number(1),
        "initialize",
        Some(json!({
            "protocolVersion": LATEST_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "IntegrationClient",
                "version": "1.0"
            }
        })),
    );
    client_transport
        .send_raw(JsonRpcMessage::Request(init_req))
        .await
        .unwrap();

    let init_msg = client_transport.read_message().await.unwrap().unwrap();
    match init_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(1));
            assert!(res.error.is_none());
            let result: InitializeResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.server_info.name, "integration-test-server");
        }
        _ => panic!("Expected Response for initialize"),
    }

    // 2. Client sends 'notifications/initialized'
    let init_notif = JsonRpcNotification::new("notifications/initialized", Some(json!({})));
    client_transport
        .send_raw(JsonRpcMessage::Notification(init_notif))
        .await
        .unwrap();

    // 3. Client calls 'tools/list'
    let tools_req = JsonRpcRequest::new(RequestId::Number(2), "tools/list", None);
    client_transport
        .send_raw(JsonRpcMessage::Request(tools_req))
        .await
        .unwrap();

    let tools_msg = client_transport.read_message().await.unwrap().unwrap();
    match tools_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());
            let result: ListToolsResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.tools.len(), 1);
            assert_eq!(result.tools[0].name, "echo_tool");
        }
        _ => panic!("Expected Response for tools/list"),
    }

    // 4. Client calls 'tools/call' on echo_tool
    let call_req = JsonRpcRequest::new(
        RequestId::Number(3),
        "tools/call",
        Some(json!({
            "name": "echo_tool",
            "arguments": {
                "message": "hello world"
            }
        })),
    );
    client_transport
        .send_raw(JsonRpcMessage::Request(call_req))
        .await
        .unwrap();

    let call_msg = client_transport.read_message().await.unwrap().unwrap();
    match call_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            assert!(res.error.is_none());
            let result: CallToolResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.content.len(), 1);
            match &result.content[0] {
                web_repo_to_ai_mcp::protocol::types::ToolContent::Text { text } => {
                    assert_eq!(text, "echo: hello world");
                }
                _ => panic!("Expected text content"),
            }
        }
        _ => panic!("Expected Response for tools/call"),
    }

    // 5. Client calls 'resources/list'
    let res_req = JsonRpcRequest::new(RequestId::Number(4), "resources/list", None);
    client_transport
        .send_raw(JsonRpcMessage::Request(res_req))
        .await
        .unwrap();

    let res_msg = client_transport.read_message().await.unwrap().unwrap();
    match res_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(4));
            assert!(res.error.is_none());
            let result: ListResourcesResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.resources.len(), 1);
            assert_eq!(result.resources[0].uri, "repo://readme");
        }
        _ => panic!("Expected Response for resources/list"),
    }

    // 6. Client calls 'resources/read'
    let read_req = JsonRpcRequest::new(
        RequestId::Number(5),
        "resources/read",
        Some(json!({
            "uri": "repo://readme"
        })),
    );
    client_transport
        .send_raw(JsonRpcMessage::Request(read_req))
        .await
        .unwrap();

    let read_msg = client_transport.read_message().await.unwrap().unwrap();
    match read_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(5));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.contents.len(), 1);
            assert_eq!(
                result.contents[0].text,
                Some("# Test Project README\n\nContent here.".to_string())
            );
        }
        _ => panic!("Expected Response for resources/read"),
    }

    // 7. Client calls unknown tool
    let unknown_req = JsonRpcRequest::new(
        RequestId::Number(6),
        "tools/call",
        Some(json!({
            "name": "nonexistent_tool"
        })),
    );
    client_transport
        .send_raw(JsonRpcMessage::Request(unknown_req))
        .await
        .unwrap();

    let unknown_msg = client_transport.read_message().await.unwrap().unwrap();
    match unknown_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(6));
            assert!(res.error.is_some());
            assert_eq!(res.error.unwrap().code, jsonrpc_codes::METHOD_NOT_FOUND);
        }
        _ => panic!("Expected Response for unknown tool"),
    }

    // 8. Terminate connection cleanly by dropping client
    drop(client_transport);

    let server_result = server_handle.await.expect("join handle succeeds");
    assert!(
        server_result.is_ok(),
        "Server loop should exit cleanly on EOF"
    );
}
