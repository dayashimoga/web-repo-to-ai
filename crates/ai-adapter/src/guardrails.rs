//! Opt-in policy guardrails and network boundary controls for AI provider adapters.
//!
//! Enforces Section 3.4 of the GLOBAL_ENGINEERING_CONTRACT:
//! "Zero Mandatory External AI / Paid Dependencies: All core document extraction,
//! normalization, deduplication, chunking, and export logic execute locally on CPU
//! without AI assistance. AI provider adapters serve solely as optional, opt-in
//! post-processing extensions. Core extraction MUST ALWAYS succeed without AI keys."

use crate::error::AiAdapterError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

/// Permitted execution boundary mode for AI adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AiExecutionMode {
    /// Completely disabled. No outbound AI calls are allowed under any circumstances.
    /// All invocations seamlessly use zero-key CPU fallbacks.
    #[default]
    Disabled,

    /// Only local inference daemons or container instances are allowed (e.g. Ollama,
    /// local vLLM, LM Studio). Remote cloud egress is blocked by policy.
    LocalOnly,

    /// Remote cloud providers (e.g. OpenAI, Anthropic, hosted vLLM) are allowed,
    /// requiring TLS encryption and prohibiting cloud metadata addresses.
    AllowRemote,
}

/// Comprehensive policy governing AI provider invocation and network egress boundaries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOptInPolicy {
    /// Current execution boundary mode.
    pub execution_mode: AiExecutionMode,

    /// Whether remote endpoints require an API key to be supplied.
    pub require_api_key_for_remote: bool,

    /// Whether plain unencrypted HTTP is permitted for remote endpoints (strongly discouraged).
    pub allow_http_for_remote: bool,

    /// Additional hosts explicitly permitted even in LocalOnly mode (e.g. internal enterprise gateways).
    pub allowed_hosts: HashSet<String>,

    /// Hosts explicitly blocked regardless of execution mode.
    pub blocked_hosts: HashSet<String>,

    /// Maximum permissible timeout in seconds for any single AI provider call.
    pub max_timeout_secs: u64,

    /// Whether to seamlessly fallback to deterministic zero-key CPU implementations on errors or disabling.
    pub allow_zero_key_fallback: bool,
}

impl Default for AiOptInPolicy {
    /// Secure default: AI execution is strictly disabled, and zero-key fallback is enabled.
    fn default() -> Self {
        Self::disabled()
    }
}

impl AiOptInPolicy {
    /// Construct a policy with execution disabled (zero-key fallback mode).
    #[must_use]
    pub fn disabled() -> Self {
        Self {
            execution_mode: AiExecutionMode::Disabled,
            require_api_key_for_remote: true,
            allow_http_for_remote: false,
            allowed_hosts: HashSet::new(),
            blocked_hosts: HashSet::new(),
            max_timeout_secs: 60,
            allow_zero_key_fallback: true,
        }
    }

    /// Construct a policy permitting only local inference runtimes (zero cloud egress).
    #[must_use]
    pub fn local_only() -> Self {
        Self {
            execution_mode: AiExecutionMode::LocalOnly,
            require_api_key_for_remote: true,
            allow_http_for_remote: false,
            allowed_hosts: HashSet::new(),
            blocked_hosts: HashSet::new(),
            max_timeout_secs: 120,
            allow_zero_key_fallback: true,
        }
    }

    /// Construct a policy permitting remote cloud AI providers over TLS.
    #[must_use]
    pub fn allow_remote() -> Self {
        Self {
            execution_mode: AiExecutionMode::AllowRemote,
            require_api_key_for_remote: true,
            allow_http_for_remote: false,
            allowed_hosts: HashSet::new(),
            blocked_hosts: HashSet::new(),
            max_timeout_secs: 120,
            allow_zero_key_fallback: true,
        }
    }

    /// Discover and construct an opt-in policy from standard environment variables:
    /// - `AI_ENABLED` ("1", "true", "yes", "on")
    /// - `AI_EXECUTION_MODE` ("disabled", "local_only", "allow_remote")
    /// - `AI_ALLOW_REMOTE` ("1", "true")
    #[must_use]
    pub fn from_env() -> Self {
        let is_enabled = std::env::var("AI_ENABLED")
            .map(|v| {
                let s = v.to_ascii_lowercase();
                s == "1" || s == "true" || s == "yes" || s == "on"
            })
            .unwrap_or(false);

        if !is_enabled {
            return Self::disabled();
        }

        let mode = if let Ok(val) = std::env::var("AI_EXECUTION_MODE") {
            match val.to_ascii_lowercase().as_str() {
                "local_only" | "local" => AiExecutionMode::LocalOnly,
                "allow_remote" | "remote" | "cloud" => AiExecutionMode::AllowRemote,
                _ => AiExecutionMode::Disabled,
            }
        } else {
            let allow_remote = std::env::var("AI_ALLOW_REMOTE")
                .map(|v| {
                    let s = v.to_ascii_lowercase();
                    s == "1" || s == "true" || s == "yes" || s == "on"
                })
                .unwrap_or(false);

            if allow_remote {
                AiExecutionMode::AllowRemote
            } else {
                AiExecutionMode::LocalOnly
            }
        };

        match mode {
            AiExecutionMode::Disabled => Self::disabled(),
            AiExecutionMode::LocalOnly => Self::local_only(),
            AiExecutionMode::AllowRemote => Self::allow_remote(),
        }
    }

    /// Set execution mode.
    #[must_use]
    pub fn with_execution_mode(mut self, mode: AiExecutionMode) -> Self {
        self.execution_mode = mode;
        self
    }

    /// Add an explicitly allowed host.
    #[must_use]
    pub fn with_allowed_host(mut self, host: impl Into<String>) -> Self {
        self.allowed_hosts.insert(host.into().to_ascii_lowercase());
        self
    }

    /// Add an explicitly blocked host.
    #[must_use]
    pub fn with_blocked_host(mut self, host: impl Into<String>) -> Self {
        self.blocked_hosts.insert(host.into().to_ascii_lowercase());
        self
    }

    /// Set whether zero-key fallback is enabled.
    #[must_use]
    pub fn with_allow_zero_key_fallback(mut self, allow: bool) -> Self {
        self.allow_zero_key_fallback = allow;
        self
    }

    /// Set whether remote endpoints require an API key.
    #[must_use]
    pub fn with_require_api_key_for_remote(mut self, require: bool) -> Self {
        self.require_api_key_for_remote = require;
        self
    }

    /// Whether AI features are actively opted-in.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.execution_mode != AiExecutionMode::Disabled
    }

    /// Check if a given hostname represents a local loopback, LAN, or local container runtime.
    #[must_use]
    pub fn is_local_host(host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        let clean = h.trim_matches('[').trim_matches(']');

        clean == "localhost"
            || clean == "127.0.0.1"
            || clean == "::1"
            || clean == "0.0.0.0"
            || clean.starts_with("127.")
            || clean == "host.docker.internal"
            || clean == "host.containers.internal"
            || clean == "ollama"
            || clean == "vllm"
            || clean == "localai"
            || clean.ends_with(".local")
            || clean.ends_with(".internal")
    }

    /// Check if a given host is a known cloud metadata IP or address.
    #[must_use]
    pub fn is_cloud_metadata(host: &str) -> bool {
        let h = host.to_ascii_lowercase();
        let clean = h.trim_matches('[').trim_matches(']');

        clean == "169.254.169.254"
            || clean.starts_with("169.254.")
            || clean == "metadata.google.internal"
            || clean == "100.100.100.200"
    }

    /// Validate an outbound target endpoint URL against this policy.
    ///
    /// # Errors
    /// Returns [`AiAdapterError`] if the endpoint violates security or policy guardrails:
    /// - [`AiAdapterError::ProviderDisabled`] if execution mode is disabled.
    /// - [`AiAdapterError::PolicyViolation`] if remote egress is blocked or points to cloud metadata.
    /// - [`AiAdapterError::AuthenticationRequired`] if remote endpoint requires a missing API key.
    pub fn validate_endpoint(
        &self,
        endpoint_url: &str,
        has_api_key: bool,
    ) -> Result<(), AiAdapterError> {
        if self.execution_mode == AiExecutionMode::Disabled {
            return Err(AiAdapterError::ProviderDisabled(
                "AI execution is disabled by policy. Enable via configuration or rely on zero-key fallback."
                    .to_string(),
            ));
        }

        let parsed = Url::parse(endpoint_url).map_err(|e| {
            AiAdapterError::InvalidRequest(format!(
                "Invalid provider base URL '{endpoint_url}': {e}"
            ))
        })?;

        let host = parsed
            .host_str()
            .ok_or_else(|| {
                AiAdapterError::InvalidRequest(format!(
                    "Base URL '{endpoint_url}' has no host component"
                ))
            })?
            .to_ascii_lowercase();

        let scheme = parsed.scheme().to_ascii_lowercase();

        // 1. Check explicit blocklist
        if self.blocked_hosts.contains(&host) {
            return Err(AiAdapterError::PolicyViolation(format!(
                "Host '{host}' is in the blocked hosts policy list"
            )));
        }

        // 2. Strict cloud metadata protection
        if Self::is_cloud_metadata(&host) {
            return Err(AiAdapterError::PolicyViolation(format!(
                "Outbound connection to cloud metadata service at '{host}' is strictly forbidden"
            )));
        }

        let is_local = Self::is_local_host(&host) || self.allowed_hosts.contains(&host);

        // 3. Enforce execution mode boundary
        match self.execution_mode {
            AiExecutionMode::Disabled => unreachable!(),
            AiExecutionMode::LocalOnly => {
                if !is_local {
                    return Err(AiAdapterError::PolicyViolation(format!(
                        "Connection to remote AI provider at '{host}' is blocked by LocalOnly execution policy"
                    )));
                }
            }
            AiExecutionMode::AllowRemote => {
                if !is_local {
                    if scheme != "https" && !self.allow_http_for_remote {
                        return Err(AiAdapterError::PolicyViolation(format!(
                            "Remote AI provider at '{endpoint_url}' must use HTTPS TLS encryption"
                        )));
                    }

                    if self.require_api_key_for_remote && !has_api_key {
                        return Err(AiAdapterError::AuthenticationRequired(format!(
                            "Remote AI provider at '{host}' requires an API key under active policy"
                        )));
                    }
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_by_default() {
        let policy = AiOptInPolicy::default();
        assert!(!policy.is_enabled());
        assert_eq!(policy.execution_mode, AiExecutionMode::Disabled);

        let err = policy
            .validate_endpoint("http://localhost:11434", false)
            .unwrap_err();
        assert!(matches!(err, AiAdapterError::ProviderDisabled(_)));
    }

    #[test]
    fn test_local_only_allows_localhost_and_containers() {
        let policy = AiOptInPolicy::local_only();
        assert!(policy.is_enabled());

        assert!(policy
            .validate_endpoint("http://localhost:11434", false)
            .is_ok());
        assert!(policy
            .validate_endpoint("http://127.0.0.1:11434", false)
            .is_ok());
        assert!(policy
            .validate_endpoint("http://[::1]:11434", false)
            .is_ok());
        assert!(policy
            .validate_endpoint("http://host.docker.internal:8000", false)
            .is_ok());
        assert!(policy
            .validate_endpoint("http://ollama:11434", false)
            .is_ok());
    }

    #[test]
    fn test_local_only_blocks_remote_endpoints() {
        let policy = AiOptInPolicy::local_only();

        let err = policy
            .validate_endpoint("https://api.openai.com/v1", true)
            .unwrap_err();
        assert!(matches!(err, AiAdapterError::PolicyViolation(_)));
        assert!(err
            .to_string()
            .contains("blocked by LocalOnly execution policy"));
    }

    #[test]
    fn test_cloud_metadata_always_blocked() {
        let policy = AiOptInPolicy::allow_remote();

        let err = policy
            .validate_endpoint("http://169.254.169.254/latest", false)
            .unwrap_err();
        assert!(matches!(err, AiAdapterError::PolicyViolation(_)));
        assert!(err.to_string().contains("metadata"));
    }

    #[test]
    fn test_remote_requires_https_and_key() {
        let policy = AiOptInPolicy::allow_remote();

        // Insecure HTTP to remote
        let err_http = policy
            .validate_endpoint("http://api.openai.com/v1", true)
            .unwrap_err();
        assert!(matches!(err_http, AiAdapterError::PolicyViolation(_)));
        assert!(err_http.to_string().contains("must use HTTPS"));

        // Missing key
        let err_key = policy
            .validate_endpoint("https://api.openai.com/v1", false)
            .unwrap_err();
        assert!(matches!(err_key, AiAdapterError::AuthenticationRequired(_)));

        // Valid remote
        assert!(policy
            .validate_endpoint("https://api.openai.com/v1", true)
            .is_ok());
    }
}
