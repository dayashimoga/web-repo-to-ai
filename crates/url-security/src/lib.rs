//! # URL Security Engine (`web-repo-to-ai-url-security`)
//!
//! Provides zero-trust security controls for URL ingestion:
//! - SSRF defenses and private IPv4/IPv6 CIDR blocking
//! - Cloud metadata service isolation (`169.254.169.254`)
//! - Alternative IP address encoding detection (hex, octal, decimal integers)
//! - Tracking/analytics query parameter stripping and canonicalization
//! - Network port restrictions and protocol confusion defenses
//! - Asynchronous DNS resolution, all-record SSRF validation, and IP pinning
//! - DNS-rebinding (TOCTOU) mitigation via pinned socket targets and cache pinning

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod dns;
pub mod error;
pub mod ip_filter;
pub mod normalizer;
pub mod pinning;
pub mod policy;
pub mod resolver;
pub mod validated_url;

pub use dns::{DnsResolver, MockDnsResolver, SystemDnsResolver};
pub use error::UrlSecurityError;
pub use ip_filter::{is_blocked_host, is_blocked_ip, is_cloud_metadata_ip, parse_alternative_ipv4};
pub use normalizer::UrlNormalizer;
pub use pinning::PinnedTarget;
pub use policy::{
    UrlSecurityPolicy, UrlSecurityPolicyBuilder, DEFAULT_BLOCKED_PORTS, DEFAULT_MAX_URL_LENGTH,
    DEFAULT_TRACKING_PARAMS,
};
pub use resolver::{SecureDnsResolver, DEFAULT_DNS_CACHE_TTL_SECS};
pub use validated_url::ValidatedUrl;

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;

    #[test]
    fn test_url_security_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-url-security");
    }

    #[test]
    fn test_valid_urls_pass_validation() {
        let policy = UrlSecurityPolicy::default();
        let valid_cases = [
            "https://example.com/docs",
            "http://example.org:8080/path/to/resource?id=123",
            "https://developer.mozilla.org/en-US/docs/Web",
            "https://sub.domain.co.uk/page?search=rust&lang=en",
        ];

        for raw in valid_cases {
            let res = ValidatedUrl::parse(raw, &policy);
            assert!(res.is_ok(), "Expected '{raw}' to be valid, got {res:?}");
        }
    }

    #[test]
    fn test_scheme_rejection() {
        let policy = UrlSecurityPolicy::default();
        let invalid_schemes = [
            "file:///etc/passwd",
            "ftp://example.com/file.txt",
            "gopher://example.com/",
            "data:text/html,<html>Hello</html>",
            "javascript:alert(1)",
            "ws://example.com/socket",
            "wss://example.com/socket",
        ];

        for raw in invalid_schemes {
            let res = ValidatedUrl::parse(raw, &policy);
            assert!(matches!(res, Err(UrlSecurityError::ForbiddenScheme { .. })));
        }
    }

    #[test]
    fn test_userinfo_rejection() {
        let policy = UrlSecurityPolicy::default();
        let urls_with_userinfo = [
            "https://user:pass@example.com/secret",
            "http://admin@example.org/dashboard",
        ];

        for raw in urls_with_userinfo {
            let res = ValidatedUrl::parse(raw, &policy);
            assert_eq!(res, Err(UrlSecurityError::UserinfoForbidden));
        }
    }

    #[test]
    fn test_length_boundary_enforcement() {
        let policy = UrlSecurityPolicy::builder().max_url_length(30).build();
        let short_url = "https://example.com/ok";
        let long_url = "https://example.com/this-is-a-very-long-url-path-that-exceeds-limit";

        assert!(ValidatedUrl::parse(short_url, &policy).is_ok());
        let res = ValidatedUrl::parse(long_url, &policy);
        assert!(matches!(res, Err(UrlSecurityError::TooLong { .. })));
    }

    #[test]
    fn test_invalid_characters_and_null_bytes() {
        let policy = UrlSecurityPolicy::default();
        let with_null = "https://example.com/path\0/evil";
        assert!(matches!(
            ValidatedUrl::parse(with_null, &policy),
            Err(UrlSecurityError::InvalidCharacters { .. })
        ));

        let with_control = "https://example.com/path\u{0007}/bell";
        assert!(matches!(
            ValidatedUrl::parse(with_control, &policy),
            Err(UrlSecurityError::InvalidCharacters { .. })
        ));
    }

    #[test]
    fn test_port_restrictions_and_default_port_stripping() {
        let policy = UrlSecurityPolicy::default();

        // Blocked ports (e.g. SSH 22, Redis 6379)
        assert!(matches!(
            ValidatedUrl::parse("https://example.com:22/repo", &policy),
            Err(UrlSecurityError::BlockedPort { port: 22, .. })
        ));
        assert!(matches!(
            ValidatedUrl::parse("http://example.com:6379/data", &policy),
            Err(UrlSecurityError::BlockedPort { port: 6379, .. })
        ));

        // Allowed custom web ports
        let custom_port = ValidatedUrl::parse("http://example.com:8080/test", &policy).unwrap();
        assert_eq!(custom_port.port(), Some(8080));

        // Default ports stripped
        let http_80 = ValidatedUrl::parse("http://example.com:80/path", &policy).unwrap();
        assert_eq!(http_80.as_str(), "http://example.com/path");
        assert_eq!(http_80.port(), None);
        assert_eq!(http_80.port_or_default(), 80);

        let https_443 = ValidatedUrl::parse("https://example.com:443/path", &policy).unwrap();
        assert_eq!(https_443.as_str(), "https://example.com/path");
        assert_eq!(https_443.port(), None);
        assert_eq!(https_443.port_or_default(), 443);
    }

    #[test]
    fn test_path_and_fragment_normalization() {
        let policy = UrlSecurityPolicy::default();
        let raw = "https://example.com/a//b///c/d#section-heading";
        let parsed = ValidatedUrl::parse(raw, &policy).unwrap();

        assert_eq!(parsed.path(), "/a/b/c/d");
        assert_eq!(parsed.fragment(), None);
        assert_eq!(parsed.as_str(), "https://example.com/a/b/c/d");
    }

    #[test]
    fn test_tracking_params_removal_and_sorting() {
        let policy = UrlSecurityPolicy::default();
        let raw = "https://example.com/articles?utm_source=twitter&topic=rust&utm_medium=social&page=1&fbclid=xyz123";
        let parsed = ValidatedUrl::parse(raw, &policy).unwrap();

        // Only topic and page remain, sorted alphabetically
        assert_eq!(parsed.query(), Some("page=1&topic=rust"));
        assert_eq!(
            parsed.as_str(),
            "https://example.com/articles?page=1&topic=rust"
        );

        // When only tracking params exist, query becomes None
        let only_tracking = "https://example.com/landing?utm_campaign=winter_sale&gclid=abc456";
        let parsed_only = ValidatedUrl::parse(only_tracking, &policy).unwrap();
        assert_eq!(parsed_only.query(), None);
        assert_eq!(parsed_only.as_str(), "https://example.com/landing");
    }

    #[test]
    fn test_ssrf_ipv4_blocklist() {
        let policy = UrlSecurityPolicy::default();
        let blocked_ips = [
            // Loopback
            "127.0.0.1",
            "127.0.0.2",
            "127.255.255.255",
            // Unspecified / Broadcast
            "0.0.0.0",
            "255.255.255.255",
            // RFC 1918 Private
            "10.0.0.1",
            "10.254.1.1",
            "172.16.0.1",
            "172.24.10.5",
            "172.31.255.255",
            "192.168.0.1",
            "192.168.100.50",
            // Link-Local / Cloud Metadata
            "169.254.169.254",
            "169.254.1.1",
            // CGNAT
            "100.64.0.1",
            "100.100.50.2",
            // Multicast & Reserved
            "224.0.0.1",
            "240.0.0.1",
            // Documentation / Benchmarking
            "192.0.2.1",
            "198.51.100.1",
            "203.0.113.1",
            "198.18.0.1",
        ];

        for ip_str in blocked_ips {
            let ip: IpAddr = ip_str.parse().unwrap();
            let res = is_blocked_ip(ip, &policy);
            assert!(
                res.is_err(),
                "Expected IP {ip_str} to be blocked, got {res:?}"
            );

            let url_str = format!("http://{ip_str}/test");
            let url_res = ValidatedUrl::parse(&url_str, &policy);
            assert!(
                url_res.is_err(),
                "Expected URL {url_str} to be blocked, got {url_res:?}"
            );
        }
    }

    #[test]
    fn test_ssrf_ipv6_blocklist() {
        let policy = UrlSecurityPolicy::default();
        let blocked_ipv6s = [
            "::1",                    // Loopback
            "::",                     // Unspecified
            "fc00::1",                // Unique-Local
            "fd12:3456:789a::1",      // Unique-Local
            "fe80::1",                // Link-Local
            "fe80::200:5aee:feaa",    // Link-Local
            "ff02::1",                // Multicast
            "2001:db8::1",            // Documentation
            "::ffff:127.0.0.1",       // IPv4-mapped loopback
            "::ffff:10.0.0.1",        // IPv4-mapped private
            "::ffff:169.254.169.254", // IPv4-mapped cloud metadata
            "::ffff:192.168.1.1",     // IPv4-mapped private
            "2002:7f00:0001::",       // 6to4 mapped loopback (127.0.0.1)
        ];

        for ip_str in blocked_ipv6s {
            let ip: IpAddr = ip_str.parse().unwrap();
            let res = is_blocked_ip(ip, &policy);
            assert!(
                res.is_err(),
                "Expected IPv6 {ip_str} to be blocked, got {res:?}"
            );

            let url_str = format!("http://[{ip_str}]/test");
            let url_res = ValidatedUrl::parse(&url_str, &policy);
            assert!(
                url_res.is_err(),
                "Expected URL {url_str} to be blocked, got {url_res:?}"
            );
        }
    }

    #[test]
    fn test_ssrf_blocked_hostnames_and_metadata() {
        let policy = UrlSecurityPolicy::default();
        let blocked_hosts = [
            "localhost",
            "test.localhost",
            "server.local",
            "internal.corp.lan",
            "myhost.localdomain",
            "metadata.google.internal",
            "metadata",
            "instance-data",
        ];

        for host in blocked_hosts {
            let url_str = format!("http://{host}/resource");
            let res = ValidatedUrl::parse(&url_str, &policy);
            assert!(
                res.is_err(),
                "Expected host '{host}' to be blocked, got {res:?}"
            );
        }
    }

    #[test]
    fn test_alternative_ip_encodings_ssrf_mitigation() {
        let policy = UrlSecurityPolicy::default();
        // 127.0.0.1 represented in various integer/octal/hex formats
        let evasive_hosts = [
            "2130706433",   // Decimal dword for 127.0.0.1
            "0x7f000001",   // Hex dword for 127.0.0.1
            "017700000001", // Octal dword for 127.0.0.1
            "127.1",        // 2-part dotted for 127.0.0.1
            "127.0.1",      // 3-part dotted for 127.0.0.1
            "0x7f.0.0.1",   // Mixed hex/decimal
            "0177.0.0.1",   // Mixed octal/decimal
            "2852039166",   // Decimal dword for 169.254.169.254
            "0xa9fea9fe",   // Hex dword for 169.254.169.254
        ];

        for host in evasive_hosts {
            let url_str = format!("http://{host}/secret");
            let res = ValidatedUrl::parse(&url_str, &policy);
            assert!(
                res.is_err(),
                "Expected alternative IP host '{host}' to be blocked, got {res:?}"
            );
        }
    }

    #[test]
    fn test_relative_link_join() {
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse("https://example.com/docs/intro", &policy).unwrap();

        // Relative path
        let j1 = base.join("getting-started", &policy).unwrap();
        assert_eq!(j1.as_str(), "https://example.com/docs/getting-started");

        // Root relative path
        let j2 = base.join("/api/reference?v=1", &policy).unwrap();
        assert_eq!(j2.as_str(), "https://example.com/api/reference?v=1");

        // Query join
        let j3 = base.join("?page=2&utm_source=mail", &policy).unwrap();
        assert_eq!(j3.as_str(), "https://example.com/docs/intro?page=2");

        // SSRF target in join rejected
        let ssrf_join = base.join("http://169.254.169.254/latest/meta-data/", &policy);
        assert!(ssrf_join.is_err());

        // Dangerous scheme in join rejected
        let js_join = base.join("javascript:alert(1)", &policy);
        assert!(js_join.is_err());
    }

    #[test]
    fn test_origin_domain_and_subpath_utilities() {
        let policy = UrlSecurityPolicy::default();
        let base = ValidatedUrl::parse("https://docs.example.com/v1/guide/", &policy).unwrap();
        let sub =
            ValidatedUrl::parse("https://docs.example.com/v1/guide/installation", &policy).unwrap();
        let sibling = ValidatedUrl::parse("https://docs.example.com/v2/guide/", &policy).unwrap();
        let other_subdomain =
            ValidatedUrl::parse("https://api.example.com/endpoint", &policy).unwrap();
        let external = ValidatedUrl::parse("https://another.org/docs", &policy).unwrap();

        assert_eq!(base.origin(), "https://docs.example.com");
        assert!(base.is_same_origin(&sub));
        assert!(base.is_same_origin(&sibling));
        assert!(!base.is_same_origin(&other_subdomain));
        assert!(!base.is_same_origin(&external));

        assert!(base.is_same_domain(&sub));
        assert!(base.is_same_domain(&other_subdomain));
        assert!(!base.is_same_domain(&external));

        assert!(sub.is_subpath_of(&base));
        assert!(!sibling.is_subpath_of(&base));
        assert!(!external.is_subpath_of(&base));
    }

    #[test]
    fn test_serde_roundtrip_and_ssrf_rejection() {
        let original = ValidatedUrl::parse_default("https://example.com/docs?id=123").unwrap();
        let json = serde_json::to_string(&original).unwrap();
        assert_eq!(json, "\"https://example.com/docs?id=123\"");

        let deserialized: ValidatedUrl = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, original);

        // Deserializing an SSRF target must fail
        let evil_json = "\"http://169.254.169.254/latest/meta-data/\"";
        let res: Result<ValidatedUrl, _> = serde_json::from_str(evil_json);
        assert!(res.is_err(), "Deserializing SSRF target should fail");
    }

    #[tokio::test]
    async fn test_secure_dns_resolver_public_host() {
        let mock = MockDnsResolver::new();
        let public_ip = Ipv4Addr::new(93, 184, 216, 34); // example.com
        let public_sock = SocketAddr::new(IpAddr::V4(public_ip), 443);
        mock.set_host("example.com", vec![public_sock]);

        let resolver =
            SecureDnsResolver::with_resolver(UrlSecurityPolicy::default(), Arc::new(mock));

        let url = ValidatedUrl::parse_default("https://example.com/index.html").unwrap();
        let pinned = resolver.resolve_and_pin(&url).await.unwrap();

        assert_eq!(pinned.host(), "example.com");
        assert_eq!(pinned.port(), 443);
        assert_eq!(pinned.pinned_ip(), IpAddr::V4(public_ip));
        assert_eq!(pinned.socket_addr(), public_sock);
        assert!(!pinned.is_expired());
        assert_eq!(resolver.cached_count().await, 1);

        // Connection verification check
        assert!(pinned.verify_socket_addr(public_sock).is_ok());
        let bad_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)), 443);
        assert!(matches!(
            pinned.verify_socket_addr(bad_sock),
            Err(UrlSecurityError::IpPinningMismatch { .. })
        ));
    }

    #[tokio::test]
    async fn test_secure_dns_resolver_blocks_internal_ip_resolution() {
        let mock = MockDnsResolver::new();
        // Domain resolves to internal RFC 1918 private IP
        mock.set_host(
            "internal.company.com",
            vec![SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)), 443)],
        );

        let resolver =
            SecureDnsResolver::with_resolver(UrlSecurityPolicy::default(), Arc::new(mock));

        let url = ValidatedUrl::parse_default("https://internal.company.com/api").unwrap();
        let res = resolver.resolve_and_pin(&url).await;

        assert!(
            matches!(res, Err(UrlSecurityError::BlockedIp { .. })),
            "Expected resolution to private IP to be rejected, got {res:?}"
        );
    }

    #[tokio::test]
    async fn test_secure_dns_resolver_all_records_validation() {
        let mock = MockDnsResolver::new();
        // Dual-record response: one public IP, but one prohibited loopback IP
        let public_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 443);
        let loopback_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 443);
        mock.set_host("split-dns.evil.com", vec![public_sock, loopback_sock]);

        let resolver =
            SecureDnsResolver::with_resolver(UrlSecurityPolicy::default(), Arc::new(mock));

        let url = ValidatedUrl::parse_default("https://split-dns.evil.com/").unwrap();
        let res = resolver.resolve_and_pin(&url).await;

        // Even though one record is public, the entire resolution must be rejected because
        // another record is internal/loopback!
        assert!(
            matches!(res, Err(UrlSecurityError::BlockedIp { .. })),
            "All-record validation must reject resolution containing any internal address"
        );
    }

    #[tokio::test]
    async fn test_dns_rebinding_defense_and_pinning() {
        let mock = MockDnsResolver::new();
        let public_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 80);
        let metadata_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)), 80);

        // Rebinding sequence: 1st resolution -> public IP, 2nd resolution -> cloud metadata IP
        mock.set_rebinding_sequence(
            "rebind.attacker.com",
            vec![vec![public_sock], vec![metadata_sock]],
        );

        let resolver =
            SecureDnsResolver::with_resolver(UrlSecurityPolicy::default(), Arc::new(mock));

        let url = ValidatedUrl::parse_default("http://rebind.attacker.com/data").unwrap();

        // 1st call: resolves to public IP and pins it in cache
        let pinned_first = resolver.resolve_and_pin(&url).await.unwrap();
        assert_eq!(
            pinned_first.pinned_ip(),
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))
        );

        // 2nd call: cache returns the pinned public IP (preventing rebinding)
        let pinned_second = resolver.resolve_and_pin(&url).await.unwrap();
        assert_eq!(pinned_second.pinned_ip(), pinned_first.pinned_ip());
        assert_eq!(pinned_second.socket_addr(), public_sock);

        // 3rd call: if cache is explicitly cleared or expired, the re-resolution to metadata is BLOCKED!
        resolver.clear_cache().await;
        let rebound_res = resolver.resolve_and_pin(&url).await;
        assert!(
            matches!(rebound_res, Err(UrlSecurityError::BlockedIp { .. })),
            "Rebound IP targeting cloud metadata must be caught and blocked"
        );
    }

    #[tokio::test]
    async fn test_pinned_target_serde_and_ip_literal() {
        let resolver = SecureDnsResolver::new(UrlSecurityPolicy::default());

        // Resolving an IP literal does not query DNS and pins directly
        let ip_url = ValidatedUrl::parse_default("https://1.1.1.1/dns-query").unwrap();
        let pinned = resolver.resolve_and_pin(&ip_url).await.unwrap();

        assert_eq!(pinned.pinned_ip(), IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        assert_eq!(pinned.port(), 443);

        // Serde roundtrip for PinnedTarget
        let json = serde_json::to_string(&pinned).unwrap();
        let deserialized: PinnedTarget = serde_json::from_str(&json).unwrap();
        assert_eq!(pinned, deserialized);
    }
}
