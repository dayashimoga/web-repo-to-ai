//! # HTTP Transport Abstraction
//!
//! Provides pluggable network transport execution with pinned socket addresses,
//! zero-automatic-redirect policies, and streaming response body delivery.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::redirect::Policy;
use reqwest::{Client, Method};
use web_repo_to_ai_url_security::PinnedTarget;

use crate::error::WebExtractorError;

/// Stream of incoming body byte chunks.
pub type BodyByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, WebExtractorError>> + Send>>;

/// Response returned by an `HttpTransport` implementation.
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// HTTP status reason text.
    pub status_text: String,
    /// Preserved response headers.
    pub headers: HashMap<String, String>,
    /// Streaming body reader.
    pub body_stream: BodyByteStream,
}

/// Pluggable HTTP transport interface.
#[async_trait]
pub trait HttpTransport: Send + Sync + std::fmt::Debug {
    /// Executes an HTTP request against a pinned network target.
    async fn execute(
        &self,
        target: &PinnedTarget,
        method: &str,
        headers: HashMap<String, String>,
        timeout: Duration,
    ) -> Result<TransportResponse, WebExtractorError>;
}

/// Production HTTP transport using `reqwest` configured for IP pinning and zero automatic redirects.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReqwestTransport;

#[async_trait]
impl HttpTransport for ReqwestTransport {
    async fn execute(
        &self,
        target: &PinnedTarget,
        method_str: &str,
        headers_map: HashMap<String, String>,
        timeout: Duration,
    ) -> Result<TransportResponse, WebExtractorError> {
        let method = Method::from_bytes(method_str.as_bytes())
            .map_err(|e| WebExtractorError::Http(format!("Invalid HTTP method: {e}")))?;

        // Build a single-request client pinned strictly to target.socket_addr()
        let mut builder = Client::builder()
            .resolve(target.host(), target.socket_addr())
            .redirect(Policy::none())
            .timeout(timeout);

        let mut req_headers = HeaderMap::new();
        for (k, v) in headers_map {
            if let (Ok(name), Ok(val)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(&v),
            ) {
                req_headers.insert(name, val);
            }
        }
        builder = builder.default_headers(req_headers);

        let client = builder.build().map_err(WebExtractorError::from)?;
        let response = client
            .request(method, target.url.as_str())
            .send()
            .await
            .map_err(WebExtractorError::from)?;

        let status = response.status().as_u16();
        let status_text = response
            .status()
            .canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let mut resp_headers = HashMap::new();
        for (name, val) in response.headers() {
            if let Ok(v_str) = val.to_str() {
                resp_headers.insert(name.as_str().to_ascii_lowercase(), v_str.to_string());
            }
        }

        let raw_stream = response.bytes_stream();
        let mapped_stream = raw_stream.map(|item| item.map_err(WebExtractorError::from));
        let boxed_stream: BodyByteStream = Box::pin(mapped_stream);

        Ok(TransportResponse {
            status,
            status_text,
            headers: resp_headers,
            body_stream: boxed_stream,
        })
    }
}

/// In-memory mock transport for deterministic testing and adversarial redirect scenarios.
#[derive(Debug, Default, Clone)]
pub struct MockHttpTransport {
    /// Map from canonical URL to configured mock responses.
    responses: Arc<Mutex<HashMap<String, MockResponse>>>,
    /// Configured simulated response delays.
    delays: Arc<Mutex<HashMap<String, Duration>>>,
    /// Configured simulated inter-chunk streaming delays.
    chunk_delays: Arc<Mutex<HashMap<String, Duration>>>,
}

/// Pre-configured mock HTTP response definition.
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// HTTP status code.
    pub status: u16,
    /// Status reason phrase.
    pub status_text: String,
    /// Preserved response headers.
    pub headers: HashMap<String, String>,
    /// Response body payload.
    pub body: Vec<u8>,
}

impl MockHttpTransport {
    /// Constructs a new empty `MockHttpTransport`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures a mock response for a given URL string.
    pub fn set_response(&self, url: &str, response: MockResponse) {
        let mut map = self.responses.lock().expect("lock not poisoned");
        map.insert(url.to_string(), response);
    }

    /// Helper to configure a simple 200 OK HTML response.
    pub fn set_html(&self, url: &str, html: &str) {
        let mut headers = HashMap::new();
        headers.insert(
            "content-type".to_string(),
            "text/html; charset=utf-8".to_string(),
        );
        headers.insert("content-length".to_string(), html.len().to_string());
        self.set_response(
            url,
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: html.as_bytes().to_vec(),
            },
        );
    }

    /// Helper to configure a redirect response (e.g. 301, 302).
    pub fn set_redirect(&self, from_url: &str, to_location: &str, status: u16) {
        let mut headers = HashMap::new();
        headers.insert("location".to_string(), to_location.to_string());
        headers.insert("content-length".to_string(), "0".to_string());
        self.set_response(
            from_url,
            MockResponse {
                status,
                status_text: "Found".to_string(),
                headers,
                body: Vec::new(),
            },
        );
    }

    /// Configures a simulated initial response delay for testing timeouts.
    pub fn set_delay(&self, url: &str, delay: Duration) {
        let mut map = self.delays.lock().expect("lock not poisoned");
        map.insert(url.to_string(), delay);
    }

    /// Configures a simulated inter-chunk streaming delay for testing chunk timeouts.
    pub fn set_chunk_delay(&self, url: &str, delay: Duration) {
        let mut map = self.chunk_delays.lock().expect("lock not poisoned");
        map.insert(url.to_string(), delay);
    }
}

#[async_trait]
impl HttpTransport for MockHttpTransport {
    async fn execute(
        &self,
        target: &PinnedTarget,
        _method: &str,
        _headers: HashMap<String, String>,
        _timeout: Duration,
    ) -> Result<TransportResponse, WebExtractorError> {
        let url_str = target.url.as_str();

        let initial_delay = self
            .delays
            .lock()
            .expect("lock not poisoned")
            .get(url_str)
            .copied();

        if let Some(delay) = initial_delay {
            tokio::time::sleep(delay).await;
        }

        let chunk_delay = self
            .chunk_delays
            .lock()
            .expect("lock not poisoned")
            .get(url_str)
            .copied();

        let map = self.responses.lock().expect("lock not poisoned");

        if let Some(mock) = map.get(url_str) {
            let boxed_stream: BodyByteStream = if let Some(cd) = chunk_delay {
                let half = mock.body.len() / 2;
                let part1 = Bytes::from(mock.body[..half].to_vec());
                let part2 = Bytes::from(mock.body[half..].to_vec());
                let stream = futures::stream::unfold(0, move |step| {
                    let p1 = part1.clone();
                    let p2 = part2.clone();
                    async move {
                        match step {
                            0 => Some((Ok(p1), 1)),
                            1 => {
                                tokio::time::sleep(cd).await;
                                Some((Ok(p2), 2))
                            }
                            _ => None,
                        }
                    }
                });
                Box::pin(stream)
            } else {
                let body_bytes = Bytes::from(mock.body.clone());
                let stream = futures::stream::once(async move { Ok(body_bytes) });
                Box::pin(stream)
            };

            Ok(TransportResponse {
                status: mock.status,
                status_text: mock.status_text.clone(),
                headers: mock.headers.clone(),
                body_stream: boxed_stream,
            })
        } else {
            Err(WebExtractorError::Network {
                details: format!("No mock HTTP response registered for '{url_str}'"),
            })
        }
    }
}
