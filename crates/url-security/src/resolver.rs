//! # Secure DNS Resolver and Rebinding Protection Engine
//!
//! Enforces zero-trust DNS resolution, all-record SSRF validation, IP pinning,
//! and Time-of-Check to Time-of-Use (TOCTOU) DNS rebinding protection.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use tokio::sync::RwLock;

use crate::dns::{DnsResolver, SystemDnsResolver};
use crate::error::UrlSecurityError;
use crate::ip_filter::{is_blocked_host, is_blocked_ip, parse_alternative_ipv4};
use crate::pinning::PinnedTarget;
use crate::policy::UrlSecurityPolicy;
use crate::validated_url::ValidatedUrl;

/// Default DNS cache time-to-live in seconds (prevents premature re-resolutions during a job).
pub const DEFAULT_DNS_CACHE_TTL_SECS: u64 = 60;

/// Central security coordinator for DNS resolution, SSRF validation, and IP pinning.
#[derive(Debug, Clone)]
pub struct SecureDnsResolver {
    /// Active URL security policy.
    policy: UrlSecurityPolicy,
    /// Underlying asynchronous DNS resolver implementation.
    resolver: Arc<dyn DnsResolver>,
    /// Thread-safe in-memory cache mapping `(host, port)` to validated `PinnedTarget`.
    cache: Arc<RwLock<HashMap<(String, u16), PinnedTarget>>>,
    /// Default TTL in seconds for pinned targets.
    ttl_secs: u64,
}

impl SecureDnsResolver {
    /// Constructs a `SecureDnsResolver` with the provided policy and system DNS resolution.
    #[must_use]
    pub fn new(policy: UrlSecurityPolicy) -> Self {
        Self {
            policy,
            resolver: Arc::new(SystemDnsResolver),
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl_secs: DEFAULT_DNS_CACHE_TTL_SECS,
        }
    }

    /// Constructs a `SecureDnsResolver` with a custom `DnsResolver` implementation (e.g. for testing).
    #[must_use]
    pub fn with_resolver(policy: UrlSecurityPolicy, resolver: Arc<dyn DnsResolver>) -> Self {
        Self {
            policy,
            resolver,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl_secs: DEFAULT_DNS_CACHE_TTL_SECS,
        }
    }

    /// Sets the DNS cache TTL in seconds.
    #[must_use]
    pub fn with_ttl(mut self, ttl_secs: u64) -> Self {
        self.ttl_secs = ttl_secs;
        self
    }

    /// Returns a reference to the active `UrlSecurityPolicy`.
    #[must_use]
    pub fn policy(&self) -> &UrlSecurityPolicy {
        &self.policy
    }

    /// Clears all cached pinned targets.
    pub async fn clear_cache(&self) {
        let mut map = self.cache.write().await;
        map.clear();
    }

    /// Returns the number of currently cached targets.
    pub async fn cached_count(&self) -> usize {
        let map = self.cache.read().await;
        map.len()
    }

    /// Resolves a `ValidatedUrl`, validates all returned addresses against SSRF policies,
    /// and returns an immutable `PinnedTarget`.
    ///
    /// ## SSRF & Rebinding Guarantees:
    /// 1. If cached and unexpired, returns the previously verified `PinnedTarget`, preventing
    ///    subsequent requests from triggering mid-session DNS rebinding.
    /// 2. If the URL host is an IP literal or alternative notation, validates it directly without DNS.
    /// 3. Validates the hostname against the prohibited domain blocklist.
    /// 4. Resolves candidate IP addresses via the underlying `DnsResolver`.
    /// 5. **ALL-RECORD VALIDATION:** Evaluates EVERY resolved `SocketAddr` against `is_blocked_ip`.
    ///    If even ONE address belongs to a prohibited CIDR (loopback, private, cloud metadata),
    ///    the entire resolution is immediately rejected with `UrlSecurityError::BlockedIp`.
    /// 6. Pins to a verified IP address (preferring IPv4 when dual-stack is present).
    pub async fn resolve_and_pin(
        &self,
        url: &ValidatedUrl,
    ) -> Result<PinnedTarget, UrlSecurityError> {
        let host = url.host_str();
        let port = url.port_or_default();
        let cache_key = (host.to_ascii_lowercase(), port);

        // 1. Check cache for valid, unexpired pinned target
        {
            let map = self.cache.read().await;
            if let Some(cached) = map.get(&cache_key) {
                if !cached.is_expired() {
                    let mut target = cached.clone();
                    target.url = url.clone();
                    return Ok(target);
                }
            }
        }

        // 2. Port validation
        if self.policy.is_port_blocked(port) {
            return Err(UrlSecurityError::BlockedPort {
                port,
                reason: "Port is blocked by security policy".to_string(),
            });
        }

        // 3. Host validation
        is_blocked_host(host, &self.policy)?;

        // 4. IP literal bypass check (stripping optional IPv6 zone index)
        let unbracketed = host.trim_start_matches('[').trim_end_matches(']');
        let ip_str = if let Some((base, _)) = unbracketed.split_once('%') {
            base
        } else {
            unbracketed
        };
        if let Ok(ip) = ip_str.parse::<IpAddr>() {
            is_blocked_ip(ip, &self.policy)?;
            let pinned = PinnedTarget::new(url.clone(), ip, vec![ip], self.ttl_secs);
            let mut map = self.cache.write().await;
            map.insert(cache_key, pinned.clone());
            return Ok(pinned);
        }

        // 5. Alternative IP representation check (e.g. 2130706433 -> 127.0.0.1)
        if let Some(v4) = parse_alternative_ipv4(host) {
            let ip = IpAddr::V4(v4);
            is_blocked_ip(ip, &self.policy)?;
            let pinned = PinnedTarget::new(url.clone(), ip, vec![ip], self.ttl_secs);
            let mut map = self.cache.write().await;
            map.insert(cache_key, pinned.clone());
            return Ok(pinned);
        }

        // 6. Asynchronous DNS resolution
        let candidate_addrs = self.resolver.resolve(host, port).await?;
        if candidate_addrs.is_empty() {
            return Err(UrlSecurityError::DnsResolutionFailed {
                host: host.to_string(),
                details: "DNS resolution returned zero records".to_string(),
            });
        }

        // 7. Rigorous All-Record SSRF Evaluation
        let mut safe_ips = Vec::with_capacity(candidate_addrs.len());
        for addr in &candidate_addrs {
            let ip = addr.ip();
            // If any single address is blocked, reject the entire hostname
            is_blocked_ip(ip, &self.policy)?;
            if !safe_ips.contains(&ip) {
                safe_ips.push(ip);
            }
        }

        // 8. Select pinned IP (prefer IPv4 for reliable routing, fallback to IPv6)
        let pinned_ip = safe_ips
            .iter()
            .find(|ip| ip.is_ipv4())
            .copied()
            .unwrap_or(safe_ips[0]);

        let pinned_target = PinnedTarget::new(url.clone(), pinned_ip, safe_ips, self.ttl_secs);

        // 9. Update cache
        {
            let mut map = self.cache.write().await;
            map.insert(cache_key, pinned_target.clone());
        }

        Ok(pinned_target)
    }

    /// Verifies that an established socket connection matches the expected pinned target.
    pub fn verify_connection(
        &self,
        target: &PinnedTarget,
        actual: std::net::SocketAddr,
    ) -> Result<(), UrlSecurityError> {
        target.verify_socket_addr(actual)
    }
}
