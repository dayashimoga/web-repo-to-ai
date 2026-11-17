//! OpenTelemetry and W3C TraceContext propagation.
//!
//! Implements the W3C Trace Context recommendation (`traceparent` header)
//! enabling distributed tracing correlation across HTTP requests and asynchronous workers.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use web_repo_to_ai_core::{CoreError, Result};

/// W3C Trace Context representation (`traceparent` header).
///
/// Format: `version(2 hex)-trace_id(32 hex)-parent_id(16 hex)-trace_flags(2 hex)`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceContext {
    /// 128-bit trace identifier represented as 32 lowercase hexadecimal characters.
    pub trace_id: String,
    /// 64-bit span identifier represented as 16 lowercase hexadecimal characters.
    pub span_id: String,
    /// Trace flags bitmap (0x01 indicates sampled).
    pub trace_flags: u8,
}

impl TraceContext {
    /// Creates a new root trace context with randomly generated trace and span identifiers.
    #[must_use]
    pub fn new_root() -> Self {
        let trace_bytes = generate_random_bytes::<16>();
        let span_bytes = generate_random_bytes::<8>();
        Self {
            trace_id: hex::encode(trace_bytes),
            span_id: hex::encode(span_bytes),
            trace_flags: 0x01, // Sampled by default
        }
    }

    /// Derives a child span context within the same distributed trace.
    #[must_use]
    pub fn new_child(&self) -> Self {
        let span_bytes = generate_random_bytes::<8>();
        Self {
            trace_id: self.trace_id.clone(),
            span_id: hex::encode(span_bytes),
            trace_flags: self.trace_flags,
        }
    }

    /// Formats this context into a standard W3C `traceparent` header value.
    #[must_use]
    pub fn to_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Parses a W3C `traceparent` header value according to the W3C recommendation.
    ///
    /// # Errors
    /// Returns [`CoreError::Validation`] if the header is malformed, uses an unsupported version,
    /// or contains all-zero identifiers.
    pub fn from_traceparent(header: &str) -> Result<Self> {
        let trimmed = header.trim();
        let parts: Vec<&str> = trimmed.split('-').collect();

        if parts.len() != 4 {
            return Err(CoreError::Validation {
                field: "traceparent".to_string(),
                reason: format!("Expected 4 dash-separated parts, found {}", parts.len()),
            });
        }

        let version = parts[0];
        let trace_id = parts[1];
        let span_id = parts[2];
        let flags_str = parts[3];

        if version != "00" {
            return Err(CoreError::Validation {
                field: "traceparent.version".to_string(),
                reason: format!(
                    "Unsupported traceparent version '{}': only '00' supported",
                    version
                ),
            });
        }

        if trace_id.len() != 32 || !trace_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "traceparent.trace_id".to_string(),
                reason: "trace_id must be exactly 32 hex characters".to_string(),
            });
        }

        if trace_id.chars().all(|c| c == '0') {
            return Err(CoreError::Validation {
                field: "traceparent.trace_id".to_string(),
                reason: "trace_id cannot be all zeros".to_string(),
            });
        }

        if span_id.len() != 16 || !span_id.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "traceparent.span_id".to_string(),
                reason: "span_id must be exactly 16 hex characters".to_string(),
            });
        }

        if span_id.chars().all(|c| c == '0') {
            return Err(CoreError::Validation {
                field: "traceparent.span_id".to_string(),
                reason: "span_id cannot be all zeros".to_string(),
            });
        }

        if flags_str.len() != 2 || !flags_str.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(CoreError::Validation {
                field: "traceparent.flags".to_string(),
                reason: "trace_flags must be exactly 2 hex characters".to_string(),
            });
        }

        let trace_flags =
            u8::from_str_radix(flags_str, 16).map_err(|err| CoreError::Validation {
                field: "traceparent.flags".to_string(),
                reason: format!("Failed to parse hex flags: {}", err),
            })?;

        Ok(Self {
            trace_id: trace_id.to_ascii_lowercase(),
            span_id: span_id.to_ascii_lowercase(),
            trace_flags,
        })
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new_root()
    }
}

impl fmt::Display for TraceContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_traceparent())
    }
}

impl FromStr for TraceContext {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_traceparent(s)
    }
}

fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    let uuid = uuid::Uuid::new_v4();
    let uuid_bytes = uuid.as_bytes();
    for (i, byte) in bytes.iter_mut().enumerate() {
        *byte = uuid_bytes[i % uuid_bytes.len()];
    }
    bytes
}

mod hex {
    pub fn encode<T: AsRef<[u8]>>(data: T) -> String {
        let bytes = data.as_ref();
        let mut s = String::with_capacity(bytes.len() * 2);
        for &b in bytes {
            use std::fmt::Write;
            let _ = write!(s, "{:02x}", b);
        }
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traceparent_roundtrip() {
        let ctx = TraceContext::new_root();
        let header = ctx.to_traceparent();
        let parsed = TraceContext::from_traceparent(&header).expect("Must parse valid header");
        assert_eq!(ctx, parsed);
    }

    #[test]
    fn test_child_span_derivation() {
        let root = TraceContext::new_root();
        let child = root.new_child();
        assert_eq!(root.trace_id, child.trace_id);
        assert_ne!(root.span_id, child.span_id);
        assert_eq!(root.trace_flags, child.trace_flags);
    }

    #[test]
    fn test_parse_standard_header() {
        let raw = "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";
        let ctx = TraceContext::from_traceparent(raw).unwrap();
        assert_eq!(ctx.trace_id, "4bf92f3577b34da6a3ce929d0e0e4736");
        assert_eq!(ctx.span_id, "00f067aa0ba902b7");
        assert_eq!(ctx.trace_flags, 0x01);
    }

    #[test]
    fn test_all_zeros_rejection() {
        let all_zeros_trace = "00-00000000000000000000000000000000-00f067aa0ba902b7-01";
        assert!(TraceContext::from_traceparent(all_zeros_trace).is_err());

        let all_zeros_span = "00-4bf92f3577b34da6a3ce929d0e0e4736-0000000000000000-01";
        assert!(TraceContext::from_traceparent(all_zeros_span).is_err());
    }
}
