//! # MCP Security & Protocol Testing Suite
//!
//! Subphase 9.3.4 Comprehensive Security & Protocol Certification:
//! - Adversarial JSON-RPC 2.0 protocol fuzzing (malformed JSON, invalid envelopes, null ID support)
//! - Server lifecycle state machine enforcement (pre-initialization rejections, repeated handshake defense)
//! - Defense-in-depth resource security & path traversal defenses (`file://`, `artifact://`, `cdm://`)
//! - Conversion tool input validation & SSRF boundary security (`convert_url`, `convert_repo`, `detect_source`, `optimize_context`)
//! - Stream transport framing resilience, error responses, pipelining, and stream isolation
//! - Multi-client concurrent execution & thread safety

use std::fs;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use serde_json::json;

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{CdmDocument, SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_mcp::error::jsonrpc_codes;
use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcError, JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use web_repo_to_ai_mcp::protocol::types::{
    CallToolResult, ClientCapabilities, ClientInfo, InitializeParams, InitializeResult,
    ReadResourceResult, ToolContent, LATEST_PROTOCOL_VERSION,
};
use web_repo_to_ai_mcp::resources::{CdmResourceProvider, FileResourceProvider, McpDocumentStore};
use web_repo_to_ai_mcp::server::dispatcher::McpDispatcher;
use web_repo_to_ai_mcp::server::registry::ServerRegistry;
use web_repo_to_ai_mcp::server::state::{ServerLifecycleState, ServerState};
use web_repo_to_ai_mcp::tools::{
    ConvertRepoHandler, ConvertUrlHandler, DetectSourceHandler, OptimizeContextHandler,
};
use web_repo_to_ai_mcp::transport::channel::ChannelTransport;
use web_repo_to_ai_mcp::transport::stream::StreamTransport;
use web_repo_to_ai_mcp::transport::trait_def::Transport;
use web_repo_to_ai_mcp::{McpError, McpServer};

/// Helper creating a minimal valid CDM test document for resource tests.
fn create_test_cdm_document() -> CdmDocument {
    let source_id = SourceIdentity::new("https://example.com/docs", "https://example.com/docs");
    let mut builder = CdmDocumentBuilder::new(
        source_id,
        SourceType::WebpageStatic,
        "https://example.com/docs",
    );

    let sec1 = CdmSectionBuilder::new(
        "overview",
        "https://example.com/docs#overview",
        SectionKind::Prose,
        "# System Architecture\n\nComprehensive security documentation.",
    )
    .title("System Architecture")
    .original_path("docs/overview.md")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "code_snippet",
        "https://example.com/docs#code",
        SectionKind::CodeFile,
        "pub fn verify_signature(data: &[u8]) -> bool { !data.is_empty() }",
    )
    .title("Security Verification")
    .language("rust")
    .original_path("src/security.rs")
    .build();

    builder = builder.add_section(sec1).add_section(sec2);
    builder.build()
}

/// Helper extracting plaintext string from CallToolResult.
fn get_tool_text(res: &CallToolResult) -> &str {
    match &res.content[0] {
        ToolContent::Text { text } => text.as_str(),
        _ => panic!("Expected ToolContent::Text"),
    }
}

/// Helper initializing a connected client transport over an active McpServer.
async fn setup_initialized_client(server: McpServer) -> ChannelTransport {
    let (server_transport, mut client_transport) = ChannelTransport::pair(64);
    tokio::spawn(async move {
        let _ = server.run(server_transport).await;
    });

    let init_params = InitializeParams {
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
        capabilities: ClientCapabilities::default(),
        client_info: ClientInfo {
            name: "SecurityTestClient".to_string(),
            version: "1.0.0".to_string(),
        },
    };
    let init_req = JsonRpcRequest::new(
        RequestId::Number(1),
        "initialize",
        Some(serde_json::to_value(init_params).unwrap()),
    );
    client_transport
        .send_raw(JsonRpcMessage::Request(init_req))
        .await
        .expect("send initialize");

    let _resp = client_transport.read_message().await.unwrap().unwrap();

    let notif = JsonRpcNotification::new("notifications/initialized", None);
    client_transport
        .send_notification(&notif)
        .await
        .expect("send initialized notification");

    client_transport
}

// ============================================================================
// 1. Adversarial JSON-RPC Protocol Fuzzing
// ============================================================================

#[test]
fn test_adversarial_jsonrpc_protocol_fuzzing() {
    // 1.1 Malformed JSON syntax fuzzing
    let malformed_payloads = [
        "",
        "   ",
        "not json at all",
        "{",
        "}",
        "{\"jsonrpc\": \"2.0\"",
        "{\"jsonrpc\": \"2.0\", \"method\": \"ping\"",
        "{\"jsonrpc\": \"2.0\", \"id\": 1,}",
        "{\"jsonrpc\": \"2.0\", \"id\": 1, \"params\": [}",
        "GET / HTTP/1.1\r\nHost: localhost\r\n\r\n",
        "{\"jsonrpc\": \"2.0\", \"id\": \0}",
        "<xml><jsonrpc>2.0</jsonrpc></xml>",
    ];

    for payload in malformed_payloads {
        let result = JsonRpcMessage::parse(payload);
        assert!(
            result.is_err(),
            "Malformed payload must be rejected: '{payload}'"
        );
        match result.unwrap_err() {
            McpError::ParseError(_) | McpError::InvalidRequest(_) => {}
            other => panic!("Unexpected error type for malformed payload: {other:?}"),
        }
    }

    // 1.2 Non-object JSON primitives
    let primitive_payloads = [
        "42",
        "3.14159",
        "\"a string\"",
        "true",
        "false",
        "null",
        "[1, 2, 3]",
        "[\"ping\", 1]",
    ];

    for payload in primitive_payloads {
        let result = JsonRpcMessage::parse(payload);
        assert!(
            result.is_err(),
            "Primitive/array JSON must be rejected: '{payload}'"
        );
        match result.unwrap_err() {
            McpError::InvalidRequest(msg) => {
                assert!(msg.contains("must be a JSON object"));
            }
            other => panic!("Expected InvalidRequest for primitive payload: {other:?}"),
        }
    }

    // 1.3 Invalid JSON-RPC 2.0 envelopes
    let invalid_envelopes = [
        // Missing jsonrpc field
        (
            r#"{"id": 1, "method": "ping"}"#,
            "Missing 'jsonrpc' version field",
        ),
        // Wrong jsonrpc version 1.0
        (
            r#"{"jsonrpc": "1.0", "id": 1, "method": "ping"}"#,
            "Unsupported jsonrpc version: '1.0'",
        ),
        // Wrong jsonrpc version 3.0
        (
            r#"{"jsonrpc": "3.0", "id": 1, "method": "ping"}"#,
            "Unsupported jsonrpc version: '3.0'",
        ),
        // Method is an integer
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": 123}"#,
            "Method name must be a string",
        ),
        // Method is an array
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": ["ping"]}"#,
            "Method name must be a string",
        ),
        // Method is a boolean
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": true}"#,
            "Method name must be a string",
        ),
        // Params is a number
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": "ping", "params": 42}"#,
            "Params must be an object or array",
        ),
        // Params is a string
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": "ping", "params": "hello"}"#,
            "Params must be an object or array",
        ),
        // Params is a boolean
        (
            r#"{"jsonrpc": "2.0", "id": 1, "method": "ping", "params": false}"#,
            "Params must be an object or array",
        ),
        // Neither method nor result/error with id
        (
            r#"{"jsonrpc": "2.0", "id": 1}"#,
            "Message must contain either 'method' or 'result'/'error'",
        ),
    ];

    for (payload, expected_msg) in invalid_envelopes {
        let result = JsonRpcMessage::parse(payload);
        assert!(result.is_err(), "Envelope must be rejected: '{payload}'");
        match result.unwrap_err() {
            McpError::InvalidRequest(msg) => {
                assert!(
                    msg.contains(expected_msg),
                    "Expected '{expected_msg}' in '{msg}'"
                );
            }
            other => panic!("Expected InvalidRequest for envelope: {other:?}"),
        }
    }

    // 1.4 Request ID null support per JSON-RPC 2.0 Specification
    let null_id_req = r#"{"jsonrpc": "2.0", "id": null, "method": "ping"}"#;
    let parsed = JsonRpcMessage::parse(null_id_req).expect("parse null id request");
    match parsed {
        JsonRpcMessage::Request(req) => {
            assert_eq!(req.id, Some(RequestId::Null));
            assert_eq!(req.method, "ping");
        }
        _ => panic!("Expected Request variant for null id"),
    }

    // Roundtrip RequestId::Null
    let null_id = RequestId::Null;
    assert_eq!(null_id.to_string(), "null");
    let serialized_null = serde_json::to_string(&null_id).expect("serialize null id");
    assert_eq!(serialized_null, "null");

    let deserialized_null: RequestId = serde_json::from_str("null").expect("deserialize null id");
    assert_eq!(deserialized_null, RequestId::Null);

    // Serialization of JsonRpcResponse with RequestId::Null
    let err_resp = JsonRpcResponse::error(
        RequestId::Null,
        JsonRpcError::new(-32700, "Parse error", None),
    );
    let serialized_resp = serde_json::to_string(&err_resp).expect("serialize error response");
    assert!(serialized_resp.contains(r#""id":null"#));
    assert!(serialized_resp.contains(r#""code":-32700"#));
}

// ============================================================================
// 2. Protocol State Machine & Lifecycle Enforcement
// ============================================================================

#[tokio::test]
async fn test_protocol_state_machine_and_lifecycle_enforcement() {
    let state = Arc::new(tokio::sync::RwLock::new(ServerState::new()));
    let registry = Arc::new(tokio::sync::RwLock::new(ServerRegistry::new()));
    let server_info = web_repo_to_ai_mcp::protocol::types::ServerInfo {
        name: "security-lifecycle-server".to_string(),
        version: "0.1.0".to_string(),
    };
    let dispatcher = McpDispatcher::new(
        state.clone(),
        registry,
        server_info,
        Some("Security Guidance".to_string()),
    );

    // 2.1 Server starts strictly in Uninitialized state
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Uninitialized
    );

    // 2.2 Pre-initialization calls to operational RPCs MUST be rejected with PROTOCOL_STATE_ERROR (-32001)
    let pre_init_calls = [
        ("tools/list", None),
        (
            "tools/call",
            Some(json!({"name": "convert_url", "arguments": {}})),
        ),
        ("resources/list", None),
        ("resources/templates/list", None),
        ("resources/read", Some(json!({"uri": "cdm://123/markdown"}))),
        ("prompts/list", None),
    ];

    for (idx, (method, params)) in pre_init_calls.iter().enumerate() {
        let req = JsonRpcRequest::new(RequestId::Number(idx as i64 + 10), *method, params.clone());
        let resp = dispatcher.dispatch_request(req).await.expect("response");
        assert_eq!(resp.id, RequestId::Number(idx as i64 + 10));
        assert!(
            resp.error.is_some(),
            "Method '{method}' must be rejected prior to initialization"
        );
        let err = resp.error.unwrap();
        assert_eq!(
            err.code,
            jsonrpc_codes::PROTOCOL_STATE_ERROR,
            "Method '{method}' must return PROTOCOL_STATE_ERROR"
        );
        assert!(err.message.contains("requires 'Initialized' state"));
    }

    // 2.3 Ping MUST succeed even in Uninitialized state
    let ping_req = JsonRpcRequest::new(RequestId::Number(99), "ping", None);
    let ping_resp = dispatcher
        .dispatch_request(ping_req)
        .await
        .expect("response");
    assert_eq!(ping_resp.id, RequestId::Number(99));
    assert!(ping_resp.error.is_none());
    assert_eq!(ping_resp.result, Some(json!({})));

    // 2.4 Invalid initialize parameters
    // Missing params
    let init_no_params = JsonRpcRequest::new(RequestId::Number(100), "initialize", None);
    let resp = dispatcher
        .dispatch_request(init_no_params)
        .await
        .expect("response");
    assert_eq!(resp.error.unwrap().code, jsonrpc_codes::INVALID_PARAMS);

    // Params as primitive string
    let init_str_params = JsonRpcRequest::new(
        RequestId::Number(101),
        "initialize",
        Some(json!("not an object")),
    );
    let resp = dispatcher
        .dispatch_request(init_str_params)
        .await
        .expect("response");
    assert_eq!(resp.error.unwrap().code, jsonrpc_codes::INVALID_PARAMS);

    // Missing clientInfo
    let init_missing_client = JsonRpcRequest::new(
        RequestId::Number(102),
        "initialize",
        Some(json!({ "protocolVersion": "2024-11-05" })),
    );
    let resp = dispatcher
        .dispatch_request(init_missing_client)
        .await
        .expect("response");
    assert_eq!(resp.error.unwrap().code, jsonrpc_codes::INVALID_PARAMS);

    // 2.5 Valid initialization handshake
    let valid_init = JsonRpcRequest::new(
        RequestId::Number(103),
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "FuzzClient",
                "version": "1.0"
            }
        })),
    );
    let init_resp = dispatcher
        .dispatch_request(valid_init)
        .await
        .expect("response");
    assert!(init_resp.error.is_none());
    let init_result: InitializeResult =
        serde_json::from_value(init_resp.result.unwrap()).expect("parse init result");
    assert_eq!(init_result.protocol_version, "2024-11-05");
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Initializing
    );

    // 2.6 Repeated initialize request MUST be rejected
    let second_init = JsonRpcRequest::new(
        RequestId::Number(104),
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "SecondClient", "version": "1.0" }
        })),
    );
    let second_resp = dispatcher
        .dispatch_request(second_init)
        .await
        .expect("response");
    assert!(second_resp.error.is_some());
    assert_eq!(
        second_resp.error.unwrap().code,
        jsonrpc_codes::PROTOCOL_STATE_ERROR
    );

    // 2.7 Transition to Initialized via notification
    let init_notif = JsonRpcNotification::new("notifications/initialized", None);
    dispatcher.dispatch_notification(init_notif).await;
    assert_eq!(
        state.read().await.lifecycle,
        ServerLifecycleState::Initialized
    );

    // 2.8 Notifications must never elicit responses or errors
    let unknown_notif = JsonRpcNotification::new(
        "notifications/unknown_event_xyz",
        Some(json!({"random": 123})),
    );
    // Notification dispatch completes silently without panic
    dispatcher.dispatch_notification(unknown_notif).await;

    // Unknown request method returns -32601 METHOD_NOT_FOUND
    let unknown_req = JsonRpcRequest::new(RequestId::Number(105), "unknown/rpc_method", None);
    let unknown_resp = dispatcher
        .dispatch_request(unknown_req)
        .await
        .expect("response");
    assert_eq!(
        unknown_resp.error.unwrap().code,
        jsonrpc_codes::METHOD_NOT_FOUND
    );
}

// ============================================================================
// 3. Defense-in-Depth Resource Security & Path Traversal Defenses
// ============================================================================

#[tokio::test]
async fn test_resource_security_and_unauthorized_rejections() {
    let test_dir = std::env::temp_dir().join(format!("mcp-sec-root-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("create test root");

    let sub_dir = test_dir.join("sub");
    fs::create_dir_all(&sub_dir).expect("create sub directory");

    let canary_file = test_dir.join("canary.txt");
    fs::write(&canary_file, b"AUTHORIZED CANARY CONTENT").expect("write canary");

    let sensitive_file =
        std::env::temp_dir().join(format!("mcp-sec-outside-{}", uuid::Uuid::new_v4()));
    fs::write(&sensitive_file, b"TOP SECRET EXTRANEOUS FILE").expect("write sensitive file");
    let sensitive_file_name = sensitive_file.file_name().unwrap().to_str().unwrap();

    let doc_store = Arc::new(McpDocumentStore::new());
    let test_doc = create_test_cdm_document();
    let doc_id = test_doc.document_id.to_string();
    doc_store.insert(test_doc);

    let file_provider = Arc::new(FileResourceProvider::new(&test_dir));
    let cdm_provider = Arc::new(CdmResourceProvider::new(doc_store.clone()));

    let server = McpServer::builder()
        .name("security-resource-server")
        .with_document_store(doc_store)
        .register_resource_provider(file_provider)
        .register_resource_provider(cdm_provider)
        .build();

    let mut client = setup_initialized_client(server).await;

    // 3.1 Reject unauthorized URI schemes
    let unauthorized_uris = [
        "http://evil.com/exploit.txt",
        "https://evil.com/sensitive.json",
        "ftp://ftp.example.com/data",
        "ssh://git@github.com/repo",
        "gopher://gopher.floodgap.com/0/",
        "unknown-scheme://payload",
    ];

    for (idx, uri) in unauthorized_uris.iter().enumerate() {
        let req = JsonRpcRequest::new(
            RequestId::Number(200 + idx as i64),
            "resources/read",
            Some(json!({ "uri": uri })),
        );
        client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();
        let msg = client.read_message().await.unwrap().unwrap();
        match msg {
            JsonRpcMessage::Response(res) => {
                assert!(
                    res.error.is_some(),
                    "Unauthorized URI '{uri}' must be rejected"
                );
                assert_eq!(res.error.unwrap().code, jsonrpc_codes::RESOURCE_ERROR);
            }
            _ => panic!("Expected Response for unauthorized URI read"),
        }
    }

    // 3.2 FileResourceProvider path traversal attacks
    let traversal_uris = [
        // Standard relative traversal
        format!("file://../{sensitive_file_name}"),
        format!("file:///../{sensitive_file_name}"),
        format!("file://../../{sensitive_file_name}"),
        // Artifact scheme traversal
        format!("artifact://../{sensitive_file_name}"),
        format!("artifact:///../../../{sensitive_file_name}"),
        // URL-encoded percent-traversal
        format!("file://%2e%2e%2f{sensitive_file_name}"),
        format!("file://%2e%2e/{sensitive_file_name}"),
        format!("file://..%2f{sensitive_file_name}"),
        format!("file://%2e%2e%2f%2e%2e%2f{sensitive_file_name}"),
        format!("artifact://%2e%2e%2f{sensitive_file_name}"),
        // Null byte injection in path
        format!("file://canary.txt%00/{sensitive_file_name}"),
        // Directory read rejection (not a regular file)
        "file://sub".to_string(),
        "file:///".to_string(),
    ];

    for (idx, uri) in traversal_uris.iter().enumerate() {
        let req = JsonRpcRequest::new(
            RequestId::Number(300 + idx as i64),
            "resources/read",
            Some(json!({ "uri": uri })),
        );
        client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();
        let msg = client.read_message().await.unwrap().unwrap();
        match msg {
            JsonRpcMessage::Response(res) => {
                assert!(
                    res.error.is_some(),
                    "Traversal URI '{uri}' must be rejected with error"
                );
                let err = res.error.unwrap();
                assert_eq!(err.code, jsonrpc_codes::RESOURCE_ERROR);
            }
            _ => panic!("Expected Response for traversal read"),
        }
    }

    // Valid file read succeeds within allowed root
    let valid_file_req = JsonRpcRequest::new(
        RequestId::Number(400),
        "resources/read",
        Some(json!({ "uri": "file://canary.txt" })),
    );
    client
        .send_raw(JsonRpcMessage::Request(valid_file_req))
        .await
        .unwrap();
    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(
                result.contents[0].text.as_deref(),
                Some("AUTHORIZED CANARY CONTENT")
            );
        }
        _ => panic!("Expected Response for valid canary file read"),
    }

    // 3.3 CdmResourceProvider malicious / malformed URIs
    let malicious_cdm_uris = [
        // Empty document ID
        "cdm://",
        "cdm:///",
        "cdm:///markdown",
        // Subpath traversal
        &format!("cdm://{doc_id}/../../etc/passwd"),
        &format!("cdm://{doc_id}/../secret"),
        // Non-existent document ID
        "cdm://missing-doc-id-9999/markdown",
        // Unsupported format extensions
        &format!("cdm://{doc_id}/exe"),
        &format!("cdm://{doc_id}/bin"),
        &format!("cdm://{doc_id}/eval"),
        // Non-existent section ID
        &format!("cdm://{doc_id}/section/non_existent_section_id_xyz"),
    ];

    for (idx, uri) in malicious_cdm_uris.iter().enumerate() {
        let req = JsonRpcRequest::new(
            RequestId::Number(500 + idx as i64),
            "resources/read",
            Some(json!({ "uri": uri })),
        );
        client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();
        let msg = client.read_message().await.unwrap().unwrap();
        match msg {
            JsonRpcMessage::Response(res) => {
                assert!(
                    res.error.is_some(),
                    "Malicious CDM URI '{uri}' must be rejected with error"
                );
                assert_eq!(res.error.unwrap().code, jsonrpc_codes::RESOURCE_ERROR);
            }
            _ => panic!("Expected Response for malicious CDM read"),
        }
    }

    // Clean up temporary files
    let _ = fs::remove_dir_all(&test_dir);
    let _ = fs::remove_file(&sensitive_file);
}

// ============================================================================
// 4. Conversion Tool Adversarial Input Validation & Boundary Defenses
// ============================================================================

#[tokio::test]
async fn test_conversion_tool_ssrf_and_adversarial_input_boundaries() {
    let url_handler = ConvertUrlHandler::new();
    let repo_handler = ConvertRepoHandler::new();
    let detect_handler = DetectSourceHandler::new();
    let opt_handler = OptimizeContextHandler::new();

    // 4.1 convert_url SSRF & scheme defenses
    let ssrf_targets = [
        // IPv4 loopback
        "http://127.0.0.1/",
        "http://127.0.0.1:8080/admin",
        "http://localhost/",
        // IPv6 loopback
        "http://[::1]/",
        "http://[::1]:3000/metrics",
        // Cloud metadata endpoints
        "http://169.254.169.254/latest/meta-data/",
        "http://metadata.google.internal/computeMetadata/v1/",
        // Private Class A/B/C ranges
        "http://10.0.0.1/",
        "http://10.254.254.254/",
        "http://172.16.0.1/",
        "http://172.31.255.255/",
        "http://192.168.1.1/",
        "http://192.168.0.254/status",
        // Non-HTTP/HTTPS schemes
        "file:///etc/passwd",
        "data:text/html,<html>alert(1)</html>",
        "javascript:alert(1)",
        // Malformed targets
        "not a valid url",
        "http://",
        "http://:8080",
    ];

    for target in ssrf_targets {
        let args = json!({ "url": target });
        let result = url_handler
            .execute(Some(args))
            .await
            .expect("tool execution response");
        assert_eq!(
            result.is_error,
            Some(true),
            "Target '{target}' must be rejected by SSRF/validation defense"
        );
        let err_text = get_tool_text(&result);
        assert!(
            err_text.contains("URL validation rejected"),
            "Error text '{err_text}' must cite URL validation rejection"
        );
    }

    // 4.2 convert_repo subpath traversal & invalid target defenses
    let repo_traversal_subpaths = [
        "../../../secret",
        "..\\..\\Windows",
        "src/../../../etc",
        "src/\0/hidden",
        "%2e%2e/evil",
        "crates/../../outside",
    ];

    for sp in repo_traversal_subpaths {
        let args = json!({
            "target": "dayashimoga/web-repo-to-ai",
            "subpath": sp
        });
        let result = repo_handler
            .execute(Some(args))
            .await
            .expect("tool response");
        assert_eq!(
            result.is_error,
            Some(true),
            "Subpath traversal '{sp}' must be rejected"
        );
        let err_text = get_tool_text(&result);
        assert!(err_text.contains("Invalid subpath"));
    }

    // convert_repo with non-existent directory
    let non_existent_args = json!({
        "target": "/path/that/does/not/exist/anywhere/12345"
    });
    let result = repo_handler
        .execute(Some(non_existent_args))
        .await
        .expect("tool response");
    assert_eq!(result.is_error, Some(true));

    // convert_repo with malformed GitHub target
    let malformed_gh_args = json!({
        "target": "too/many/slash/components/in/this/target/spec"
    });
    let result = repo_handler
        .execute(Some(malformed_gh_args))
        .await
        .expect("tool response");
    assert_eq!(result.is_error, Some(true));

    // 4.3 optimize_context budget bounds & edge cases
    // Zero token budget
    let zero_budget_args = json!({
        "content": "# Line 1\n# Line 2\n# Line 3",
        "token_budget": 0
    });
    let res = opt_handler
        .execute(Some(zero_budget_args))
        .await
        .expect("optimize zero budget");
    assert!(res.is_error.is_none());
    let parsed: serde_json::Value = serde_json::from_str(get_tool_text(&res)).unwrap();
    assert_eq!(parsed["optimizedTokens"], 0);
    assert_eq!(parsed["truncated"], true);

    // Empty content
    let empty_content_args = json!({
        "content": "",
        "token_budget": 500
    });
    let res = opt_handler
        .execute(Some(empty_content_args))
        .await
        .expect("optimize empty content");
    assert!(res.is_error.is_none());
    let parsed: serde_json::Value = serde_json::from_str(get_tool_text(&res)).unwrap();
    assert_eq!(parsed["initialTokens"], 0);
    assert_eq!(parsed["truncated"], false);

    // Unknown model name falls back safely to Gpt4o
    let unknown_model_args = json!({
        "content": "fn test() { assert!(true); }",
        "token_budget": 100,
        "model": "unknown-futuristic-llm-9000"
    });
    let res = opt_handler
        .execute(Some(unknown_model_args))
        .await
        .expect("optimize unknown model");
    assert!(res.is_error.is_none());

    // 4.4 detect_source resilience with arbitrary/adversarial inputs
    // Huge input string
    let huge_input = "a".repeat(50_000);
    let res = detect_handler
        .execute(Some(json!({ "target": huge_input })))
        .await
        .expect("detect huge input");
    assert!(res.is_error.is_none());
    let parsed: serde_json::Value = serde_json::from_str(get_tool_text(&res)).unwrap();
    assert_eq!(parsed["sourceType"], "unknown");

    // Null bytes in input
    let null_byte_input = "https://example.com/\0/hidden";
    let res = detect_handler
        .execute(Some(json!({ "target": null_byte_input })))
        .await
        .expect("detect null byte");
    assert!(res.is_error.is_none());
    let parsed: serde_json::Value = serde_json::from_str(get_tool_text(&res)).unwrap();
    assert_eq!(parsed["sourceType"], "unknown");
}

// ============================================================================
// 5. Stream Transport Framing Resilience & Stream Isolation
// ============================================================================

#[tokio::test]
async fn test_stream_transport_framing_isolation_and_resilience() {
    let (client_duplex, server_duplex) = tokio::io::duplex(8192);
    let (server_r, server_w) = tokio::io::split(server_duplex);
    let (client_r, mut client_w) = tokio::io::split(client_duplex);

    let server_transport = StreamTransport::new(server_r, server_w);
    let mut client_lines = BufReader::new(client_r).lines();

    let server = McpServer::builder()
        .name("stream-framing-server")
        .with_defaults()
        .build();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    // 5.1 Send blank lines, keepalives, and whitespace-only lines (MUST be ignored)
    client_w.write_all(b"\n   \n\t\r\n\n").await.unwrap();
    client_w.flush().await.unwrap();

    // 5.2 Send malformed JSON syntax (MUST elicit -32700 Parse error with id: null)
    client_w
        .write_all(b"{\"broken_json_syntax: 123\n")
        .await
        .unwrap();
    client_w.flush().await.unwrap();

    let parse_err_line = client_lines
        .next_line()
        .await
        .unwrap()
        .expect("line expected");
    let parse_err: serde_json::Value =
        serde_json::from_str(&parse_err_line).expect("valid json response");
    assert_eq!(parse_err["jsonrpc"], "2.0");
    assert_eq!(parse_err["id"], serde_json::Value::Null);
    assert_eq!(parse_err["error"]["code"], jsonrpc_codes::PARSE_ERROR);

    // 5.3 Send invalid JSON-RPC 2.0 envelope (wrong jsonrpc: "1.0", MUST elicit -32600 Invalid Request)
    client_w
        .write_all(b"{\"jsonrpc\": \"1.0\", \"id\": 1, \"method\": \"ping\"}\n")
        .await
        .unwrap();
    client_w.flush().await.unwrap();

    let invalid_req_line = client_lines
        .next_line()
        .await
        .unwrap()
        .expect("line expected");
    let invalid_req: serde_json::Value =
        serde_json::from_str(&invalid_req_line).expect("valid json response");
    assert_eq!(invalid_req["jsonrpc"], "2.0");
    assert_eq!(invalid_req["id"], serde_json::Value::Null);
    assert_eq!(invalid_req["error"]["code"], jsonrpc_codes::INVALID_REQUEST);

    // 5.4 Send valid request immediately following errors (server remains resilient and functional)
    let valid_ping = r#"{"jsonrpc": "2.0", "id": 50, "method": "ping"}"#;
    client_w
        .write_all(format!("{valid_ping}\n").as_bytes())
        .await
        .unwrap();
    client_w.flush().await.unwrap();

    let ping_resp_line = client_lines
        .next_line()
        .await
        .unwrap()
        .expect("line expected");
    let ping_resp: serde_json::Value =
        serde_json::from_str(&ping_resp_line).expect("valid json response");
    assert_eq!(ping_resp["jsonrpc"], "2.0");
    assert_eq!(ping_resp["id"], 50);
    assert_eq!(ping_resp["result"], json!({}));

    // 5.5 Pipelined requests (multiple requests in single stream write)
    let pipelined_payload = format!(
        "{}\n{}\n{}\n",
        r#"{"jsonrpc": "2.0", "id": 61, "method": "ping"}"#,
        r#"{"jsonrpc": "2.0", "id": 62, "method": "ping"}"#,
        r#"{"jsonrpc": "2.0", "id": 63, "method": "ping"}"#,
    );
    client_w
        .write_all(pipelined_payload.as_bytes())
        .await
        .unwrap();
    client_w.flush().await.unwrap();

    for expected_id in 61..=63 {
        let line = client_lines
            .next_line()
            .await
            .unwrap()
            .expect("pipelined line expected");
        let resp: serde_json::Value = serde_json::from_str(&line).expect("valid json response");
        assert_eq!(resp["id"], expected_id);
        assert_eq!(resp["result"], json!({}));
    }

    // 5.6 Graceful stream termination on EOF
    drop(client_lines);
    drop(client_w);
    let outcome = server_handle.await.expect("server task joined");
    assert!(
        outcome.is_ok(),
        "Server loop should terminate cleanly on stream EOF"
    );
}

// ============================================================================
// 6. Multi-Client Concurrent Execution & Thread Safety
// ============================================================================

#[tokio::test]
async fn test_server_concurrent_requests_and_thread_safety() {
    let server = McpServer::builder()
        .name("concurrent-test-server")
        .with_defaults()
        .build();

    let (server_transport, client_transport) = ChannelTransport::pair(128);
    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    let client = Arc::new(tokio::sync::Mutex::new(client_transport));

    // Perform initialization handshake
    let init_req = JsonRpcRequest::new(
        RequestId::Number(1),
        "initialize",
        Some(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": { "name": "ConcurTester", "version": "1.0" }
        })),
    );
    client
        .lock()
        .await
        .send_raw(JsonRpcMessage::Request(init_req))
        .await
        .unwrap();
    let _init_resp = client.lock().await.read_message().await.unwrap().unwrap();

    let notif = JsonRpcNotification::new("notifications/initialized", None);
    client.lock().await.send_notification(&notif).await.unwrap();

    // Spawn 20 concurrent tasks performing mixed requests
    let task_count = 20;
    let (tx, mut rx) = mpsc::channel(task_count);

    for i in 0..task_count {
        let client_clone = client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let req_id = RequestId::Number(i as i64 + 100);
            let req = match i % 4 {
                0 => JsonRpcRequest::new(req_id.clone(), "ping", None),
                1 => JsonRpcRequest::new(req_id.clone(), "tools/list", None),
                2 => JsonRpcRequest::new(req_id.clone(), "resources/list", None),
                _ => JsonRpcRequest::new(
                    req_id.clone(),
                    "tools/call",
                    Some(json!({
                        "name": "detect_source",
                        "arguments": { "target": "https://docs.rs/tokio" }
                    })),
                ),
            };

            client_clone
                .lock()
                .await
                .send_raw(JsonRpcMessage::Request(req))
                .await
                .unwrap();
            tx_clone.send(req_id).await.unwrap();
        });
    }

    drop(tx);

    // Read all sent requests confirmation
    let mut sent_ids = Vec::new();
    while let Some(id) = rx.recv().await {
        sent_ids.push(id);
    }
    assert_eq!(sent_ids.len(), task_count);

    // Read all responses from client transport
    let mut received_responses = 0;
    for _ in 0..task_count {
        let msg = client
            .lock()
            .await
            .read_message()
            .await
            .unwrap()
            .expect("response expected");
        match msg {
            JsonRpcMessage::Response(res) => {
                assert!(
                    res.error.is_none(),
                    "Concurrent request must not fail: {res:?}"
                );
                received_responses += 1;
            }
            _ => panic!("Expected Response variant"),
        }
    }
    assert_eq!(received_responses, task_count);

    // Drop client half to trigger clean server shutdown
    drop(client);
    let server_res = server_handle.await.expect("join server handle");
    assert!(server_res.is_ok());
}
