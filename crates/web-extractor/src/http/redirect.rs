//! # Redirect Tracking and Loop Mitigation
//!
//! Enforces hop limits, cyclic redirect loop detection, and hop-by-hop provenance capture.

use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use crate::error::WebExtractorError;

/// Recorded provenance data for a single HTTP redirect transition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedirectHop {
    /// Originating URL that issued the redirect response.
    pub from_url: ValidatedUrl,
    /// Destination URL extracted from the `Location` header and revalidated.
    pub to_url: ValidatedUrl,
    /// HTTP status code of the redirect (e.g. 301, 302, 303, 307, 308).
    pub status_code: u16,
    /// Resolved IP address from which the redirect response was received.
    pub pinned_ip: IpAddr,
    /// Timestamp when this redirect hop was executed and validated.
    pub timestamp: DateTime<Utc>,
}

/// Tracks visited URLs during an acquisition session to enforce hop limits and detect cyclic redirect loops.
#[derive(Debug, Clone)]
pub struct RedirectTracker {
    /// Maximum allowed redirect hops before aborting.
    max_redirects: usize,
    /// Set of canonical URLs visited during this acquisition chain.
    visited_urls: Vec<ValidatedUrl>,
    /// Chronological list of recorded redirect transitions.
    hops: Vec<RedirectHop>,
}

impl RedirectTracker {
    /// Constructs a new `RedirectTracker` with the given hop limit.
    #[must_use]
    pub fn new(max_redirects: usize) -> Self {
        Self {
            max_redirects,
            visited_urls: Vec::new(),
            hops: Vec::new(),
        }
    }

    /// Records the initial target URL before any redirect transitions occur.
    pub fn record_initial(&mut self, url: &ValidatedUrl) {
        if self.visited_urls.is_empty() {
            self.visited_urls.push(url.clone());
        }
    }

    /// Validates a redirect transition, checks for loops, and appends a `RedirectHop`.
    ///
    /// ## Security Guarantees:
    /// - Revalidates `location` string against `policy` using `from.join(location, policy)`.
    /// - Detects cycles: if `to_url` was previously visited in this chain, returns `RedirectLoop`.
    /// - Enforces maximum hop count: returns `TooManyRedirects` if `hops.len() >= max_redirects`.
    pub fn check_and_record(
        &mut self,
        from: &ValidatedUrl,
        location: &str,
        status: u16,
        pinned_ip: IpAddr,
        policy: &UrlSecurityPolicy,
    ) -> Result<ValidatedUrl, WebExtractorError> {
        if self.hops.len() >= self.max_redirects {
            return Err(WebExtractorError::TooManyRedirects {
                limit: self.max_redirects,
            });
        }

        let to_url = from
            .join(location, policy)
            .map_err(WebExtractorError::from)?;

        // Cycle / redirect loop detection
        if self.visited_urls.iter().any(|visited| visited == &to_url) {
            return Err(WebExtractorError::RedirectLoop {
                url: to_url.to_string(),
                hops: self.hops.len() + 1,
            });
        }

        self.visited_urls.push(to_url.clone());
        self.hops.push(RedirectHop {
            from_url: from.clone(),
            to_url: to_url.clone(),
            status_code: status,
            pinned_ip,
            timestamp: Utc::now(),
        });

        Ok(to_url)
    }

    /// Consumes the tracker and returns the chronological list of recorded hops.
    #[must_use]
    pub fn into_hops(self) -> Vec<RedirectHop> {
        self.hops
    }

    /// Returns the number of redirect hops recorded so far.
    #[must_use]
    pub fn hop_count(&self) -> usize {
        self.hops.len()
    }
}
