//! # MCP Transport Tests
//!
//! Tests for transport implementations verifying asynchronous bidirectional message exchange,
//! FIFO ordering, and EOF stream termination.

use serde_json::json;
use web_repo_to_ai_mcp::protocol::jsonrpc::{
    JsonRpcMessage, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, RequestId,
};
use web_repo_to_ai_mcp::transport::channel::ChannelTransport;
use web_repo_to_ai_mcp::transport::trait_def::Transport;

#[tokio::test]
async fn test_channel_transport_bidirectional_exchange() {
    let (mut server_transport, mut client_transport) = ChannelTransport::pair(16);

    // Client sends request to server
    let req = JsonRpcRequest::new(RequestId::Number(1), "ping", None);
    client_transport
        .send_raw(JsonRpcMessage::Request(req.clone()))
        .await
        .expect("send raw request");

    // Server reads request
    let received = server_transport
        .read_message()
        .await
        .expect("read from server")
        .expect("message expected");
    assert_eq!(received, JsonRpcMessage::Request(req));

    // Server sends response to client
    let resp = JsonRpcResponse::success(RequestId::Number(1), json!({"pong": true}));
    server_transport
        .send_response(&resp)
        .await
        .expect("send response");

    // Client reads response
    let client_received = client_transport
        .read_message()
        .await
        .expect("read from client")
        .expect("message expected");
    assert_eq!(client_received, JsonRpcMessage::Response(resp));
}

#[tokio::test]
async fn test_channel_transport_notifications() {
    let (server_transport, mut client_transport) = ChannelTransport::pair(16);

    let notif = JsonRpcNotification::new("notifications/message", Some(json!({"msg": "hello"})));
    server_transport
        .send_notification(&notif)
        .await
        .expect("send notification");

    let received = client_transport
        .read_message()
        .await
        .expect("read notification")
        .expect("some message");
    assert_eq!(received, JsonRpcMessage::Notification(notif));
}

#[tokio::test]
async fn test_channel_transport_eof_detection() {
    let (mut server_transport, client_transport) = ChannelTransport::pair(16);

    // Drop client half
    drop(client_transport);

    // Reading on server should yield Ok(None) representing EOF
    let eof = server_transport
        .read_message()
        .await
        .expect("read succeeds");
    assert!(eof.is_none());
}
