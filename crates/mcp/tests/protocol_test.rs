//! # MCP Protocol & JSON-RPC 2.0 Unit & Integration Tests

use serde_json::json;
use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcError, JsonRpcMessage, JsonRpcResponse, RequestId, JSONRPC_VERSION,
};
use web_repo_to_ai_mcp::protocol::types::{
    CallToolResult, ClientCapabilities, ClientInfo, InitializeParams, InitializeResult,
    ListResourcesResult, ListToolsResult, ReadResourceResult, Resource, ResourceContents,
    ServerCapabilities, ServerInfo, Tool, LATEST_PROTOCOL_VERSION,
};

#[test]
fn test_request_id_serialization_and_deserialization() {
    let num_id = RequestId::Number(42);
    let str_id = RequestId::String("req-abc-123".to_string());

    let num_json = serde_json::to_string(&num_id).expect("serialize num id");
    let str_json = serde_json::to_string(&str_id).expect("serialize str id");

    assert_eq!(num_json, "42");
    assert_eq!(str_json, "\"req-abc-123\"");

    let deserialized_num: RequestId = serde_json::from_str("42").expect("deserialize num");
    let deserialized_str: RequestId =
        serde_json::from_str("\"req-abc-123\"").expect("deserialize str");

    assert_eq!(deserialized_num, num_id);
    assert_eq!(deserialized_str, str_id);
    assert_eq!(deserialized_num.to_string(), "42");
    assert_eq!(deserialized_str.to_string(), "req-abc-123");
}

#[test]
fn test_jsonrpc_request_parsing() {
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"ping","params":{"foo":"bar"}}"#;
    let msg = JsonRpcMessage::parse(raw).expect("parse valid request");

    match msg {
        JsonRpcMessage::Request(req) => {
            assert_eq!(req.jsonrpc, JSONRPC_VERSION);
            assert_eq!(req.id, Some(RequestId::Number(1)));
            assert_eq!(req.method, "ping");
            assert_eq!(req.params, Some(json!({"foo": "bar"})));
            assert!(!req.is_notification());
        }
        _ => panic!("Expected Request variant"),
    }
}

#[test]
fn test_jsonrpc_notification_parsing() {
    let raw = r#"{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}"#;
    let msg = JsonRpcMessage::parse(raw).expect("parse valid notification");

    match msg {
        JsonRpcMessage::Notification(notif) => {
            assert_eq!(notif.jsonrpc, JSONRPC_VERSION);
            assert_eq!(notif.method, "notifications/initialized");
            assert_eq!(notif.params, Some(json!({})));
        }
        _ => panic!("Expected Notification variant"),
    }
}

#[test]
fn test_jsonrpc_response_parsing() {
    let raw = r#"{"jsonrpc":"2.0","id":"abc","result":{"status":"ok"}}"#;
    let msg = JsonRpcMessage::parse(raw).expect("parse valid response");

    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.jsonrpc, JSONRPC_VERSION);
            assert_eq!(res.id, RequestId::String("abc".to_string()));
            assert_eq!(res.result, Some(json!({"status": "ok"})));
            assert!(res.error.is_none());
        }
        _ => panic!("Expected Response variant"),
    }
}

#[test]
fn test_jsonrpc_error_response_serialization() {
    let err = JsonRpcError::new(-32601, "Method not found", Some(json!({"method": "foo"})));
    let response = JsonRpcResponse::error(RequestId::Number(10), err);

    let serialized = serde_json::to_value(&response).expect("serialize error response");
    assert_eq!(serialized["jsonrpc"], "2.0");
    assert_eq!(serialized["id"], 10);
    assert_eq!(serialized["error"]["code"], -32601);
    assert_eq!(serialized["error"]["message"], "Method not found");
    assert_eq!(serialized["error"]["data"]["method"], "foo");
    assert!(serialized.get("result").is_none());
}

#[test]
fn test_jsonrpc_parse_rejections() {
    // Malformed JSON
    assert!(JsonRpcMessage::parse("not json").is_err());

    // Non-object JSON
    assert!(JsonRpcMessage::parse("[1, 2, 3]").is_err());

    // Missing jsonrpc
    assert!(JsonRpcMessage::parse(r#"{"id":1,"method":"ping"}"#).is_err());

    // Wrong jsonrpc version
    assert!(JsonRpcMessage::parse(r#"{"jsonrpc":"1.0","id":1,"method":"ping"}"#).is_err());

    // Missing method and result
    assert!(JsonRpcMessage::parse(r#"{"jsonrpc":"2.0","id":1}"#).is_err());
}

#[test]
fn test_mcp_initialize_params_roundtrip() {
    let params = InitializeParams {
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "TestClient".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let serialized = serde_json::to_value(&params).expect("serialize initialize params");
    assert_eq!(serialized["protocolVersion"], "2024-11-05");
    assert_eq!(serialized["clientInfo"]["name"], "TestClient");
    assert_eq!(serialized["clientInfo"]["version"], "1.0.0");

    let deserialized: InitializeParams =
        serde_json::from_value(serialized).expect("deserialize initialize params");
    assert_eq!(deserialized, params);
}

#[test]
fn test_mcp_initialize_result_roundtrip() {
    let result = InitializeResult {
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities::default(),
        server_info: ServerInfo {
            name: "web-repo-to-ai-mcp".to_string(),
            version: "0.1.0".to_string(),
        },
        instructions: Some("AI Guidance Instructions".to_string()),
    };

    let serialized = serde_json::to_value(&result).expect("serialize initialize result");
    assert_eq!(serialized["protocolVersion"], "2024-11-05");
    assert_eq!(serialized["serverInfo"]["name"], "web-repo-to-ai-mcp");
    assert_eq!(serialized["instructions"], "AI Guidance Instructions");

    let deserialized: InitializeResult =
        serde_json::from_value(serialized).expect("deserialize initialize result");
    assert_eq!(deserialized, result);
}

#[test]
fn test_tool_definitions_and_results() {
    let tool = Tool::new(
        "convert_url",
        Some("Converts a single webpage to clean context".to_string()),
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string" }
            },
            "required": ["url"]
        }),
    );

    let tools_result = ListToolsResult {
        tools: vec![tool.clone()],
        next_cursor: None,
    };

    let val = serde_json::to_value(&tools_result).expect("serialize tools result");
    assert_eq!(val["tools"][0]["name"], "convert_url");
    assert_eq!(
        val["tools"][0]["description"],
        "Converts a single webpage to clean context"
    );

    let success_result = CallToolResult::text("Converted markdown output");
    let err_result = CallToolResult::error("Fetch failed");

    let success_val = serde_json::to_value(&success_result).expect("serialize tool success");
    assert_eq!(success_val["content"][0]["type"], "text");
    assert_eq!(
        success_val["content"][0]["text"],
        "Converted markdown output"
    );
    assert!(success_val.get("isError").is_none());

    let err_val = serde_json::to_value(&err_result).expect("serialize tool error");
    assert_eq!(err_val["isError"], true);
    assert_eq!(err_val["content"][0]["text"], "Fetch failed");
}

#[test]
fn test_resource_definitions_and_results() {
    let resource = Resource {
        uri: "repo://metadata".to_string(),
        name: "Repository Metadata".to_string(),
        description: Some("Contextual metadata for active repository".to_string()),
        mime_type: Some("application/json".to_string()),
    };

    let list_res = ListResourcesResult {
        resources: vec![resource],
        next_cursor: None,
    };

    let list_val = serde_json::to_value(&list_res).expect("serialize resources result");
    assert_eq!(list_val["resources"][0]["uri"], "repo://metadata");
    assert_eq!(list_val["resources"][0]["mimeType"], "application/json");

    let content = ResourceContents {
        uri: "repo://metadata".to_string(),
        mime_type: Some("application/json".to_string()),
        text: Some("{\"files\": 10}".to_string()),
        blob: None,
    };

    let read_result = ReadResourceResult {
        contents: vec![content],
    };

    let read_val = serde_json::to_value(&read_result).expect("serialize read result");
    assert_eq!(read_val["contents"][0]["text"], "{\"files\": 10}");
}
