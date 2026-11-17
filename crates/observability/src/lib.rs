//! # Observability & Telemetry (`web-repo-to-ai-observability`)
//!
//! Provides telemetry, metrics, and structured logging primitives:
//! - **Structured Logging:** [`init_tracing`], [`LoggingConfig`], and [`LogFormat`] (JSON, pretty, compact).
//! - **Secret Redaction:** [`redact_sensitive_text`], [`is_sensitive_key`], and [`redact_json_value`].
//! - **Prometheus Metrics:** [`AppMetrics`], [`Counter`], [`Gauge`], and [`Histogram`].
//! - **Distributed Tracing:** [`TraceContext`] implementing W3C TraceContext `traceparent` propagation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod logging;
pub mod metrics;
pub mod redaction;
pub mod trace;

// Re-exports for clean ergonomic use
pub use logging::{init_tracing, LogFormat, LoggingConfig};
pub use metrics::{AppMetrics, Counter, CounterVec, Gauge, Histogram, LATENCY_BUCKETS_MS};
pub use redaction::{
    is_sensitive_key, redact_json_value, redact_sensitive_text, REDACTED_API_KEY,
    REDACTED_GITHUB_TOKEN, REDACTED_SECRET, REDACTED_TOKEN,
};
pub use trace::TraceContext;

/// Returns the semantic crate version.
#[must_use]
pub fn crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-observability");
        assert!(!crate_version().is_empty());
    }

    #[test]
    fn test_observability_reexports() {
        let trace = TraceContext::new_root();
        assert_eq!(trace.to_traceparent().len(), 55);

        let metrics = AppMetrics::new();
        metrics.jobs_total.inc("submitted");
        assert!(metrics
            .render_prometheus_text()
            .contains("web_repo_to_ai_jobs_total"));

        let redacted = redact_sensitive_text("ghp_123456789012345678901234567890123456");
        assert_eq!(redacted, REDACTED_GITHUB_TOKEN);
    }
}
