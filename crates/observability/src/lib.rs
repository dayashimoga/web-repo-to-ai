//! # Observability & Telemetry (`web-repo-to-ai-observability`)
//!
//! Provides telemetry, metrics, and logging primitives:
//! - Structured JSON tracing subscriber with sensitive credential redaction
//! - Real-time SSE telemetry log formatting
//! - Pipeline stage duration and memory metrics

#![warn(missing_docs)]

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
    }
}
