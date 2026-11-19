//! # Web Fetcher and Secure Acquisition Engine
//!
//! Executes hop-by-hop HTTP acquisition with IP pinning, bounded streaming body ingestion,
//! redirect revalidation, decompression bomb defense, and timeout controls.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures::StreamExt;
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy};

use super::redirect::RedirectTracker;
use super::response::FetchedResponse;
use super::transport::{HttpTransport, ReqwestTransport};
use super::validator::{FetchOutcome, HttpValidator};
use crate::compression::{BodyLimits, BoundedDecompressor, ContentEncoding};
use crate::error::WebExtractorError;
use crate::url::WebTarget;

/// Default maximum allowable HTTP redirect hops before aborting.
pub const DEFAULT_MAX_REDIRECTS: usize = 10;

/// Default response body ceiling (10 MB).
pub const DEFAULT_MAX_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Default request timeout duration.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Default connect timeout duration.
pub const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Default inactivity timeout between streamed chunks.
pub const DEFAULT_CHUNK_READ_TIMEOUT: Duration = Duration::from_secs(10);

/// Default overall request timeout duration covering DNS, TLS, redirects, and body streaming.
pub const DEFAULT_TOTAL_TIMEOUT: Duration = Duration::from_secs(30);

/// Default User-Agent header string.
pub const DEFAULT_USER_AGENT: &str =
    "web-repo-to-ai/0.1.0 (+https://github.com/dayashimoga/web-repo-to-ai)";

/// Default Accept-Encoding header string.
pub const DEFAULT_ACCEPT_ENCODING: &str = "gzip, deflate, br, zstd, identity";

/// Secure web content fetcher orchestrating DNS pinning, HTTP transport, and redirect revalidation.
#[derive(Debug, Clone)]
pub struct WebFetcher {
    /// URL security policy enforcing scheme and SSRF constraints.
    policy: UrlSecurityPolicy,
    /// Secure DNS resolution engine providing IP pinning and all-record validation.
    dns_resolver: SecureDnsResolver,
    /// Pluggable HTTP transport implementation.
    transport: Arc<dyn HttpTransport>,
    /// Maximum permitted redirect transitions.
    max_redirects: usize,
    /// Configurable body size ceilings and compression bomb guards.
    body_limits: BodyLimits,
    /// Connect / transport timeout.
    connect_timeout: Duration,
    /// Inactivity timeout while waiting for streamed response chunks.
    chunk_read_timeout: Duration,
    /// Total overall operation timeout covering all hops and streaming.
    total_timeout: Duration,
    /// Configured User-Agent header value.
    user_agent: String,
    /// Configured Accept-Encoding header value.
    accept_encoding: String,
}

impl WebFetcher {
    /// Constructs a `WebFetcher` with default `ReqwestTransport` and standard defaults.
    #[must_use]
    pub fn new(policy: UrlSecurityPolicy, dns_resolver: SecureDnsResolver) -> Self {
        Self {
            policy,
            dns_resolver,
            transport: Arc::new(ReqwestTransport),
            max_redirects: DEFAULT_MAX_REDIRECTS,
            body_limits: BodyLimits::default(),
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            chunk_read_timeout: DEFAULT_CHUNK_READ_TIMEOUT,
            total_timeout: DEFAULT_TOTAL_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_string(),
            accept_encoding: DEFAULT_ACCEPT_ENCODING.to_string(),
        }
    }

    /// Sets a custom `HttpTransport` implementation (e.g. `MockHttpTransport` for testing).
    #[must_use]
    pub fn with_transport(mut self, transport: Arc<dyn HttpTransport>) -> Self {
        self.transport = transport;
        self
    }

    /// Sets the maximum allowable redirect hops.
    #[must_use]
    pub fn with_max_redirects(mut self, max: usize) -> Self {
        self.max_redirects = max;
        self
    }

    /// Sets custom body and decompression limits.
    #[must_use]
    pub fn with_body_limits(mut self, limits: BodyLimits) -> Self {
        self.body_limits = limits;
        self
    }

    /// Sets the maximum allowable raw response body byte ceiling.
    #[must_use]
    pub fn with_max_body_bytes(mut self, max: usize) -> Self {
        self.body_limits.max_raw_body_bytes = max;
        self
    }

    /// Sets the connect and overall operation timeout.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self.total_timeout = timeout;
        self
    }

    /// Sets the overall operation timeout covering all hops and streaming.
    #[must_use]
    pub fn with_total_timeout(mut self, timeout: Duration) -> Self {
        self.total_timeout = timeout;
        self
    }

    /// Sets the chunk read / inactivity timeout.
    #[must_use]
    pub fn with_chunk_read_timeout(mut self, timeout: Duration) -> Self {
        self.chunk_read_timeout = timeout;
        self
    }

    /// Sets the User-Agent header string.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets the Accept-Encoding header string.
    #[must_use]
    pub fn with_accept_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.accept_encoding = encoding.into();
        self
    }

    /// Returns a reference to the active `UrlSecurityPolicy`.
    #[must_use]
    pub fn policy(&self) -> &UrlSecurityPolicy {
        &self.policy
    }

    /// Returns a reference to the active `SecureDnsResolver`.
    #[must_use]
    pub fn dns_resolver(&self) -> &SecureDnsResolver {
        &self.dns_resolver
    }

    /// Returns a reference to the active `BodyLimits`.
    #[must_use]
    pub fn body_limits(&self) -> &BodyLimits {
        &self.body_limits
    }

    /// Returns the maximum raw body byte ceiling.
    #[must_use]
    pub fn max_body_bytes(&self) -> usize {
        self.body_limits.max_raw_body_bytes
    }

    /// Executes an HTTP acquisition chain starting from `target`.
    ///
    /// The entire operation is bounded by `total_timeout`.
    pub async fn fetch(&self, target: &WebTarget) -> Result<FetchedResponse, WebExtractorError> {
        match self.fetch_conditional(target, None).await? {
            FetchOutcome::Modified(resp) => Ok(resp),
            FetchOutcome::NotModified {
                url,
                status,
                headers,
                validator,
                fetched_at,
            } => {
                let etag = validator
                    .as_ref()
                    .and_then(|v| v.etag.as_ref().map(|e| e.raw.clone()));
                let last_modified = validator.as_ref().and_then(|v| v.raw_last_modified.clone());
                Ok(FetchedResponse {
                    url: url.clone(),
                    initial_url: url,
                    status,
                    status_text: "Not Modified".to_string(),
                    headers,
                    content_type: None,
                    content_length: Some(0),
                    content_encoding: None,
                    raw_body_size: 0,
                    etag,
                    last_modified,
                    redirect_history: Vec::new(),
                    body: Vec::new(),
                    pinned_ip: std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                    fetched_at,
                })
            }
        }
    }

    /// Executes an HTTP acquisition chain with optional conditional validation headers.
    ///
    /// If `validator` is provided with an ETag or Last-Modified timestamp,
    /// `If-None-Match` and `If-Modified-Since` headers are transmitted.
    ///
    /// If the origin returns `304 Not Modified`, `FetchOutcome::NotModified` is returned
    /// without downloading or decompressing content.
    pub async fn fetch_conditional(
        &self,
        target: &WebTarget,
        validator: Option<&HttpValidator>,
    ) -> Result<FetchOutcome, WebExtractorError> {
        tokio::time::timeout(
            self.total_timeout,
            self.fetch_conditional_internal(target, validator),
        )
        .await
        .map_err(|_| WebExtractorError::Timeout {
            details: format!(
                "Total fetch duration exceeded configured limit of {:?}",
                self.total_timeout
            ),
        })?
    }

    /// Internal execution loop for conditional fetch operation.
    async fn fetch_conditional_internal(
        &self,
        target: &WebTarget,
        validator: Option<&HttpValidator>,
    ) -> Result<FetchOutcome, WebExtractorError> {
        let mut current_url = target.url.clone();
        let mut redirect_tracker = RedirectTracker::new(self.max_redirects);
        redirect_tracker.record_initial(&current_url);

        loop {
            // Step 1: Resolve and pin current URL destination socket
            let pinned_target = self.dns_resolver.resolve_and_pin(&current_url).await?;

            // Step 2: Prepare request headers
            let mut req_headers = HashMap::new();
            req_headers.insert("user-agent".to_string(), self.user_agent.clone());
            req_headers.insert(
                "accept".to_string(),
                "text/html,application/xhtml+xml,application/json;q=0.9,*/*;q=0.8".to_string(),
            );
            req_headers.insert("accept-encoding".to_string(), self.accept_encoding.clone());
            req_headers.insert("host".to_string(), pinned_target.host().to_string());

            // Inject conditional validation headers on initial request hop
            if redirect_tracker.hop_count() == 0 {
                if let Some(val) = validator {
                    for (header_k, header_v) in val.conditional_headers() {
                        req_headers.insert(header_k, header_v);
                    }
                }
            }

            // Step 3: Execute HTTP request
            let mut transport_resp = self
                .transport
                .execute(&pinned_target, "GET", req_headers, self.connect_timeout)
                .await?;

            let status = transport_resp.status;

            // Step 4: Check for 304 Not Modified before handling general redirect range
            if status == 304 {
                let validator_meta =
                    HttpValidator::from_headers(&transport_resp.headers, Utc::now());
                return Ok(FetchOutcome::NotModified {
                    url: current_url,
                    status: 304,
                    headers: transport_resp.headers,
                    validator: Some(validator_meta),
                    fetched_at: Utc::now(),
                });
            }

            // Step 5: Check for redirect responses (300..=399)
            if (300..=399).contains(&status) {
                let location =
                    transport_resp
                        .headers
                        .get("location")
                        .cloned()
                        .ok_or_else(|| WebExtractorError::MissingLocationHeader {
                            status,
                            url: current_url.to_string(),
                        })?;

                // Revalidate redirect destination and check for cycles
                let next_url = redirect_tracker.check_and_record(
                    &current_url,
                    &location,
                    status,
                    pinned_target.pinned_ip(),
                    &self.policy,
                )?;

                current_url = next_url;
                continue;
            }

            // Step 6: Terminal response
            // Early exit if declared Content-Length exceeds raw ceiling
            let declared_content_length = transport_resp
                .headers
                .get("content-length")
                .and_then(|val| val.parse::<u64>().ok());

            if let Some(content_length) = declared_content_length {
                if content_length > self.body_limits.max_raw_body_bytes as u64 {
                    return Err(WebExtractorError::PayloadTooLarge {
                        actual: content_length as usize,
                        maximum: self.body_limits.max_raw_body_bytes,
                    });
                }
            }

            // Step 7: Stream body chunks with chunk inactivity timeout
            let mut accumulated_raw = Vec::new();
            loop {
                let chunk_res = tokio::time::timeout(
                    self.chunk_read_timeout,
                    transport_resp.body_stream.next(),
                )
                .await;

                let chunk_opt = match chunk_res {
                    Ok(opt) => opt,
                    Err(_) => {
                        return Err(WebExtractorError::Timeout {
                            details: format!(
                                "Inactivity timeout exceeded ({:?}) waiting for next response body chunk",
                                self.chunk_read_timeout
                            ),
                        });
                    }
                };

                match chunk_opt {
                    Some(Ok(chunk)) => {
                        if accumulated_raw.len() + chunk.len() > self.body_limits.max_raw_body_bytes
                        {
                            return Err(WebExtractorError::PayloadTooLarge {
                                actual: accumulated_raw.len() + chunk.len(),
                                maximum: self.body_limits.max_raw_body_bytes,
                            });
                        }
                        accumulated_raw.extend_from_slice(&chunk);
                    }
                    Some(Err(err)) => return Err(err),
                    None => break, // Stream exhausted
                }
            }

            // Step 8: Validate Content-Length if declared and enforced
            if let Some(expected) = declared_content_length {
                let actual = accumulated_raw.len() as u64;
                if self.body_limits.enforce_content_length && actual != expected {
                    return Err(WebExtractorError::ContentLengthMismatch { expected, actual });
                }
            }

            // Step 9: Bounded decompression
            let content_encoding = transport_resp.headers.get("content-encoding").cloned();
            let raw_body_size = accumulated_raw.len();

            let decompressed_body = if let Some(encoding_header) = &content_encoding {
                let encodings = ContentEncoding::parse_header(encoding_header)?;
                BoundedDecompressor::decompress_chain(
                    &accumulated_raw,
                    &encodings,
                    &self.body_limits,
                )?
            } else {
                if accumulated_raw.len() > self.body_limits.max_decompressed_bytes {
                    return Err(WebExtractorError::PayloadTooLarge {
                        actual: accumulated_raw.len(),
                        maximum: self.body_limits.max_decompressed_bytes,
                    });
                }
                accumulated_raw
            };

            let content_type = transport_resp.headers.get("content-type").cloned();
            let etag = transport_resp.headers.get("etag").cloned();
            let last_modified = transport_resp.headers.get("last-modified").cloned();

            return Ok(FetchOutcome::Modified(FetchedResponse {
                url: current_url,
                initial_url: target.url.clone(),
                status,
                status_text: transport_resp.status_text,
                headers: transport_resp.headers,
                content_type,
                content_length: declared_content_length,
                content_encoding,
                raw_body_size,
                etag,
                last_modified,
                redirect_history: redirect_tracker.into_hops(),
                body: decompressed_body,
                pinned_ip: pinned_target.pinned_ip(),
                fetched_at: Utc::now(),
            }));
        }
    }
}
