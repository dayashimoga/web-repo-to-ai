//! # MCP Conversion Tools Integration Test Suite
//!
//! Validates `convert_url`, `convert_repo`, `detect_source`, and `optimize_context` tools:
//! schema validation, tool registration, SSRF defenses, path traversal defenses,
//! model token optimization, and multi-format exports.

use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use serde_json::json;

use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, RequestId,
};
use web_repo_to_ai_mcp::protocol::types::{
    CallToolResult, InitializeResult, ListToolsResult, ToolContent, LATEST_PROTOCOL_VERSION,
};
use web_repo_to_ai_mcp::tools::convert_repo::ConvertRepoHandler;
use web_repo_to_ai_mcp::tools::convert_url::ConvertUrlHandler;
use web_repo_to_ai_mcp::tools::detect_source::DetectSourceHandler;
use web_repo_to_ai_mcp::tools::optimize_context::OptimizeContextHandler;
use web_repo_to_ai_mcp::transport::channel::ChannelTransport;
use web_repo_to_ai_mcp::transport::trait_def::Transport;
use web_repo_to_ai_mcp::McpServer;
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::http::transport::{MockHttpTransport, MockResponse};

fn get_text(result: &CallToolResult) -> &str {
    match &result.content[0] {
        ToolContent::Text { text } => text,
        _ => panic!("Expected text content"),
    }
}

#[tokio::test]
async fn test_default_tools_registration_and_list() {
    let server = McpServer::builder()
        .name("test-server")
        .version("1.0.0")
        .with_default_tools()
        .build();

    let registry = server.registry();
    let reg = registry.read().await;

    let tools = reg.list_tools();
    assert_eq!(
        tools.len(),
        4,
        "Default server should register exactly 4 conversion tools"
    );

    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(tool_names.contains(&"convert_url"));
    assert!(tool_names.contains(&"convert_repo"));
    assert!(tool_names.contains(&"detect_source"));
    assert!(tool_names.contains(&"optimize_context"));

    for tool in &tools {
        assert!(tool.description.is_some());
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema.get("properties").is_some());
        assert!(tool.input_schema.get("required").is_some());
    }
}

#[tokio::test]
async fn test_detect_source_tool() {
    let handler = DetectSourceHandler::new();

    // 1. Local directory
    let local_res = handler
        .execute(Some(json!({ "target": "." })))
        .await
        .expect("detect_source should succeed");
    assert!(local_res.is_error.is_none());
    let local_val: serde_json::Value =
        serde_json::from_str(get_text(&local_res)).expect("valid json text");
    assert_eq!(local_val["sourceType"], "local_directory");
    assert_eq!(local_val["exists"], true);

    // 2. GitHub repository URL
    let gh_res = handler
        .execute(Some(
            json!({ "target": "https://github.com/rust-lang/rust" }),
        ))
        .await
        .expect("detect_source should succeed");
    assert!(gh_res.is_error.is_none());
    let gh_val: serde_json::Value =
        serde_json::from_str(get_text(&gh_res)).expect("valid json text");
    assert_eq!(gh_val["sourceType"], "github_repository");
    assert_eq!(gh_val["owner"], "rust-lang");
    assert_eq!(gh_val["repo"], "rust");

    // 3. Documentation URL
    let doc_res = handler
        .execute(Some(
            json!({ "target": "https://docs.rs/tokio/latest/tokio/" }),
        ))
        .await
        .expect("detect_source should succeed");
    assert!(doc_res.is_error.is_none());
    let doc_val: serde_json::Value =
        serde_json::from_str(get_text(&doc_res)).expect("valid json text");
    assert_eq!(doc_val["sourceType"], "web_documentation");
    assert_eq!(doc_val["domain"], "docs.rs");

    // 4. Unknown target
    let unknown_res = handler
        .execute(Some(
            json!({ "target": "invalid:::target-not-url-nor-path" }),
        ))
        .await
        .expect("detect_source should succeed");
    assert!(unknown_res.is_error.is_none());
    let unk_val: serde_json::Value =
        serde_json::from_str(get_text(&unknown_res)).expect("valid json text");
    assert_eq!(unk_val["sourceType"], "unknown");
}

#[tokio::test]
async fn test_optimize_context_tool() {
    let handler = OptimizeContextHandler::new();

    // 1. Content within budget
    let small_content = "This is a concise, single-paragraph text that fits well within budget.";
    let res = handler
        .execute(Some(json!({
            "content": small_content,
            "token_budget": 500
        })))
        .await
        .expect("optimize_context should succeed");
    assert!(res.is_error.is_none());
    let val: serde_json::Value = serde_json::from_str(get_text(&res)).expect("valid json text");
    assert_eq!(val["truncated"], false);
    assert_eq!(val["tokensPruned"], 0);
    assert_eq!(val["optimizedContent"], small_content);

    // 2. Content exceeding budget
    let large_content = (1..=200)
        .map(|i| {
            format!("Line {i}: This is structured log data for benchmarking context optimization.")
        })
        .collect::<Vec<_>>()
        .join("\n");

    let res_exceeded = handler
        .execute(Some(json!({
            "content": large_content,
            "token_budget": 50,
            "model": "claude-3-5-sonnet"
        })))
        .await
        .expect("optimize_context should succeed");
    assert!(res_exceeded.is_error.is_none());
    let val_exceeded: serde_json::Value =
        serde_json::from_str(get_text(&res_exceeded)).expect("valid json text");
    assert_eq!(val_exceeded["truncated"], true);
    assert!(val_exceeded["tokensPruned"].as_u64().unwrap() > 0);
    assert!(val_exceeded["optimizedTokens"].as_u64().unwrap() <= 50);
}

#[tokio::test]
async fn test_convert_url_ssrf_defense_rejection() {
    let handler = ConvertUrlHandler::new();

    // 1. Loopback IP rejection
    let res_loopback = handler
        .execute(Some(json!({ "url": "http://127.0.0.1:8080/secret" })))
        .await
        .expect("execute returns CallToolResult");
    assert_eq!(
        res_loopback.is_error,
        Some(true),
        "Loopback URL must be rejected by SSRF defense"
    );
    assert!(get_text(&res_loopback).contains("URL validation rejected"));

    // 2. Cloud metadata IP rejection
    let res_meta = handler
        .execute(Some(
            json!({ "url": "http://169.254.169.254/latest/meta-data" }),
        ))
        .await
        .expect("execute returns CallToolResult");
    assert_eq!(
        res_meta.is_error,
        Some(true),
        "Link-local metadata IP must be rejected"
    );

    // 3. Invalid scheme
    let res_scheme = handler
        .execute(Some(json!({ "url": "ftp://example.com/file.txt" })))
        .await
        .expect("execute returns CallToolResult");
    assert_eq!(
        res_scheme.is_error,
        Some(true),
        "Non-HTTP scheme must be rejected"
    );
}

#[tokio::test]
async fn test_convert_url_mock_fetch_and_multi_format_exports() {
    let mock_dns = MockDnsResolver::new();
    let host_ip = Ipv4Addr::new(93, 184, 216, 34);
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
    );

    let html_body = r#"<!DOCTYPE html>
<html>
<head><title>Universal Converter Documentation</title></head>
<body>
  <article>
    <h1>Universal Converter Documentation</h1>
    <p>This technical guide explains how web-repo-to-ai translates web pages into AI-ready context.</p>
  </article>
</body>
</html>"#;

    let mut headers = HashMap::new();
    headers.insert(
        "content-type".to_string(),
        "text/html; charset=utf-8".to_string(),
    );

    let mock_transport = Arc::new(MockHttpTransport::new());
    mock_transport.set_response(
        "https://example.com/docs/guide",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers,
            body: html_body.as_bytes().to_vec(),
        },
    );

    let policy = UrlSecurityPolicy::default();
    let secure_dns = SecureDnsResolver::with_resolver(policy, Arc::new(mock_dns));
    let handler = ConvertUrlHandler::with_deps(Some(mock_transport), Some(secure_dns));

    // 1. Export as Markdown (default)
    let md_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "markdown"
        })))
        .await
        .expect("convert_url should succeed");
    assert!(md_res.is_error.is_none());
    assert!(get_text(&md_res).contains("Universal Converter Documentation"));

    // 2. Export as JSON
    let json_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "json"
        })))
        .await
        .expect("convert_url JSON export should succeed");
    assert!(json_res.is_error.is_none());
    let parsed_json: serde_json::Value =
        serde_json::from_str(get_text(&json_res)).expect("valid JSON export");
    assert_eq!(parsed_json["schema_version"], "1.0.0");

    // 3. Export as Plain Text
    let txt_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "txt"
        })))
        .await
        .expect("convert_url text export should succeed");
    assert!(txt_res.is_error.is_none());
    assert!(get_text(&txt_res).contains("Universal Converter Documentation"));

    // 4. Export as HTML
    let html_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "html"
        })))
        .await
        .expect("convert_url html export should succeed");
    assert!(html_res.is_error.is_none());
    assert!(get_text(&html_res).contains("<!DOCTYPE html>"));

    // 5. Export as Manifest
    let manifest_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "manifest"
        })))
        .await
        .expect("convert_url manifest export should succeed");
    assert!(manifest_res.is_error.is_none());
    let manifest_val: serde_json::Value =
        serde_json::from_str(get_text(&manifest_res)).expect("valid manifest JSON");
    assert!(manifest_val.get("document_id").is_some());

    // 6. Export as Checksums
    let chk_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "checksums"
        })))
        .await
        .expect("convert_url checksums export should succeed");
    assert!(chk_res.is_error.is_none());
    assert!(get_text(&chk_res).contains("content.md"));

    // 7. Export as ZIP
    let zip_res = handler
        .execute(Some(json!({
            "url": "https://example.com/docs/guide",
            "format": "zip"
        })))
        .await
        .expect("convert_url zip export should succeed");
    assert!(zip_res.is_error.is_none());
    assert!(get_text(&zip_res).contains("[ZIP Package:"));
}

#[tokio::test]
async fn test_convert_repo_path_traversal_rejection() {
    let handler = ConvertRepoHandler::new();

    let res = handler
        .execute(Some(json!({
            "target": ".",
            "subpath": "../../etc/shadow"
        })))
        .await
        .expect("execute returns CallToolResult");
    assert_eq!(
        res.is_error,
        Some(true),
        "Path traversal in subpath must be rejected"
    );
    assert!(get_text(&res).contains("Invalid subpath"));
}

#[tokio::test]
async fn test_convert_repo_local_directory_extraction() {
    let test_dir =
        std::env::temp_dir().join(format!("web-repo-to-ai-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(test_dir.join("src")).expect("create temp test dir");

    fs::write(
        test_dir.join("src").join("lib.rs"),
        b"pub fn add(a: i32, b: i32) -> i32 { a + b }",
    )
    .expect("write test lib.rs");

    fs::write(
        test_dir.join("README.md"),
        b"# Test Monorepo\nThis is a test repository for conversion testing.",
    )
    .expect("write test README.md");

    // File in ignored directory
    fs::create_dir_all(test_dir.join("target")).expect("create target dir");
    fs::write(
        test_dir.join("target").join("dummy.rs"),
        b"// Should be ignored",
    )
    .expect("write target dummy");

    let handler = ConvertRepoHandler::new();

    // 1. Convert whole directory to Markdown
    let md_res = handler
        .execute(Some(json!({
            "target": test_dir.display().to_string(),
            "format": "markdown"
        })))
        .await
        .expect("convert_repo should succeed");
    assert!(md_res.is_error.is_none());
    assert!(get_text(&md_res).contains("src/lib.rs"));
    assert!(get_text(&md_res).contains("README.md"));
    assert!(
        !get_text(&md_res).contains("dummy.rs"),
        "target directory must be ignored"
    );

    // 2. Convert with subpath filter "src"
    let subpath_res = handler
        .execute(Some(json!({
            "target": test_dir.display().to_string(),
            "subpath": "src",
            "format": "markdown"
        })))
        .await
        .expect("convert_repo with subpath should succeed");
    assert!(subpath_res.is_error.is_none());
    assert!(get_text(&subpath_res).contains("src/lib.rs"));
    assert!(
        !get_text(&subpath_res).contains("README.md"),
        "README.md outside subpath must not be included"
    );

    // 3. Convert to JSON format
    let json_res = handler
        .execute(Some(json!({
            "target": test_dir.display().to_string(),
            "format": "json"
        })))
        .await
        .expect("convert_repo json should succeed");
    assert!(json_res.is_error.is_none());
    let parsed: serde_json::Value =
        serde_json::from_str(get_text(&json_res)).expect("valid JSON CDM output");
    assert_eq!(parsed["sections"].as_array().unwrap().len(), 2);

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}

#[tokio::test]
async fn test_dispatcher_tools_call_end_to_end() {
    let (server_transport, mut client_transport) = ChannelTransport::pair(32);

    let server = McpServer::builder()
        .name("conversion-server")
        .version("1.0.0")
        .with_default_tools()
        .build();

    let server_handle = tokio::spawn(async move { server.run(server_transport).await });

    // 1. Initialize handshake
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
            let result: InitializeResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.server_info.name, "conversion-server");
        }
        _ => panic!("Expected Response for initialize"),
    }

    // 2. Send initialized notification
    let init_notif = JsonRpcNotification::new("notifications/initialized", Some(json!({})));
    client_transport
        .send_raw(JsonRpcMessage::Notification(init_notif))
        .await
        .unwrap();

    // 3. List tools
    let list_req = JsonRpcRequest::new(RequestId::Number(2), "tools/list", None);
    client_transport
        .send_raw(JsonRpcMessage::Request(list_req))
        .await
        .unwrap();

    let list_msg = client_transport.read_message().await.unwrap().unwrap();
    match list_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            let result: ListToolsResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.tools.len(), 4);
        }
        _ => panic!("Expected Response for tools/list"),
    }

    // 4. Call detect_source tool
    let call_req = JsonRpcRequest::new(
        RequestId::Number(3),
        "tools/call",
        Some(json!({
            "name": "detect_source",
            "arguments": {
                "target": "https://github.com/facebook/react"
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
            assert!(result.is_error.is_none());
            let parsed: serde_json::Value =
                serde_json::from_str(get_text(&result)).expect("valid JSON detect output");
            assert_eq!(parsed["sourceType"], "github_repository");
            assert_eq!(parsed["owner"], "facebook");
            assert_eq!(parsed["repo"], "react");
        }
        _ => panic!("Expected Response for tools/call"),
    }

    // 5. Terminate cleanly
    drop(client_transport);
    let server_res = server_handle.await.expect("server exits cleanly");
    assert!(server_res.is_ok());
}
