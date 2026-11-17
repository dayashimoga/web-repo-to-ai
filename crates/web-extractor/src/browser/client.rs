//! # Browser Worker Asynchronous IPC Client
//!
//! Provides an async HTTP client for dispatching dynamic page rendering jobs,
//! polling health/readiness status, and parsing IPC responses from `services/browser-worker`.

use serde::Deserialize;
use std::time::Duration;

use super::error::BrowserWorkerError;
use super::model::{BrowserRenderRequest, BrowserRenderResponse, BrowserWorkerHealth};

/// Default HTTP request timeout for communication with the browser worker.
pub const DEFAULT_CLIENT_TIMEOUT: Duration = Duration::from_secs(45);

/// RFC 7807 problem details structure for parsing server errors.
#[derive(Debug, Deserialize)]
struct ProblemDetailsPayload {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    detail: Option<String>,
    #[serde(default)]
    error_code: Option<String>,
}

/// Asynchronous HTTP client for the isolated `services/browser-worker` microservice.
#[derive(Debug, Clone)]
pub struct BrowserWorkerClient {
    base_url: String,
    http_client: reqwest::Client,
}

impl BrowserWorkerClient {
    /// Constructs a client pointing to the given browser-worker base URL (e.g. `"http://127.0.0.1:3000"`).
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        let trimmed = base_url.trim_end_matches('/');
        let http_client = reqwest::Client::builder()
            .timeout(DEFAULT_CLIENT_TIMEOUT)
            .build()
            .unwrap_or_default();

        Self {
            base_url: trimmed.to_string(),
            http_client,
        }
    }

    /// Sets a custom HTTP transport timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http_client = reqwest::Client::builder()
            .timeout(timeout)
            .build()
            .unwrap_or(self.http_client);
        self
    }

    /// Checks the liveness health status of the browser worker service (`GET /health`).
    pub async fn check_health(&self) -> Result<BrowserWorkerHealth, BrowserWorkerError> {
        let url = format!("{}/health", self.base_url);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        if !status.is_success() {
            return Err(BrowserWorkerError::UnexpectedStatus {
                status: status.as_u16(),
                message: body,
            });
        }

        serde_json::from_str::<BrowserWorkerHealth>(&body)
            .map_err(|e| BrowserWorkerError::DeserializationFailed(e.to_string()))
    }

    /// Checks the browser worker readiness status (`GET /ready`).
    pub async fn check_readiness(&self) -> Result<BrowserWorkerHealth, BrowserWorkerError> {
        let url = format!("{}/ready", self.base_url);
        let resp = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        if !status.is_success() {
            return Err(BrowserWorkerError::UnexpectedStatus {
                status: status.as_u16(),
                message: body,
            });
        }

        serde_json::from_str::<BrowserWorkerHealth>(&body)
            .map_err(|e| BrowserWorkerError::DeserializationFailed(e.to_string()))
    }

    /// Dispatches a page rendering request to the browser worker (`POST /render`).
    pub async fn render(
        &self,
        request: &BrowserRenderRequest,
    ) -> Result<BrowserRenderResponse, BrowserWorkerError> {
        let url = format!("{}/render", self.base_url);
        let json_body = serde_json::to_string(request)
            .map_err(|e| BrowserWorkerError::InvalidRequest(e.to_string()))?;

        let resp = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(json_body)
            .send()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        let status = resp.status();
        let body_text = resp
            .text()
            .await
            .map_err(|e| BrowserWorkerError::HttpTransportError(e.to_string()))?;

        if status.is_success() {
            return serde_json::from_str::<BrowserRenderResponse>(&body_text)
                .map_err(|e| BrowserWorkerError::DeserializationFailed(e.to_string()));
        }

        // Parse error response (RFC 7807 problem details or plain body)
        let status_code = status.as_u16();

        if let Ok(problem) = serde_json::from_str::<ProblemDetailsPayload>(&body_text) {
            let code = problem
                .error_code
                .or(problem.title)
                .unwrap_or_else(|| "Error".to_string());
            let detail = problem.detail.unwrap_or_else(|| body_text.clone());

            match code.as_str() {
                "InvalidRequest" | "InvalidOption" | "InvalidUrl" | "MalformedJson" => {
                    return Err(BrowserWorkerError::InvalidRequest(detail));
                }
                "InvalidScheme" | "BlockedHost" | "BlockedUrl" => {
                    return Err(BrowserWorkerError::BlockedUrl(detail));
                }
                "NavigationTimeout" | "SelectorTimeout" => {
                    return Err(BrowserWorkerError::NavigationTimeout(detail));
                }
                "BrowserCrash" => {
                    return Err(BrowserWorkerError::BrowserCrash(detail));
                }
                _ => {}
            }
        }

        Err(BrowserWorkerError::UnexpectedStatus {
            status: status_code,
            message: body_text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::model::{Viewport, WaitUntil};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[test]
    fn test_render_request_builder() {
        let req = BrowserRenderRequest::new("https://example.com/app")
            .with_wait_until(WaitUntil::DomContentLoaded)
            .with_timeout_ms(15000)
            .with_wait_for_selector("#main")
            .with_viewport(1920, 1080)
            .with_user_agent("TestBot/1.0")
            .with_header("Authorization", "Bearer xyz")
            .with_javascript(false)
            .with_block_subresource_ips(true);

        assert_eq!(req.url, "https://example.com/app");
        assert_eq!(req.wait_until, WaitUntil::DomContentLoaded);
        assert_eq!(req.timeout_ms, 15000);
        assert_eq!(req.wait_for_selector.as_deref(), Some("#main"));
        assert_eq!(req.viewport, Viewport::new(1920, 1080));
        assert_eq!(req.user_agent.as_deref(), Some("TestBot/1.0"));
        assert_eq!(
            req.headers.get("Authorization").map(String::as_str),
            Some("Bearer xyz")
        );
        assert!(!req.javascript_enabled);
        assert!(req.block_subresource_ips);

        let json = serde_json::to_string(&req).unwrap();
        let parsed: BrowserRenderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, req);
    }

    #[tokio::test]
    async fn test_browser_worker_client_health_and_render_mock() {
        // Spawn an ephemeral TCP server to simulate browser-worker HTTP service
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let req_str = String::from_utf8_lossy(&buf[..n]);

                    if req_str.starts_with("GET /health") {
                        let body = r#"{"status":"ok","service":"browser-worker","version":"0.1.0","timestamp":"2026-11-17T00:00:00Z"}"#;
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = socket.write_all(resp.as_bytes()).await;
                    } else if req_str.starts_with("GET /ready") {
                        let body = r#"{"status":"ready","service":"browser-worker","version":"0.1.0","browser_ready":true}"#;
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = socket.write_all(resp.as_bytes()).await;
                    } else if req_str.starts_with("POST /render") {
                        if req_str.contains("blocked.example.com") {
                            let body = r#"{"type":"https://web-repo-to-ai.dev/errors/blockedurl","title":"BlockedUrl","status":400,"detail":"Private IP range blocked","error_code":"BlockedUrl"}"#;
                            let resp = format!(
                                "HTTP/1.1 400 Bad Request\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(),
                                body
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                        } else if req_str.contains("timeout.example.com") {
                            let body = r#"{"type":"https://web-repo-to-ai.dev/errors/selectortimeout","title":"SelectorTimeout","status":504,"detail":"Timed out waiting for selector '#app'","error_code":"SelectorTimeout"}"#;
                            let resp = format!(
                                "HTTP/1.1 504 Gateway Timeout\r\nContent-Type: application/problem+json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(),
                                body
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                        } else {
                            let body = r#"{
                                "url":"https://example.com/app",
                                "status_code":200,
                                "html":"<html><head><title>App</title></head><body><main>Content</main></body></html>",
                                "title":"App",
                                "load_time_ms":120,
                                "redirects":[],
                                "console_messages":[],
                                "page_errors":[],
                                "blocked_subresources":[],
                                "timing":{"navigation_start":1,"dom_content_loaded":2,"load":3}
                            }"#;
                            let resp = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(),
                                body
                            );
                            let _ = socket.write_all(resp.as_bytes()).await;
                        }
                    }
                });
            }
        });

        let client = BrowserWorkerClient::new(&base_url);

        // 1. Health check
        let health = client.check_health().await.unwrap();
        assert_eq!(health.status, "ok");
        assert_eq!(health.service, "browser-worker");

        // 2. Readiness check
        let readiness = client.check_readiness().await.unwrap();
        assert_eq!(readiness.status, "ready");
        assert_eq!(readiness.browser_ready, Some(true));

        // 3. Render success
        let req = BrowserRenderRequest::new("https://example.com/app");
        let resp = client.render(&req).await.unwrap();
        assert_eq!(resp.url, "https://example.com/app");
        assert_eq!(resp.status_code, 200);
        assert_eq!(resp.title, "App");
        assert!(resp.html.contains("<main>Content</main>"));

        // 4. Render blocked URL error
        let blocked_req = BrowserRenderRequest::new("https://blocked.example.com");
        let err = client.render(&blocked_req).await.unwrap_err();
        match err {
            BrowserWorkerError::BlockedUrl(msg) => {
                assert!(msg.contains("Private IP range blocked"));
            }
            other => panic!("Expected BlockedUrl error, got {other:?}"),
        }

        // 5. Render selector timeout error
        let timeout_req = BrowserRenderRequest::new("https://timeout.example.com");
        let err = client.render(&timeout_req).await.unwrap_err();
        match err {
            BrowserWorkerError::NavigationTimeout(msg) => {
                assert!(msg.contains("Timed out waiting for selector"));
            }
            other => panic!("Expected NavigationTimeout error, got {other:?}"),
        }
    }
}
