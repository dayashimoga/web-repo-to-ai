//! # IP Pinning and Verified Connection Targets
//!
//! Encapsulates verified network target endpoints bound to a specific, SSRF-validated IP address,
//! preventing DNS rebinding during connection establishment.

use std::net::{IpAddr, SocketAddr};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::error::UrlSecurityError;
use crate::validated_url::ValidatedUrl;

/// A verified network target bound to a specific pinned IP address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedTarget {
    /// The originating validated URL.
    pub url: ValidatedUrl,
    /// Destination hostname.
    pub host: String,
    /// Network destination port.
    pub port: u16,
    /// The specific IP address to which connection establishment is pinned.
    pub pinned_ip: IpAddr,
    /// All IP addresses resolved for this hostname that passed SSRF validation.
    pub all_resolved_ips: Vec<IpAddr>,
    /// Timestamp when DNS resolution and validation completed.
    pub resolved_at: DateTime<Utc>,
    /// Time-to-live in seconds during which this pinned target is considered valid.
    pub ttl_secs: u64,
}

impl PinnedTarget {
    /// Constructs a new `PinnedTarget`.
    #[must_use]
    pub fn new(
        url: ValidatedUrl,
        pinned_ip: IpAddr,
        all_resolved_ips: Vec<IpAddr>,
        ttl_secs: u64,
    ) -> Self {
        let host = url.host_str().to_string();
        let port = url.port_or_default();
        Self {
            url,
            host,
            port,
            pinned_ip,
            all_resolved_ips,
            resolved_at: Utc::now(),
            ttl_secs,
        }
    }

    /// Returns the exact `SocketAddr` to which the client socket MUST connect.
    #[must_use]
    pub fn socket_addr(&self) -> SocketAddr {
        SocketAddr::new(self.pinned_ip, self.port)
    }

    /// Returns the pinned IP address.
    #[must_use]
    pub fn pinned_ip(&self) -> IpAddr {
        self.pinned_ip
    }

    /// Returns the destination hostname.
    #[must_use]
    pub fn host(&self) -> &str {
        &self.host
    }

    /// Returns the destination network port.
    #[must_use]
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Returns reference to the originating `ValidatedUrl`.
    #[must_use]
    pub fn url(&self) -> &ValidatedUrl {
        &self.url
    }

    /// Returns all safe resolved IP addresses.
    #[must_use]
    pub fn all_resolved_ips(&self) -> &[IpAddr] {
        &self.all_resolved_ips
    }

    /// Checks whether this pinned resolution has exceeded its configured TTL.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        let max_age = Duration::seconds(self.ttl_secs as i64);
        Utc::now() > self.resolved_at + max_age
    }

    /// Verifies that a connected socket address matches this target's pinned IP and port.
    pub fn verify_socket_addr(&self, actual: SocketAddr) -> Result<(), UrlSecurityError> {
        if actual.ip() != self.pinned_ip || actual.port() != self.port {
            return Err(UrlSecurityError::IpPinningMismatch {
                expected: format!("{}:{}", self.pinned_ip, self.port),
                actual: actual.to_string(),
            });
        }
        Ok(())
    }
}
