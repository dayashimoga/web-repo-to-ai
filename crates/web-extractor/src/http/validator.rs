//! # HTTP Caching Validators and Conditional Request Engine
//!
//! Provides RFC 7231 / RFC 7232 compliant HTTP caching validators (`ETag`, `Last-Modified`,
//! `Cache-Control`, and `Expires`), conditional request header generation (`If-None-Match`,
//! `If-Modified-Since`), freshness calculation, and 304 Not Modified evaluation.

use std::collections::HashMap;
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::ValidatedUrl;

use super::response::FetchedResponse;

/// Standard IMF-fixdate format per RFC 7231 Section 7.1.1.1.
pub const HTTP_DATE_FMT: &str = "%a, %d %b %Y %H:%M:%S GMT";

/// Parses an HTTP date string according to RFC 7231 formats:
/// 1. IMF-fixdate (preferred): `Sun, 06 Nov 1994 08:49:37 GMT`
/// 2. RFC 850 format: `Sunday, 06-Nov-94 08:49:37 GMT`
/// 3. ANSI C asctime format: `Sun Nov  6 08:49:37 1994`
#[must_use]
pub fn parse_http_date(s: &str) -> Option<DateTime<Utc>> {
    let trimmed = s.trim();

    // 1. Try standard IMF-fixdate
    if let Ok(dt) = DateTime::parse_from_str(trimmed, "%a, %d %b %Y %H:%M:%S GMT") {
        return Some(dt.with_timezone(&Utc));
    }

    // Also try with RFC 2822 parser (handles various timezone formats)
    if let Ok(dt) = DateTime::parse_from_rfc2822(trimmed) {
        return Some(dt.with_timezone(&Utc));
    }

    // 2. Try RFC 850 format: Sunday, 06-Nov-94 08:49:37 GMT
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%A, %d-%b-%y %H:%M:%S GMT") {
        return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
    }

    // 3. Try asctime format: Sun Nov  6 08:49:37 1994
    if let Ok(naive) = NaiveDateTime::parse_from_str(trimmed, "%a %b %e %H:%M:%S %Y") {
        return Some(DateTime::from_naive_utc_and_offset(naive, Utc));
    }

    None
}

/// Formats a UTC timestamp into standard RFC 7231 IMF-fixdate format.
#[must_use]
pub fn format_http_date(dt: &DateTime<Utc>) -> String {
    dt.format(HTTP_DATE_FMT).to_string()
}

/// HTTP Entity Tag (`ETag`) validator per RFC 7232 Section 2.3.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Etag {
    /// Normalized opaque entity tag value without enclosing quotes or weak indicator.
    pub value: String,
    /// Indicates whether the entity tag is a weak validator (starts with `W/`).
    pub is_weak: bool,
    /// Exact raw wire header value as received from the server.
    pub raw: String,
}

impl Etag {
    /// Parses an ETag header value string into a structured `Etag`.
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }

        let (is_weak, value_slice) = if let Some(rest) = trimmed.strip_prefix("W/") {
            (true, rest.trim())
        } else if let Some(rest) = trimmed.strip_prefix("w/") {
            (true, rest.trim())
        } else {
            (false, trimmed)
        };

        let unquoted =
            if value_slice.starts_with('"') && value_slice.ends_with('"') && value_slice.len() >= 2
            {
                &value_slice[1..value_slice.len() - 1]
            } else {
                value_slice
            };

        if unquoted.is_empty() {
            return None;
        }

        Some(Self {
            value: unquoted.to_string(),
            is_weak,
            raw: trimmed.to_string(),
        })
    }

    /// Compares two entity tags according to RFC 7232 Section 2.3.2.
    ///
    /// - **Strong comparison**: Both validators must NOT be weak, and the opaque tags must match character-by-character.
    /// - **Weak comparison**: The opaque tags match character-by-character, regardless of weak indicators.
    #[must_use]
    pub fn matches(&self, other: &Self, strong_comparison: bool) -> bool {
        if strong_comparison {
            !self.is_weak && !other.is_weak && self.value == other.value
        } else {
            self.value == other.value
        }
    }

    /// Returns the exact wire format suitable for HTTP request headers (`If-None-Match`).
    #[must_use]
    pub fn wire_value(&self) -> &str {
        &self.raw
    }
}

/// Parsed HTTP `Cache-Control` header directives per RFC 7234 Section 5.2.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpCacheControl {
    /// The `max-age` directive specifies the maximum amount of time in seconds that a fetched
    /// response is considered fresh.
    pub max_age: Option<Duration>,
    /// The `s-maxage` directive specifies the maximum lifetime for shared caches.
    pub s_maxage: Option<Duration>,
    /// The `no-cache` directive indicates that the response MUST NOT be used to satisfy subsequent
    /// requests without successful validation on the origin server.
    pub no_cache: bool,
    /// The `no-store` directive indicates that a cache MUST NOT store any part of either this response or request.
    pub no_store: bool,
    /// The `must-revalidate` directive indicates that once a response becomes stale, a cache MUST NOT
    /// use the response to satisfy subsequent requests without successful validation on the origin server.
    pub must_revalidate: bool,
    /// The `proxy-revalidate` directive has the same meaning as `must-revalidate`, but only for shared caches.
    pub proxy_revalidate: bool,
    /// The `public` directive indicates that any cache MAY store the response.
    pub is_public: bool,
    /// The `private` directive indicates that the response message is intended for a single user.
    pub is_private: bool,
    /// The `immutable` directive indicates that the response body will not change over time.
    pub immutable: bool,
}

impl HttpCacheControl {
    /// Parses a `Cache-Control` header string containing comma-separated directives.
    #[must_use]
    pub fn parse(header: &str) -> Self {
        let mut cc = Self::default();

        for directive in header.split(',') {
            let part = directive.trim();
            if part.is_empty() {
                continue;
            }

            let mut kv = part.splitn(2, '=');
            let key = kv.next().unwrap_or("").trim().to_ascii_lowercase();
            let val = kv.next().map(|v| v.trim().trim_matches('"'));

            match key.as_str() {
                "max-age" => {
                    if let Some(secs) = val.and_then(|v| v.parse::<u64>().ok()) {
                        cc.max_age = Some(Duration::from_secs(secs));
                    }
                }
                "s-maxage" => {
                    if let Some(secs) = val.and_then(|v| v.parse::<u64>().ok()) {
                        cc.s_maxage = Some(Duration::from_secs(secs));
                    }
                }
                "no-cache" => cc.no_cache = true,
                "no-store" => cc.no_store = true,
                "must-revalidate" => cc.must_revalidate = true,
                "proxy-revalidate" => cc.proxy_revalidate = true,
                "public" => cc.is_public = true,
                "private" => cc.is_private = true,
                "immutable" => cc.immutable = true,
                _ => {}
            }
        }

        cc
    }

    /// Determines whether the response is considered fresh at the given `now` timestamp based on `retrieved_at`.
    #[must_use]
    pub fn is_fresh(&self, retrieved_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        if self.no_cache || self.no_store {
            return false;
        }

        if let Some(max_age) = self.max_age {
            let elapsed = (now - retrieved_at).num_seconds();
            if elapsed < 0 {
                return true;
            }
            (elapsed as u64) < max_age.as_secs()
        } else {
            false
        }
    }

    /// Returns true if the cache policy explicitly requires revalidation before serving.
    #[must_use]
    pub fn requires_revalidation(&self) -> bool {
        self.no_cache || self.must_revalidate
    }
}

/// HTTP validator metadata extracted from response headers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpValidator {
    /// Parsed entity tag (`ETag`) if provided.
    pub etag: Option<Etag>,
    /// Parsed timestamp from `Last-Modified` header if provided.
    pub last_modified: Option<DateTime<Utc>>,
    /// Preserved raw `Last-Modified` header string.
    pub raw_last_modified: Option<String>,
    /// Parsed `Cache-Control` directives if provided.
    pub cache_control: Option<HttpCacheControl>,
    /// Explicit expiration timestamp from `Expires` header or calculated from `max-age`.
    pub expires_at: Option<DateTime<Utc>>,
}

impl HttpValidator {
    /// Constructs an `HttpValidator` by inspecting preserved HTTP response headers.
    #[must_use]
    pub fn from_headers(headers: &HashMap<String, String>, fetched_at: DateTime<Utc>) -> Self {
        let etag = headers.get("etag").and_then(|val| Etag::parse(val));

        let (last_modified, raw_last_modified) = if let Some(raw_lm) = headers.get("last-modified")
        {
            (parse_http_date(raw_lm), Some(raw_lm.clone()))
        } else {
            (None, None)
        };

        let cache_control = headers
            .get("cache-control")
            .map(|cc| HttpCacheControl::parse(cc));

        // Calculate expires_at: check max-age first, then Expires header
        let expires_at = if let Some(ref cc) = cache_control {
            if let Some(max_age) = cc.max_age {
                Some(
                    fetched_at
                        + chrono::Duration::from_std(max_age)
                            .unwrap_or_else(|_| chrono::Duration::zero()),
                )
            } else {
                headers.get("expires").and_then(|exp| parse_http_date(exp))
            }
        } else {
            headers.get("expires").and_then(|exp| parse_http_date(exp))
        };

        Self {
            etag,
            last_modified,
            raw_last_modified,
            cache_control,
            expires_at,
        }
    }

    /// Returns conditional HTTP request headers (`If-None-Match`, `If-Modified-Since`)
    /// suitable for transmitting in a conditional revalidation request.
    #[must_use]
    pub fn conditional_headers(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        if let Some(ref etag) = self.etag {
            map.insert("if-none-match".to_string(), etag.wire_value().to_string());
        }

        if let Some(ref raw_lm) = self.raw_last_modified {
            map.insert("if-modified-since".to_string(), raw_lm.clone());
        } else if let Some(dt) = self.last_modified {
            map.insert("if-modified-since".to_string(), format_http_date(&dt));
        }

        map
    }

    /// Determines whether the cached content is still fresh without making a network request.
    #[must_use]
    pub fn is_fresh(&self, fetched_at: DateTime<Utc>, now: DateTime<Utc>) -> bool {
        if let Some(ref cc) = self.cache_control {
            if cc.requires_revalidation() {
                return false;
            }
            if cc.is_fresh(fetched_at, now) {
                return true;
            }
        }

        if let Some(expires) = self.expires_at {
            return now < expires;
        }

        false
    }
}

/// The result of an HTTP acquisition that may have evaluated conditional validation headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchOutcome {
    /// Upstream content was modified (HTTP 200..=299) and a full response payload was returned.
    Modified(FetchedResponse),
    /// Upstream content was not modified (HTTP 304 Not Modified).
    ///
    /// The cached content remains authoritative; metadata and headers are updated.
    NotModified {
        /// The terminal validated URL that returned 304.
        url: ValidatedUrl,
        /// HTTP status code (304).
        status: u16,
        /// Preserved headers from the 304 response.
        headers: HashMap<String, String>,
        /// Refreshed HTTP validator metadata from the 304 response.
        validator: Option<HttpValidator>,
        /// Timestamp when the 304 response was received.
        fetched_at: DateTime<Utc>,
    },
}

impl FetchOutcome {
    /// Returns `true` if the outcome is `NotModified` (HTTP 304).
    #[must_use]
    pub fn is_not_modified(&self) -> bool {
        matches!(self, Self::NotModified { .. })
    }

    /// Returns `true` if the outcome is `Modified` (new payload returned).
    #[must_use]
    pub fn is_modified(&self) -> bool {
        matches!(self, Self::Modified(_))
    }

    /// Returns the HTTP status code of the response.
    #[must_use]
    pub fn status(&self) -> u16 {
        match self {
            Self::Modified(resp) => resp.status,
            Self::NotModified { status, .. } => *status,
        }
    }

    /// Returns a reference to the terminal validated URL.
    #[must_use]
    pub fn url(&self) -> &ValidatedUrl {
        match self {
            Self::Modified(resp) => &resp.url,
            Self::NotModified { url, .. } => url,
        }
    }

    /// Returns a reference to the preserved response headers.
    #[must_use]
    pub fn headers(&self) -> &HashMap<String, String> {
        match self {
            Self::Modified(resp) => &resp.headers,
            Self::NotModified { headers, .. } => headers,
        }
    }

    /// Returns the refreshed or extracted `HttpValidator` if present.
    #[must_use]
    pub fn validator(&self) -> Option<HttpValidator> {
        match self {
            Self::Modified(resp) => {
                Some(HttpValidator::from_headers(&resp.headers, resp.fetched_at))
            }
            Self::NotModified { validator, .. } => validator.clone(),
        }
    }

    /// Consumes the outcome and returns the inner `FetchedResponse` if modified.
    #[must_use]
    pub fn into_response(self) -> Option<FetchedResponse> {
        match self {
            Self::Modified(resp) => Some(resp),
            Self::NotModified { .. } => None,
        }
    }
}
