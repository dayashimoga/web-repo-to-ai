//! # MCP Resource Retrieval Integration Test Suite
//!
//! Validates MCP 2024-11-05 resource retrieval capabilities:
//! - URI template discovery (`resources/templates/list`)
//! - Concrete resource enumeration (`resources/list`)
//! - Multi-format CDM document retrieval (`resources/read` for `cdm://` URIs)
//! - Quality and completeness audit reports (`cdm://{id}/report`)
//! - Base64 ZIP archive payload packaging (`cdm://{id}/zip`)
//! - Granular section retrieval (`cdm://{id}/section/{section_id}`)
//! - Canonical root boundary defense against path traversal (`file://` and `artifact://`)
//! - End-to-end pipeline: tool execution -> document store -> resource provider retrieval

use std::fs;
use std::sync::Arc;

use serde_json::json;

use web_repo_to_ai_cdm::builder::{CdmDocumentBuilder, CdmSectionBuilder};
use web_repo_to_ai_cdm::model::{SourceIdentity, SourceType};
use web_repo_to_ai_cdm::section::SectionKind;
use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, RequestId,
};
use web_repo_to_ai_mcp::protocol::types::{
    CallToolResult, ListResourceTemplatesResult, ListResourcesResult, ReadResourceResult,
    ToolContent, LATEST_PROTOCOL_VERSION,
};
use web_repo_to_ai_mcp::resources::base64::decode_base64;
use web_repo_to_ai_mcp::resources::{FileResourceProvider, McpDocumentStore};
use web_repo_to_ai_mcp::transport::channel::ChannelTransport;
use web_repo_to_ai_mcp::transport::trait_def::Transport;
use web_repo_to_ai_mcp::McpServer;

/// Helper initializing a connected client transport over an active McpServer.
async fn setup_initialized_client(server: McpServer) -> ChannelTransport {
    let (server_transport, mut client_transport) = ChannelTransport::pair(64);
    tokio::spawn(async move {
        let _ = server.run(server_transport).await;
    });

    let init_req = JsonRpcRequest::new(
        RequestId::Number(1),
        "initialize",
        Some(json!({
            "protocolVersion": LATEST_PROTOCOL_VERSION,
            "capabilities": {},
            "clientInfo": {
                "name": "test-resource-client",
                "version": "1.0.0"
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
        }
        _ => panic!("Expected Response for initialize"),
    }

    let init_notif = JsonRpcNotification::new("notifications/initialized", Some(json!({})));
    client_transport
        .send_raw(JsonRpcMessage::Notification(init_notif))
        .await
        .unwrap();

    client_transport
}

/// Helper building a representative test CDM document.
fn create_test_cdm_document() -> web_repo_to_ai_cdm::model::CdmDocument {
    let source_id = SourceIdentity::new("acme-org", "converter-docs");
    let mut builder = CdmDocumentBuilder::new(
        source_id,
        SourceType::WebpageStatic,
        "https://example.com/docs/guide",
    );

    let sec1 = CdmSectionBuilder::new(
        "overview",
        "https://example.com/docs/guide#overview",
        SectionKind::Prose,
        "# System Overview\n\nThis is the architecture guide for the converter system.",
    )
    .title("System Overview")
    .original_path("overview.md")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "quickstart",
        "https://example.com/docs/guide#quickstart",
        SectionKind::CodeFile,
        "fn main() {\n    println!(\"Hello MCP Resources!\");\n}",
    )
    .title("Quickstart Code")
    .original_path("src/main.rs")
    .language("rust")
    .build();

    builder = builder.add_section(sec1).add_section(sec2);
    builder.build()
}

#[tokio::test]
async fn test_resources_templates_list() {
    let server = McpServer::builder()
        .name("templates-test-server")
        .with_default_resources()
        .build();

    let mut client = setup_initialized_client(server).await;

    let req = JsonRpcRequest::new(RequestId::Number(2), "resources/templates/list", None);
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());

            let result: ListResourceTemplatesResult =
                serde_json::from_value(res.result.unwrap()).unwrap();
            let templates = result.resource_templates;

            assert!(
                templates.len() >= 12,
                "Expected at least 12 resource templates (10 cdm, 2 filesystem/artifact), got {}",
                templates.len()
            );

            let uri_templates: Vec<&str> =
                templates.iter().map(|t| t.uri_template.as_str()).collect();

            // Verify CDM URI templates
            assert!(uri_templates.contains(&"cdm://{document_id}/markdown"));
            assert!(uri_templates.contains(&"cdm://{document_id}/txt"));
            assert!(uri_templates.contains(&"cdm://{document_id}/json"));
            assert!(uri_templates.contains(&"cdm://{document_id}/html"));
            assert!(uri_templates.contains(&"cdm://{document_id}/jsonl"));
            assert!(uri_templates.contains(&"cdm://{document_id}/manifest"));
            assert!(uri_templates.contains(&"cdm://{document_id}/checksums"));
            assert!(uri_templates.contains(&"cdm://{document_id}/report"));
            assert!(uri_templates.contains(&"cdm://{document_id}/zip"));
            assert!(uri_templates.contains(&"cdm://{document_id}/section/{section_id}"));

            // Verify Filesystem / Artifact URI templates
            assert!(uri_templates.contains(&"file://{path}"));
            assert!(uri_templates.contains(&"artifact://{path}"));
        }
        _ => panic!("Expected Response for resources/templates/list"),
    }
}

#[tokio::test]
async fn test_resources_list_empty_and_populated() {
    let doc_store = Arc::new(McpDocumentStore::new());
    let server = McpServer::builder()
        .name("list-test-server")
        .with_document_store(doc_store.clone())
        .with_default_resources()
        .build();

    let mut client = setup_initialized_client(server).await;

    // 1. Initial list when store is empty
    let req1 = JsonRpcRequest::new(RequestId::Number(2), "resources/list", None);
    client
        .send_raw(JsonRpcMessage::Request(req1))
        .await
        .unwrap();

    let msg1 = client.read_message().await.unwrap().unwrap();
    match msg1 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());
            let result: ListResourcesResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert!(result.resources.is_empty(), "Initial list should be empty");
        }
        _ => panic!("Expected Response for resources/list"),
    }

    // 2. Insert test document into store
    let test_doc = create_test_cdm_document();
    let doc_id = test_doc.document_id.to_string();
    doc_store.insert(test_doc);

    // 3. Second list after document insertion
    let req2 = JsonRpcRequest::new(RequestId::Number(3), "resources/list", None);
    client
        .send_raw(JsonRpcMessage::Request(req2))
        .await
        .unwrap();

    let msg2 = client.read_message().await.unwrap().unwrap();
    match msg2 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            assert!(res.error.is_none());
            let result: ListResourcesResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(
                result.resources.len(),
                6,
                "Expected 6 concrete resources per document"
            );

            let uris: Vec<&str> = result.resources.iter().map(|r| r.uri.as_str()).collect();
            assert!(uris.contains(&format!("cdm://{doc_id}/markdown").as_str()));
            assert!(uris.contains(&format!("cdm://{doc_id}/json").as_str()));
            assert!(uris.contains(&format!("cdm://{doc_id}/manifest").as_str()));
            assert!(uris.contains(&format!("cdm://{doc_id}/checksums").as_str()));
            assert!(uris.contains(&format!("cdm://{doc_id}/report").as_str()));
            assert!(uris.contains(&format!("cdm://{doc_id}/zip").as_str()));
        }
        _ => panic!("Expected Response for resources/list"),
    }
}

#[tokio::test]
async fn test_resources_read_cdm_formats() {
    let doc_store = Arc::new(McpDocumentStore::new());
    let server = McpServer::builder()
        .name("read-formats-server")
        .with_document_store(doc_store.clone())
        .with_default_resources()
        .build();

    let test_doc = create_test_cdm_document();
    let doc_id = test_doc.document_id.to_string();
    doc_store.insert(test_doc);

    let mut client = setup_initialized_client(server).await;

    // 1. Read Markdown
    let md_uri = format!("cdm://{doc_id}/markdown");
    let req = JsonRpcRequest::new(
        RequestId::Number(2),
        "resources/read",
        Some(json!({ "uri": md_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.contents.len(), 1);
            let item = &result.contents[0];
            assert_eq!(item.uri, md_uri);
            assert_eq!(item.mime_type.as_deref(), Some("text/markdown"));
            let text = item.text.as_ref().expect("text content");
            assert!(text.contains("System Overview"));
            assert!(text.contains("Quickstart Code"));
            assert!(item.blob.is_none());
        }
        _ => panic!("Expected Response for resources/read (markdown)"),
    }

    // 2. Read Plain Text
    let txt_uri = format!("cdm://{doc_id}/txt");
    let req = JsonRpcRequest::new(
        RequestId::Number(3),
        "resources/read",
        Some(json!({ "uri": txt_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/plain"));
            assert!(item.text.as_ref().unwrap().contains("System Overview"));
        }
        _ => panic!("Expected Response for resources/read (txt)"),
    }

    // 3. Read JSON (Canonical CDM)
    let json_uri = format!("cdm://{doc_id}/json");
    let req = JsonRpcRequest::new(
        RequestId::Number(4),
        "resources/read",
        Some(json!({ "uri": json_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(4));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/json"));
            let parsed: serde_json::Value =
                serde_json::from_str(item.text.as_ref().unwrap()).expect("valid JSON CDM");
            assert_eq!(parsed["document_id"], doc_id);
            assert_eq!(parsed["schema_version"], "1.0.0");
        }
        _ => panic!("Expected Response for resources/read (json)"),
    }

    // 4. Read HTML
    let html_uri = format!("cdm://{doc_id}/html");
    let req = JsonRpcRequest::new(
        RequestId::Number(5),
        "resources/read",
        Some(json!({ "uri": html_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(5));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/html"));
            assert!(item.text.as_ref().unwrap().contains("<!DOCTYPE html>"));
        }
        _ => panic!("Expected Response for resources/read (html)"),
    }

    // 5. Read JSONL
    let jsonl_uri = format!("cdm://{doc_id}/jsonl");
    let req = JsonRpcRequest::new(
        RequestId::Number(6),
        "resources/read",
        Some(json!({ "uri": jsonl_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(6));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/x-ndjson"));
            assert!(!item.text.as_ref().unwrap().is_empty());
        }
        _ => panic!("Expected Response for resources/read (jsonl)"),
    }

    // 6. Read Manifest
    let manifest_uri = format!("cdm://{doc_id}/manifest");
    let req = JsonRpcRequest::new(
        RequestId::Number(7),
        "resources/read",
        Some(json!({ "uri": manifest_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(7));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/json"));
            let parsed: serde_json::Value =
                serde_json::from_str(item.text.as_ref().unwrap()).expect("valid manifest JSON");
            assert!(parsed.get("document_id").is_some());
        }
        _ => panic!("Expected Response for resources/read (manifest)"),
    }

    // 7. Read Checksums
    let chk_uri = format!("cdm://{doc_id}/checksums");
    let req = JsonRpcRequest::new(
        RequestId::Number(8),
        "resources/read",
        Some(json!({ "uri": chk_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(8));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/plain"));
            assert!(item.text.as_ref().unwrap().contains("content.md"));
        }
        _ => panic!("Expected Response for resources/read (checksums)"),
    }

    // 8. Read Completeness / Quality Report
    let report_uri = format!("cdm://{doc_id}/report");
    let req = JsonRpcRequest::new(
        RequestId::Number(9),
        "resources/read",
        Some(json!({ "uri": report_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(9));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/json"));
            let parsed: serde_json::Value =
                serde_json::from_str(item.text.as_ref().unwrap()).expect("valid report JSON");
            assert_eq!(parsed["document_id"], doc_id);
            assert!(parsed.get("completeness_percentage").is_some());
        }
        _ => panic!("Expected Response for resources/read (report)"),
    }

    // 9. Read ZIP package
    let zip_uri = format!("cdm://{doc_id}/zip");
    let req = JsonRpcRequest::new(
        RequestId::Number(10),
        "resources/read",
        Some(json!({ "uri": zip_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(10));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/zip"));
            assert!(item.text.is_none());
            let b64 = item.blob.as_ref().expect("ZIP blob content");
            let raw_zip = decode_base64(b64).expect("valid Base64 decoded bytes");
            // Check ZIP magic bytes PK\x03\x04
            assert!(raw_zip.len() >= 4);
            assert_eq!(&raw_zip[0..4], &[0x50, 0x4b, 0x03, 0x04]);
        }
        _ => panic!("Expected Response for resources/read (zip)"),
    }

    // 10. Read Granular Section
    let sec_uri = format!("cdm://{doc_id}/section/overview");
    let req = JsonRpcRequest::new(
        RequestId::Number(11),
        "resources/read",
        Some(json!({ "uri": sec_uri })),
    );
    client.send_raw(JsonRpcMessage::Request(req)).await.unwrap();

    let msg = client.read_message().await.unwrap().unwrap();
    match msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(11));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/plain"));
            assert!(item.text.as_ref().unwrap().contains("architecture guide"));
        }
        _ => panic!("Expected Response for resources/read (section/overview)"),
    }
}

#[tokio::test]
async fn test_resources_read_not_found_and_errors() {
    let server = McpServer::builder()
        .name("error-test-server")
        .with_default_resources()
        .build();

    let doc_store = server.document_store();
    let mut client = setup_initialized_client(server).await;

    // 1. Non-existent document
    let req1 = JsonRpcRequest::new(
        RequestId::Number(2),
        "resources/read",
        Some(json!({ "uri": "cdm://non-existent-id/markdown" })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req1))
        .await
        .unwrap();

    let msg1 = client.read_message().await.unwrap().unwrap();
    match msg1 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            let err = res.error.expect("Expected error for non-existent doc");
            assert_eq!(err.code, -32003);
            assert!(err.message.contains("Document 'non-existent-id' not found"));
        }
        _ => panic!("Expected Response with error"),
    }

    // 2. Insert document and test non-existent section
    let doc = create_test_cdm_document();
    let doc_id = doc.document_id.to_string();
    doc_store.insert(doc);

    let req2 = JsonRpcRequest::new(
        RequestId::Number(3),
        "resources/read",
        Some(json!({ "uri": format!("cdm://{doc_id}/section/missing_section") })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req2))
        .await
        .unwrap();

    let msg2 = client.read_message().await.unwrap().unwrap();
    match msg2 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            let err = res.error.expect("Expected error for missing section");
            assert_eq!(err.code, -32003);
            assert!(err.message.contains("Section 'missing_section' not found"));
        }
        _ => panic!("Expected Response with error"),
    }

    // 3. Unsupported subpath
    let req3 = JsonRpcRequest::new(
        RequestId::Number(4),
        "resources/read",
        Some(json!({ "uri": format!("cdm://{doc_id}/unsupported_type") })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req3))
        .await
        .unwrap();

    let msg3 = client.read_message().await.unwrap().unwrap();
    match msg3 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(4));
            let err = res.error.expect("Expected error for unsupported subpath");
            assert_eq!(err.code, -32003);
            assert!(err.message.contains("Unsupported resource subpath"));
        }
        _ => panic!("Expected Response with error"),
    }
}

#[tokio::test]
async fn test_file_and_artifact_resource_provider() {
    let test_dir = std::env::temp_dir().join(format!("mcp-file-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&test_dir).expect("create test directory");

    let sub_dir = test_dir.join("artifacts");
    fs::create_dir_all(&sub_dir).expect("create sub directory");

    let file_path = test_dir.join("sample.txt");
    fs::write(&file_path, b"Hello from monorepo source file!").expect("write sample.txt");

    let artifact_path = sub_dir.join("bundle.json");
    fs::write(&artifact_path, b"{\"bundle_id\": \"b-100\"}").expect("write bundle.json");

    // Parent directory outside test_dir to test traversal
    let outside_path = std::env::temp_dir().join(format!("mcp-outside-{}", uuid::Uuid::new_v4()));
    fs::write(&outside_path, b"CONFIDENTIAL").expect("write outside file");

    let file_provider = Arc::new(FileResourceProvider::new(&test_dir));

    let server = McpServer::builder()
        .name("file-test-server")
        .register_resource_provider(file_provider)
        .build();

    let mut client = setup_initialized_client(server).await;

    // 1. Read file:// URI
    let req1 = JsonRpcRequest::new(
        RequestId::Number(2),
        "resources/read",
        Some(json!({ "uri": "file://sample.txt" })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req1))
        .await
        .unwrap();

    let msg1 = client.read_message().await.unwrap().unwrap();
    match msg1 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/plain"));
            assert_eq!(
                item.text.as_deref(),
                Some("Hello from monorepo source file!")
            );
        }
        _ => panic!("Expected Response for file://sample.txt"),
    }

    // 2. Read artifact:// URI
    let req2 = JsonRpcRequest::new(
        RequestId::Number(3),
        "resources/read",
        Some(json!({ "uri": "artifact://artifacts/bundle.json" })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req2))
        .await
        .unwrap();

    let msg2 = client.read_message().await.unwrap().unwrap();
    match msg2 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/json"));
            assert_eq!(item.text.as_deref(), Some("{\"bundle_id\": \"b-100\"}"));
        }
        _ => panic!("Expected Response for artifact://artifacts/bundle.json"),
    }

    // 3. Path traversal attack rejection
    let req3 = JsonRpcRequest::new(
        RequestId::Number(4),
        "resources/read",
        Some(json!({ "uri": "file://../secret.txt" })),
    );
    client
        .send_raw(JsonRpcMessage::Request(req3))
        .await
        .unwrap();

    let msg3 = client.read_message().await.unwrap().unwrap();
    match msg3 {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(4));
            let err = res.error.expect("Expected error for traversal attempt");
            assert_eq!(err.code, -32003);
        }
        _ => panic!("Expected Response with error"),
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
    let _ = fs::remove_file(&outside_path);
}

#[tokio::test]
async fn test_end_to_end_tool_conversion_to_resource_pipeline() {
    let test_dir = std::env::temp_dir().join(format!("mcp-e2e-pipeline-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(test_dir.join("src")).expect("create test dir");

    fs::write(
        test_dir.join("src").join("main.rs"),
        b"fn main() { println!(\"Pipeline test!\"); }",
    )
    .expect("write main.rs");

    fs::write(
        test_dir.join("README.md"),
        b"# Pipeline Test\nTesting tool execution into document store and resource retrieval.",
    )
    .expect("write README.md");

    let server = McpServer::builder()
        .name("pipeline-server")
        .with_defaults()
        .build();

    let doc_store = server.document_store();
    let mut client = setup_initialized_client(server).await;

    // 1. Initial document store count is 0
    assert_eq!(doc_store.len(), 0);

    // 2. Call convert_repo tool
    let call_req = JsonRpcRequest::new(
        RequestId::Number(2),
        "tools/call",
        Some(json!({
            "name": "convert_repo",
            "arguments": {
                "target": test_dir.display().to_string(),
                "format": "markdown"
            }
        })),
    );
    client
        .send_raw(JsonRpcMessage::Request(call_req))
        .await
        .unwrap();

    let call_msg = client.read_message().await.unwrap().unwrap();
    match call_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(2));
            assert!(res.error.is_none());
            let result: CallToolResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert!(result.is_error.is_none());
            match &result.content[0] {
                ToolContent::Text { text } => {
                    assert!(text.contains("Pipeline Test"));
                    assert!(text.contains("src/main.rs"));
                }
                _ => panic!("Expected text tool content"),
            }
        }
        _ => panic!("Expected Response for tools/call"),
    }

    // 3. Verify document was inserted into document store
    assert_eq!(doc_store.len(), 1);
    let doc_summaries = doc_store.list();
    let doc_id = doc_summaries[0].document_id.clone();

    // 4. List resources and verify concrete URIs are returned
    let list_req = JsonRpcRequest::new(RequestId::Number(3), "resources/list", None);
    client
        .send_raw(JsonRpcMessage::Request(list_req))
        .await
        .unwrap();

    let list_msg = client.read_message().await.unwrap().unwrap();
    match list_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(3));
            assert!(res.error.is_none());
            let result: ListResourcesResult = serde_json::from_value(res.result.unwrap()).unwrap();
            assert_eq!(result.resources.len(), 6);
        }
        _ => panic!("Expected Response for resources/list"),
    }

    // 5. Read converted document through cdm:// URI
    let read_req = JsonRpcRequest::new(
        RequestId::Number(4),
        "resources/read",
        Some(json!({ "uri": format!("cdm://{doc_id}/markdown") })),
    );
    client
        .send_raw(JsonRpcMessage::Request(read_req))
        .await
        .unwrap();

    let read_msg = client.read_message().await.unwrap().unwrap();
    match read_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(4));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("text/markdown"));
            let text = item.text.as_ref().unwrap();
            assert!(text.contains("Pipeline Test"));
            assert!(text.contains("src/main.rs"));
        }
        _ => panic!("Expected Response for resources/read"),
    }

    // 6. Read ZIP package through cdm:// URI
    let zip_req = JsonRpcRequest::new(
        RequestId::Number(5),
        "resources/read",
        Some(json!({ "uri": format!("cdm://{doc_id}/zip") })),
    );
    client
        .send_raw(JsonRpcMessage::Request(zip_req))
        .await
        .unwrap();

    let zip_msg = client.read_message().await.unwrap().unwrap();
    match zip_msg {
        JsonRpcMessage::Response(res) => {
            assert_eq!(res.id, RequestId::Number(5));
            assert!(res.error.is_none());
            let result: ReadResourceResult = serde_json::from_value(res.result.unwrap()).unwrap();
            let item = &result.contents[0];
            assert_eq!(item.mime_type.as_deref(), Some("application/zip"));
            let b64 = item.blob.as_ref().unwrap();
            let zip_bytes = decode_base64(b64).unwrap();
            assert_eq!(&zip_bytes[0..4], &[0x50, 0x4b, 0x03, 0x04]);
        }
        _ => panic!("Expected Response for resources/read (zip)"),
    }

    // Clean up
    let _ = fs::remove_dir_all(&test_dir);
}
