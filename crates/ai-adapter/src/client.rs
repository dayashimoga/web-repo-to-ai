//! Shared HTTP client configuration, headers, and error translation.

use crate::error::AiAdapterError;
use reqwest::header::{
    HeaderMap, HeaderName, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT,
};
use reqwest::{Client, Response, StatusCode};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration options for connecting to an AI provider HTTP endpoint.
#[derive(Debug, Clone)]
pub struct AiClientConfig {
    /// Base URL of the provider endpoint (e.g. `http://localhost:11434` or `https://api.openai.com/v1`).
    pub base_url: String,
    /// Optional API key or bearer token.
    pub api_key: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum request retries for transient errors.
    pub max_retries: usize,
    /// Optional organization identifier (OpenAI / Azure).
    pub organization: Option<String>,
    /// Optional project identifier (OpenAI / Google).
    pub project: Option<String>,
    /// Additional custom HTTP headers.
    pub custom_headers: HashMap<String, String>,
}

impl AiClientConfig {
    /// Create a new client configuration with default settings.
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut base = base_url.into();
        // Trim trailing slashes for clean URI concatenation
        while base.ends_with('/') {
            base.pop();
        }

        Self {
            base_url: base,
            api_key: None,
            timeout_secs: 60,
            max_retries: 2,
            organization: None,
            project: None,
            custom_headers: HashMap::new(),
        }
    }

    /// Set API key.
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set request timeout in seconds.
    pub fn with_timeout_secs(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set organization header.
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Add a custom header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    /// Construct a configured reqwest HTTP client.
    pub fn build_client(&self) -> Result<Client, AiAdapterError> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("web-repo-to-ai-adapter/0.1.0"),
        );

        if let Some(ref key) = self.api_key {
            let auth_val = format!("Bearer {}", key.trim());
            if let Ok(hv) = HeaderValue::from_str(&auth_val) {
                headers.insert(AUTHORIZATION, hv);
            }
        }

        if let Some(ref org) = self.organization {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(b"openai-organization"),
                HeaderValue::from_str(org),
            ) {
                headers.insert(hn, hv);
            }
        }

        if let Some(ref proj) = self.project {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(b"openai-project"),
                HeaderValue::from_str(proj),
            ) {
                headers.insert(hn, hv);
            }
        }

        for (k, v) in &self.custom_headers {
            if let (Ok(hn), Ok(hv)) = (
                HeaderName::from_bytes(k.as_bytes()),
                HeaderValue::from_str(v),
            ) {
                headers.insert(hn, hv);
            }
        }

        Client::builder()
            .timeout(Duration::from_secs(self.timeout_secs))
            .default_headers(headers)
            .build()
            .map_err(|e| AiAdapterError::Internal(format!("Failed to build HTTP client: {e}")))
    }

    /// Format a full URL given an API subpath.
    pub fn url(&self, path: &str) -> String {
        let clean_path = path.strip_prefix('/').unwrap_or(path);
        format!("{}/{}", self.base_url, clean_path)
    }
}

/// Helper translating reqwest network errors into categorized `AiAdapterError`.
pub fn map_reqwest_error(err: reqwest::Error) -> AiAdapterError {
    if err.is_timeout() {
        AiAdapterError::Timeout(format!("Request timed out: {err}"))
    } else if err.is_connect() {
        AiAdapterError::NetworkError(format!("Connection failed or host unreachable: {err}"))
    } else {
        AiAdapterError::NetworkError(err.to_string())
    }
}

/// Helper translating HTTP error status responses into structured `AiAdapterError`.
pub async fn map_http_response_error(resp: Response) -> AiAdapterError {
    let status = resp.status();
    let status_code = status.as_u16();

    // Check for standard Retry-After header on 429
    let retry_after = if status == StatusCode::TOO_MANY_REQUESTS {
        resp.headers()
            .get("retry-after")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
    } else {
        None
    };

    let body_text = resp
        .text()
        .await
        .unwrap_or_else(|_| "<failed to read response body>".to_string());

    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            AiAdapterError::AuthenticationRequired(format!(
                "HTTP {status_code}: Authentication failed. Check your API key. Detail: {body_text}"
            ))
        }
        StatusCode::TOO_MANY_REQUESTS => AiAdapterError::RateLimitExceeded {
            retry_after_secs: retry_after,
            message: format!("HTTP 429 Rate Limit Exceeded: {body_text}"),
        },
        StatusCode::NOT_FOUND => AiAdapterError::ModelNotFound {
            model: "unknown".to_string(),
            message: format!("HTTP 404 Endpoint or Model Not Found: {body_text}"),
        },
        StatusCode::PAYLOAD_TOO_LARGE => AiAdapterError::ContextLengthExceeded {
            requested_tokens: 0,
            max_tokens: 0,
            message: format!("HTTP 413 Payload Too Large: {body_text}"),
        },
        StatusCode::BAD_REQUEST => {
            if body_text.contains("context_length_exceeded")
                || body_text.contains("maximum context length")
            {
                AiAdapterError::ContextLengthExceeded {
                    requested_tokens: 0,
                    max_tokens: 0,
                    message: format!("Context length exceeded: {body_text}"),
                }
            } else if body_text.contains("model_not_found") || body_text.contains("does not exist")
            {
                AiAdapterError::ModelNotFound {
                    model: "unknown".to_string(),
                    message: format!("Model not found: {body_text}"),
                }
            } else {
                AiAdapterError::InvalidRequest(format!("HTTP 400 Bad Request: {body_text}"))
            }
        }
        _ if status.is_server_error() => AiAdapterError::ServerUnavailable {
            status_code: Some(status_code),
            message: format!("HTTP {status_code} Server Error: {body_text}"),
        },
        _ => AiAdapterError::Internal(format!("HTTP {status_code} Unexpected Error: {body_text}")),
    }
}
