//! # URL Normalizer
//!
//! Canonicalizes and normalizes URLs according to zero-trust security policies:
//! - Character validation and length checks
//! - Scheme, host, and port normalization
//! - Path normalization and dot segment resolution
//! - Tracking/analytics parameter removal and deterministic query sorting
//! - Fragment stripping

use url::Url;

use crate::error::UrlSecurityError;
use crate::ip_filter::is_blocked_host;
use crate::policy::UrlSecurityPolicy;

/// Normalizes and validates raw URL strings against security policies.
pub struct UrlNormalizer;

impl UrlNormalizer {
    /// Validates and canonicalizes a raw URL string using the provided policy.
    pub fn normalize(raw: &str, policy: &UrlSecurityPolicy) -> Result<Url, UrlSecurityError> {
        let trimmed = raw.trim();

        // 1. Length validation
        if trimmed.len() > policy.max_url_length {
            return Err(UrlSecurityError::TooLong {
                actual: trimmed.len(),
                maximum: policy.max_url_length,
            });
        }

        // 2. Control character / null byte detection
        for (idx, ch) in trimmed.chars().enumerate() {
            if ch == '\0' {
                return Err(UrlSecurityError::InvalidCharacters {
                    details: format!("Null byte detected at character position {idx}"),
                });
            }
            if ch.is_control() {
                return Err(UrlSecurityError::InvalidCharacters {
                    details: format!(
                        "Prohibited control character U+{:04X} at position {idx}",
                        ch as u32
                    ),
                });
            }
        }

        // 3. WHATWG URL Parsing
        let mut parsed = Url::parse(trimmed).map_err(|e| UrlSecurityError::ParseError {
            details: e.to_string(),
        })?;

        // 4. Scheme validation
        let scheme = parsed.scheme().to_ascii_lowercase();
        if !policy.is_scheme_allowed(&scheme) {
            return Err(UrlSecurityError::ForbiddenScheme {
                scheme,
                reason: format!("Scheme not in permitted list {:?}", policy.allowed_schemes),
            });
        }

        // 5. Userinfo credentials validation
        if policy.disallow_userinfo
            && (!parsed.username().is_empty() || parsed.password().is_some())
        {
            return Err(UrlSecurityError::UserinfoForbidden);
        }

        // 6. Host existence and SSRF validation
        let host_str = parsed.host_str().ok_or(UrlSecurityError::MissingHost)?;
        is_blocked_host(host_str, policy)?;

        // 7. Port validation and default-port stripping
        if let Some(port) = parsed.port() {
            if policy.is_port_blocked(port) {
                return Err(UrlSecurityError::BlockedPort {
                    port,
                    reason: "Port is blocked by security policy".to_string(),
                });
            }

            // Strip default ports (80 for http, 443 for https)
            if (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
                let _ = parsed.set_port(None);
            }
        }

        // 8. Path normalization: collapse multiple consecutive slashes
        let current_path = parsed.path();
        let normalized_path = normalize_path_slashes(current_path);
        parsed.set_path(&normalized_path);

        // 9. Query normalization: strip tracking params & sort remaining pairs deterministically
        if parsed.query().is_some() {
            let mut pairs: Vec<(String, String)> = parsed
                .query_pairs()
                .filter(|(k, _)| !policy.is_tracking_param(k))
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect();

            if pairs.is_empty() {
                parsed.set_query(None);
            } else {
                pairs.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
                let mut serializer = url::form_urlencoded::Serializer::new(String::new());
                for (k, v) in pairs {
                    serializer.append_pair(&k, &v);
                }
                parsed.set_query(Some(&serializer.finish()));
            }
        }

        // 10. Fragment stripping
        if policy.strip_fragments {
            parsed.set_fragment(None);
        }

        Ok(parsed)
    }
}

/// Collapses duplicate consecutive `/` slashes in URL paths while preserving leading `/`.
fn normalize_path_slashes(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }

    let mut out = String::with_capacity(path.len());
    let mut prev_slash = false;

    for c in path.chars() {
        if c == '/' {
            if !prev_slash {
                out.push(c);
                prev_slash = true;
            }
        } else {
            out.push(c);
            prev_slash = false;
        }
    }

    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}
