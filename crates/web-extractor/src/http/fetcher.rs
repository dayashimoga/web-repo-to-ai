//! # Web Fetcher and Secure Acquisition Engine
//!
//! Executes hop-by-hop HTTP acquisition with IP pinning, streaming body buffering,
//! redirect revalidation, and cycle mitigation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures::StreamExt;
use web_repo_to_ai_url_security::{SecureDnsResolver, UrlSecurityPolicy};

use super::redirect::RedirectTracker;
use super::response::FetchedResponse;
use super::transport::{HttpTransport, ReqwestTransport};
use crate::error::WebExtractorError;
use crate::url::WebTarget;

/// Default maximum allowable HTTP redirect hops before aborting.
pub const DEFAULT_MAX_REDIRECTS: usize = 10;

/// Default response body ceiling (15 MB, well within 16 GB system baseline).
pub const DEFAULT_MAX_BODY_BYTES: usize = 15 * 1024 * 1024;

/// Default request timeout duration.
pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Default User-Agent header string.
pub const DEFAULT_USER_AGENT: &str =
    "web-repo-to-ai/0.1.0 (+https://github.com/dayashimoga/web-repo-to-ai)";

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
    /// Maximum body payload in bytes before truncation/error.
    max_body_bytes: usize,
    /// Request timeout duration.
    timeout: Duration,
    /// Configured User-Agent header value.
    user_agent: String,
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
            max_body_bytes: DEFAULT_MAX_BODY_BYTES,
            timeout: DEFAULT_REQUEST_TIMEOUT,
            user_agent: DEFAULT_USER_AGENT.to_string(),
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

    /// Sets the maximum allowable response body byte ceiling.
    #[must_use]
    pub fn with_max_body_bytes(mut self, max: usize) -> Self {
        self.max_body_bytes = max;
        self
    }

    /// Sets the request timeout duration.
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the User-Agent header string.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
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

    /// Executes an HTTP acquisition chain starting from `target`.
    ///
    /// ## Hop-by-Hop Secure Acquisition Flow:
    /// 1. Initializes `RedirectTracker` with the starting target URL.
    /// 2. Resolves and pins the current URL via `SecureDnsResolver` (guaranteeing no private IP/metadata).
    /// 3. Injects standard headers (`Host`, `User-Agent`, `Accept`).
    /// 4. Executes the HTTP request through the pinned transport.
    /// 5. If response is a redirect (`301`, `302`, `303`, `307`, `308`):
    ///    - Extracts `Location` header.
    ///    - Revalidates redirect target URL against policy and checks for cycles via `check_and_record`.
    ///    - Updates `current_url` and loops back to step 2!
    /// 6. If response is terminal:
    ///    - Streams body chunks up to `max_body_bytes`.
    ///    - If payload exceeds ceiling, aborts with `WebExtractorError::PayloadTooLarge`.
    ///    - Packages complete response and redirect history into `FetchedResponse`.
    pub async fn fetch(&self, target: &WebTarget) -> Result<FetchedResponse, WebExtractorError> {
        let mut current_url = target.url.clone();
        let mut redirect_tracker = RedirectTracker::new(self.max_redirects);
        redirect_tracker.record_initial(&current_url);

        loop {
            // Step 2: Resolve and pin current URL destination socket
            let pinned_target = self.dns_resolver.resolve_and_pin(&current_url).await?;

            // Step 3: Prepare request headers
            let mut req_headers = HashMap::new();
            req_headers.insert("user-agent".to_string(), self.user_agent.clone());
            req_headers.insert(
                "accept".to_string(),
                "text/html,application/xhtml+xml,application/json;q=0.9,*/*;q=0.8".to_string(),
            );
            req_headers.insert("host".to_string(), pinned_target.host().to_string());

            // Step 4: Execute HTTP request
            let mut transport_resp = self
                .transport
                .execute(&pinned_target, "GET", req_headers, self.timeout)
                .await?;

            let status = transport_resp.status;

            // Step 5: Check for redirect responses
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

            // Step 6: Terminal response - stream body with memory ceiling
            let mut accumulated_bytes = Vec::new();
            while let Some(chunk_res) = transport_resp.body_stream.next().await {
                let chunk = chunk_res?;
                if accumulated_bytes.len() + chunk.len() > self.max_body_bytes {
                    return Err(WebExtractorError::PayloadTooLarge {
                        actual: accumulated_bytes.len() + chunk.len(),
                        maximum: self.max_body_bytes,
                    });
                }
                accumulated_bytes.extend_from_slice(&chunk);
            }

            let content_type = transport_resp.headers.get("content-type").cloned();
            let content_length = transport_resp
                .headers
                .get("content-length")
                .and_then(|val| val.parse::<u64>().ok());
            let etag = transport_resp.headers.get("etag").cloned();
            let last_modified = transport_resp.headers.get("last-modified").cloned();

            return Ok(FetchedResponse {
                url: current_url,
                initial_url: target.url.clone(),
                status,
                status_text: transport_resp.status_text,
                headers: transport_resp.headers,
                content_type,
                content_length,
                etag,
                last_modified,
                redirect_history: redirect_tracker.into_hops(),
                body: accumulated_bytes,
                pinned_ip: pinned_target.pinned_ip(),
                fetched_at: Utc::now(),
            });
        }
    }
}
