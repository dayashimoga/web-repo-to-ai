//! # Validated URL
//!
//! A strongly-typed, immutable domain type encapsulating a normalized, SSRF-validated URL.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::error::UrlSecurityError;
use crate::normalizer::UrlNormalizer;
use crate::policy::UrlSecurityPolicy;

/// A canonicalized, security-evaluated URL safe for HTTP acquisition and link traversal.
#[derive(Clone, Eq)]
pub struct ValidatedUrl {
    /// Normalized WHATWG URL instance.
    url: Url,
    /// Preserved original input string.
    raw_input: String,
}

impl ValidatedUrl {
    /// Parses, validates, and normalizes a raw URL string against the specified security policy.
    pub fn parse(raw: &str, policy: &UrlSecurityPolicy) -> Result<Self, UrlSecurityError> {
        let normalized = UrlNormalizer::normalize(raw, policy)?;
        Ok(Self {
            url: normalized,
            raw_input: raw.to_string(),
        })
    }

    /// Parses, validates, and normalizes a raw URL string using the default zero-trust policy.
    pub fn parse_default(raw: &str) -> Result<Self, UrlSecurityError> {
        Self::parse(raw, &UrlSecurityPolicy::default())
    }

    /// Creates a `ValidatedUrl` from an existing `Url`, re-validating against policy.
    pub fn from_url(url: Url, policy: &UrlSecurityPolicy) -> Result<Self, UrlSecurityError> {
        Self::parse(url.as_str(), policy)
    }

    /// Resolves and validates a relative or absolute link target against this base URL.
    pub fn join(&self, link: &str, policy: &UrlSecurityPolicy) -> Result<Self, UrlSecurityError> {
        let trimmed = link.trim();
        // Check for null bytes or forbidden control characters in link
        for (idx, ch) in trimmed.chars().enumerate() {
            if ch == '\0' || (ch.is_control() && ch != '\t' && ch != '\r' && ch != '\n') {
                return Err(UrlSecurityError::InvalidCharacters {
                    details: format!("Prohibited character at position {idx} in relative link"),
                });
            }
        }

        let joined = self
            .url
            .join(trimmed)
            .map_err(|e| UrlSecurityError::ParseError {
                details: format!("Failed to join '{trimmed}' with base '{}': {e}", self.url),
            })?;

        Self::parse(joined.as_str(), policy)
    }

    /// Returns a string slice reference of the canonical URL string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }

    /// Returns an immutable reference to the inner `Url`.
    #[must_use]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the raw input string provided when this URL was parsed.
    #[must_use]
    pub fn raw_input(&self) -> &str {
        &self.raw_input
    }

    /// Returns the scheme of the URL (e.g., `http` or `https`).
    #[must_use]
    pub fn scheme(&self) -> &str {
        self.url.scheme()
    }

    /// Returns the normalized host string.
    #[must_use]
    pub fn host_str(&self) -> &str {
        // Unwrapping is safe because validation guarantees host existence
        self.url.host_str().unwrap_or_default()
    }

    /// Returns the explicit port if configured, or default port (80 for http, 443 for https).
    #[must_use]
    pub fn port_or_default(&self) -> u16 {
        self.url.port_or_known_default().unwrap_or_else(|| {
            if self.scheme() == "https" {
                443
            } else {
                80
            }
        })
    }

    /// Returns the optional explicit non-default port.
    #[must_use]
    pub fn port(&self) -> Option<u16> {
        self.url.port()
    }

    /// Returns the normalized path component of the URL.
    #[must_use]
    pub fn path(&self) -> &str {
        self.url.path()
    }

    /// Returns the optional query string (without leading `?`).
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.url.query()
    }

    /// Returns the optional fragment identifier (without leading `#`).
    #[must_use]
    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment()
    }

    /// Computes the origin string (`scheme://host[:port]`).
    #[must_use]
    pub fn origin(&self) -> String {
        let scheme = self.scheme();
        let host = self.host_str();
        if let Some(port) = self.port() {
            format!("{scheme}://{host}:{port}")
        } else {
            format!("{scheme}://{host}")
        }
    }

    /// Checks whether two URLs share the exact same origin (scheme, host, and port).
    #[must_use]
    pub fn is_same_origin(&self, other: &ValidatedUrl) -> bool {
        self.scheme() == other.scheme()
            && self.host_str().eq_ignore_ascii_case(other.host_str())
            && self.port_or_default() == other.port_or_default()
    }

    /// Checks whether two URLs share the same registered root domain.
    ///
    /// For example, `docs.example.com` and `api.example.com` share domain `example.com`.
    #[must_use]
    pub fn is_same_domain(&self, other: &ValidatedUrl) -> bool {
        let h1 = self.host_str();
        let h2 = other.host_str();

        if h1.eq_ignore_ascii_case(h2) {
            return true;
        }

        // Compare effective 2-part domain suffix (e.g. example.com)
        fn get_root_domain(host: &str) -> &str {
            let parts: Vec<&str> = host.split('.').collect();
            if parts.len() >= 2 {
                let start_idx =
                    host.len() - parts[parts.len() - 2].len() - parts[parts.len() - 1].len() - 1;
                &host[start_idx..]
            } else {
                host
            }
        }

        get_root_domain(h1).eq_ignore_ascii_case(get_root_domain(h2))
    }

    /// Checks whether this URL's path is a subpath of the provided base URL.
    #[must_use]
    pub fn is_subpath_of(&self, base: &ValidatedUrl) -> bool {
        if !self.is_same_origin(base) {
            return false;
        }
        let base_path = base.path().trim_end_matches('/');
        if base_path.is_empty() || base_path == "/" {
            return true;
        }
        let current_path = self.path();
        current_path == base_path || current_path.starts_with(&format!("{base_path}/"))
    }
}

impl Deref for ValidatedUrl {
    type Target = Url;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

impl fmt::Debug for ValidatedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedUrl")
            .field("url", &self.url.as_str())
            .field("raw_input", &self.raw_input)
            .finish()
    }
}

impl fmt::Display for ValidatedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url.as_str())
    }
}

impl PartialEq for ValidatedUrl {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl Hash for ValidatedUrl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
    }
}

impl Serialize for ValidatedUrl {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ValidatedUrl {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse_default(&s).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<&str> for ValidatedUrl {
    type Error = UrlSecurityError;

    fn try_from(raw: &str) -> Result<Self, Self::Error> {
        Self::parse_default(raw)
    }
}
