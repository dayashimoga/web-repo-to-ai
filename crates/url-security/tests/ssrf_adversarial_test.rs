//! Comprehensive adversarial test suite for SSRF and network security controls (`crates/url-security`).
//!
//! Evaluates:
//! 1. Alternative IPv4 notations (integer dword, octal, hex, multi-base dotted, trailing dots, percent encoding)
//! 2. IPv6 evasions (loopback, unspecified, IPv4-mapped, IPv4-compatible, 6to4, Teredo, zone identifiers RFC 6874)
//! 3. DNS rebinding & split-horizon multi-record responses
//! 4. Time-of-check to time-of-use (TOCTOU) DNS pinning cache immunity
//! 5. Administrative, database, and infrastructure port blocking matrix
//! 6. Redirect hop destination revalidation via `join`
//! 7. Userinfo credential rejection and tracking parameter sanitization

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use web_repo_to_ai_url_security::dns::DnsResolver;
use web_repo_to_ai_url_security::error::UrlSecurityError;
use web_repo_to_ai_url_security::policy::{UrlSecurityPolicy, DEFAULT_BLOCKED_PORTS};
use web_repo_to_ai_url_security::resolver::SecureDnsResolver;
use web_repo_to_ai_url_security::validated_url::ValidatedUrl;

/// Mock DNS resolver with programmable response lists for testing split-horizon and TOCTOU DNS rebinding.
#[derive(Debug, Default)]
struct MockAdversarialResolver {
    /// Mapping from hostname to list of IP addresses.
    records: HashMap<String, Vec<IpAddr>>,
    /// Counter for tracking resolution invocations.
    call_counts: HashMap<String, Arc<AtomicUsize>>,
}

impl MockAdversarialResolver {
    fn new() -> Self {
        Self::default()
    }

    fn add_record(&mut self, host: &str, ips: Vec<IpAddr>) {
        self.records.insert(host.to_ascii_lowercase(), ips);
        self.call_counts
            .insert(host.to_ascii_lowercase(), Arc::new(AtomicUsize::new(0)));
    }

    fn call_count(&self, host: &str) -> usize {
        self.call_counts
            .get(&host.to_ascii_lowercase())
            .map(|c| c.load(Ordering::SeqCst))
            .unwrap_or(0)
    }
}

#[async_trait]
impl DnsResolver for MockAdversarialResolver {
    async fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, UrlSecurityError> {
        let key = host.to_ascii_lowercase();
        if let Some(counter) = self.call_counts.get(&key) {
            counter.fetch_add(1, Ordering::SeqCst);
        }
        if let Some(ips) = self.records.get(&key) {
            Ok(ips.iter().map(|ip| SocketAddr::new(*ip, port)).collect())
        } else {
            Err(UrlSecurityError::DnsResolutionFailed {
                host: host.to_string(),
                details: "No mock DNS record found".to_string(),
            })
        }
    }
}

/// Dynamic flapper DNS resolver that returns public IP on query 1, and loopback IP on query 2.
#[derive(Debug)]
struct FlappingRebindingResolver {
    query_count: Arc<AtomicUsize>,
    initial_ip: IpAddr,
    rebound_ip: IpAddr,
}

impl FlappingRebindingResolver {
    fn new(initial_ip: IpAddr, rebound_ip: IpAddr) -> Self {
        Self {
            query_count: Arc::new(AtomicUsize::new(0)),
            initial_ip,
            rebound_ip,
        }
    }

    fn query_count(&self) -> usize {
        self.query_count.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl DnsResolver for FlappingRebindingResolver {
    async fn resolve(&self, _host: &str, port: u16) -> Result<Vec<SocketAddr>, UrlSecurityError> {
        let count = self.query_count.fetch_add(1, Ordering::SeqCst);
        let chosen_ip = if count == 0 {
            self.initial_ip
        } else {
            self.rebound_ip
        };
        Ok(vec![SocketAddr::new(chosen_ip, port)])
    }
}

// ==============================================================================
// 1. ADVERSARIAL IPv4 NOTATIONS AND EVASIVE ENCODINGS
// ==============================================================================

#[test]
fn test_adversarial_ipv4_alternative_encodings() {
    let policy = UrlSecurityPolicy::default();

    let evasive_cases = [
        // Decimal dwords
        ("http://2130706433/path", "127.0.0.1 decimal dword"),
        ("http://2130706434/test", "127.0.0.2 decimal dword"),
        (
            "http://2852039166/metadata",
            "169.254.169.254 decimal dword",
        ),
        ("http://167772161/api", "10.0.0.1 decimal dword"),
        ("http://3232235521/admin", "192.168.0.1 decimal dword"),
        ("http://2886729729/corp", "172.16.0.1 decimal dword"),
        ("http://1681915905/cgnat", "100.64.0.1 decimal dword"),
        ("http://0/root", "0.0.0.0 single 0 integer"),
        (
            "http://4294967295/broadcast",
            "255.255.255.255 broadcast dword",
        ),
        // Hexadecimal notations
        ("http://0x7f000001/secret", "127.0.0.1 hex dword"),
        ("http://0x7F000001/secret", "127.0.0.1 uppercase hex dword"),
        ("http://0x7f.0.0.1/test", "127.0.0.1 mixed hex/decimal"),
        ("http://0x7f.0x0.0x0.0x1/test", "127.0.0.1 all-hex octets"),
        ("http://0xa9fea9fe/meta", "169.254.169.254 hex dword"),
        (
            "http://0xa9.0xfe.0xa9.0xfe/meta",
            "169.254.169.254 dotted hex",
        ),
        ("http://0x0a000001/private", "10.0.0.1 hex dword"),
        ("http://0xc0a80001/router", "192.168.0.1 hex dword"),
        ("http://0xac100001/intranet", "172.16.0.1 hex dword"),
        ("http://0x0/zero", "0.0.0.0 hex zero"),
        // Octal notations
        ("http://017700000001/loopback", "127.0.0.1 octal dword"),
        ("http://0177.0.0.1/test", "127.0.0.1 mixed octal/decimal"),
        ("http://0177.00.00.01/test", "127.0.0.1 multi-digit octal"),
        (
            "http://0251.0376.0251.0376/meta",
            "169.254.169.254 dotted octal",
        ),
        ("http://012.0.0.1/corp", "10.0.0.1 octal first byte"),
        ("http://0300.0250.0.1/lan", "192.168.0.1 dotted octal"),
        ("http://00/zero", "0.0.0.0 octal zero"),
        // Dotted shorthand (POSIX inet_aton)
        ("http://127.1/short", "127.0.0.1 two-part dotted shorthand"),
        (
            "http://127.0.1/three",
            "127.0.0.1 three-part dotted shorthand",
        ),
        ("http://10.1/internal", "10.0.0.1 two-part dotted shorthand"),
        ("http://0x7f.1/mixed", "127.0.0.1 mixed hex + shorthand"),
        (
            "http://0177.1/octal_short",
            "127.0.0.1 mixed octal + shorthand",
        ),
        // Trailing dot notations
        ("http://127.0.0.1./dot", "127.0.0.1 with trailing dot"),
        ("http://localhost./dot", "localhost with trailing dot"),
        (
            "http://169.254.169.254./dot",
            "169.254.169.254 with trailing dot",
        ),
        (
            "http://2130706433./dot",
            "2130706433 decimal with trailing dot",
        ),
        ("http://0x7f000001./dot", "0x7f000001 hex with trailing dot"),
        // Percent-encoded IP literals
        (
            "http://%31%32%37%2E%30%2E%30%2E%31/encoded",
            "Percent-encoded 127.0.0.1",
        ),
        (
            "http://%31%36%39%2E%32%35%34%2E%31%36%39%2E%32%35%34/meta",
            "Percent-encoded 169.254.169.254",
        ),
    ];

    for (raw_url, desc) in evasive_cases {
        let res = ValidatedUrl::parse(raw_url, &policy);
        assert!(
            res.is_err(),
            "Evasive SSRF target '{desc}' ({raw_url}) must be blocked by ValidatedUrl::parse"
        );
    }
}

// ==============================================================================
// 2. ADVERSARIAL IPv6, SCOPES AND TUNNELING EVASIONS
// ==============================================================================

#[test]
fn test_adversarial_ipv6_scopes_and_tunneling_evasions() {
    let policy = UrlSecurityPolicy::default();

    let ipv6_cases = [
        // Loopback & Unspecified
        ("http://[::1]/admin", "IPv6 loopback ::1"),
        ("http://[::]/zero", "IPv6 unspecified ::"),
        ("http://[0:0:0:0:0:0:0:1]/loop", "IPv6 expanded loopback"),
        (
            "http://[0000:0000:0000:0000:0000:0000:0000:0001]/padded",
            "IPv6 padded loopback",
        ),
        // IPv4-mapped IPv6
        ("http://[::ffff:127.0.0.1]/map", "IPv4-mapped 127.0.0.1"),
        (
            "http://[::ffff:169.254.169.254]/meta",
            "IPv4-mapped 169.254.169.254",
        ),
        ("http://[::ffff:10.0.0.1]/corp", "IPv4-mapped 10.0.0.1"),
        ("http://[::ffff:192.168.1.1]/lan", "IPv4-mapped 192.168.1.1"),
        ("http://[::ffff:172.16.0.1]/b", "IPv4-mapped 172.16.0.1"),
        ("http://[::ffff:7f00:1]/hex", "IPv4-mapped hex 127.0.0.1"),
        (
            "http://[::ffff:a9fe:a9fe]/hex_meta",
            "IPv4-mapped hex metadata",
        ),
        // IPv4-compatible IPv6 (deprecated)
        ("http://[::127.0.0.1]/compat", "IPv4-compatible 127.0.0.1"),
        (
            "http://[::169.254.169.254]/compat_meta",
            "IPv4-compatible metadata",
        ),
        (
            "http://[::10.0.0.1]/compat_corp",
            "IPv4-compatible 10.0.0.1",
        ),
        // 6to4 encapsulation (2002::/16)
        (
            "http://[2002:7f00:0001::]/6to4_loop",
            "6to4 encapsulated 127.0.0.1",
        ),
        (
            "http://[2002:a9fe:a9fe::]/6to4_meta",
            "6to4 encapsulated metadata",
        ),
        (
            "http://[2002:0a00:0001::]/6to4_corp",
            "6to4 encapsulated 10.0.0.1",
        ),
        // Teredo tunneling (2001:0000::/32, bytes 12..16 XOR 0xff)
        // 127.0.0.1 inverted is: (127^255).(0^255).(0^255).(1^255) = 128.255.255.254 = 80ff:feff
        (
            "http://[2001:0000:4136:e378:8000:63bf:80ff:feff]/teredo_loop",
            "Teredo tunnel inverted 127.0.0.1",
        ),
        // 169.254.169.254 inverted is: (169^255).(254^255).(169^255).(254^255) = 86.1.86.1 = 5601:5601
        (
            "http://[2001:0000:4136:e378:8000:63bf:5601:5601]/teredo_meta",
            "Teredo tunnel inverted metadata",
        ),
        // Unique-Local Addresses (fc00::/7)
        ("http://[fc00::1]/ula", "IPv6 Unique-Local fc00::1"),
        ("http://[fd00::1]/ula_fd", "IPv6 Unique-Local fd00::1"),
        (
            "http://[fd12:3456:789a::1]/ula_routed",
            "IPv6 Unique-Local routed",
        ),
        // Link-Local (fe80::/10) with RFC 6874 zone IDs
        ("http://[fe80::1]/linklocal", "IPv6 Link-Local fe80::1"),
        (
            "http://[fe80::200:5aee:feaa]/linklocal_eui",
            "IPv6 Link-Local EUI-64",
        ),
        (
            "http://[fe80::1%eth0]/zone",
            "IPv6 Link-Local with zone index %eth0",
        ),
        (
            "http://[fe80::1%1]/zone_int",
            "IPv6 Link-Local with numeric zone %1",
        ),
        ("http://[::1%lo]/zone_loop", "IPv6 loopback with zone %lo"),
    ];

    for (raw_url, desc) in ipv6_cases {
        let res = ValidatedUrl::parse(raw_url, &policy);
        assert!(
            res.is_err(),
            "Adversarial IPv6 target '{desc}' ({raw_url}) must be rejected"
        );
    }
}

// ==============================================================================
// 3. ADVERSARIAL DNS RESOLUTION: SPLIT-HORIZON & ALL-RECORD REJECTION
// ==============================================================================

#[tokio::test]
async fn test_adversarial_dns_split_horizon_and_all_record_rejection() {
    let policy = UrlSecurityPolicy::default();
    let mut mock = MockAdversarialResolver::new();

    // Adversary registers split-horizon domains resolving to mixed public & private IPs
    mock.add_record(
        "split-loopback.adversary.org",
        vec![
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), // Public IPv4
            IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),     // Loopback IPv4
        ],
    );

    mock.add_record(
        "split-metadata.adversary.org",
        vec![
            IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),         // Public IPv4
            IpAddr::V4(Ipv4Addr::new(169, 254, 169, 254)), // Cloud Metadata IPv4
        ],
    );

    mock.add_record(
        "split-rfc1918.adversary.org",
        vec![
            IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)),     // Public IPv4
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),    // RFC1918 Class A
            IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), // RFC1918 Class C
        ],
    );

    mock.add_record(
        "split-ipv6-loopback.adversary.org",
        vec![
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), // Public IPv4
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)), // IPv6 loopback ::1
        ],
    );

    mock.add_record(
        "split-ipv6-linklocal.adversary.org",
        vec![
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), // Public IPv4
            IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1)), // IPv6 link-local
        ],
    );

    // Legitimate public domain
    mock.add_record(
        "clean.example.com",
        vec![
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 35)),
        ],
    );

    let mock = Arc::new(mock);
    let resolver = SecureDnsResolver::with_resolver(policy.clone(), mock.clone());

    // A. Verify that EVERY split-horizon domain is rejected with BlockedIp
    let test_domains = [
        "https://split-loopback.adversary.org/exploit",
        "https://split-metadata.adversary.org/exploit",
        "https://split-rfc1918.adversary.org/exploit",
        "https://split-ipv6-loopback.adversary.org/exploit",
        "https://split-ipv6-linklocal.adversary.org/exploit",
    ];

    for domain_url in test_domains {
        let valid_url = ValidatedUrl::parse(domain_url, &policy).unwrap();
        let res = resolver.resolve_and_pin(&valid_url).await;
        assert!(
            res.is_err(),
            "Split-horizon domain '{domain_url}' must be rejected because at least one record is private"
        );
        match res.unwrap_err() {
            UrlSecurityError::BlockedIp { ip, category } => {
                assert!(
                    !ip.is_empty(),
                    "Error must identify the blocked IP triggering rejection"
                );
                assert!(
                    !category.is_empty(),
                    "Error must identify the security category"
                );
            }
            other => panic!("Expected BlockedIp, got {other:?}"),
        }
    }

    // B. Clean domain must resolve and pin safely
    let clean_url = ValidatedUrl::parse("https://clean.example.com/docs", &policy).unwrap();
    let pinned = resolver.resolve_and_pin(&clean_url).await.unwrap();
    assert_eq!(pinned.host(), "clean.example.com");
    assert_eq!(
        pinned.pinned_ip(),
        IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))
    );
    assert_eq!(pinned.all_resolved_ips().len(), 2);
    assert_eq!(mock.call_count("clean.example.com"), 1);
}

// ==============================================================================
// 4. ADVERSARIAL TOCTOU DNS REBINDING IMMUNITY
// ==============================================================================

#[tokio::test]
async fn test_adversarial_dns_rebinding_pinning_cache_defense() {
    let policy = UrlSecurityPolicy::default();

    // Adversarial server that responds with public IP first, then rebinds to 127.0.0.1
    let public_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));
    let rebound_ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
    let mock = Arc::new(FlappingRebindingResolver::new(public_ip, rebound_ip));

    let resolver = SecureDnsResolver::with_resolver(policy.clone(), mock.clone());
    let target_url =
        ValidatedUrl::parse("https://rebind-attack.adversary.io/payload", &policy).unwrap();

    // Query 1: Initial resolution
    let first_target = resolver.resolve_and_pin(&target_url).await.unwrap();
    assert_eq!(first_target.pinned_ip(), public_ip);
    assert_eq!(mock.query_count(), 1);

    // Query 2: Immediate subsequent request during active session
    // SecureDnsResolver must serve the unexpired pinned target from cache,
    // refusing to re-resolve and connect to the rebinded 127.0.0.1 loopback!
    let second_target = resolver.resolve_and_pin(&target_url).await.unwrap();
    assert_eq!(second_target.pinned_ip(), public_ip);
    // Crucial invariant: DNS was NOT queried a second time, defeating the rebinding attack
    assert_eq!(
        mock.query_count(),
        1,
        "DNS must not be re-resolved while pinned cache is valid"
    );

    // Socket address verification:
    // Pinned target must verify the original public socket address
    let safe_socket = SocketAddr::new(public_ip, 443);
    assert!(second_target.verify_socket_addr(safe_socket).is_ok());

    // Pinned target must REJECT any connection established to the rebinded socket address
    let evil_socket = SocketAddr::new(rebound_ip, 443);
    let verify_res = second_target.verify_socket_addr(evil_socket);
    assert!(
        verify_res.is_err(),
        "PinnedTarget must reject socket connection to rebinded IP"
    );
}

// ==============================================================================
// 5. ADMINISTRATIVE AND INFRASTRUCTURE PORT SECURITY MATRIX
// ==============================================================================

#[test]
fn test_adversarial_blocked_ports_matrix() {
    let policy = UrlSecurityPolicy::default();

    // Verify all 28 default blocked ports are rejected
    for port in DEFAULT_BLOCKED_PORTS {
        let url_str = format!("http://example.com:{port}/service");
        let res = ValidatedUrl::parse(&url_str, &policy);
        assert!(
            res.is_err(),
            "Port {port} must be blocked by default security policy"
        );
        match res.unwrap_err() {
            UrlSecurityError::BlockedPort { port: p, reason } => {
                assert_eq!(p, *port);
                assert!(reason.contains("blocked by security policy"));
            }
            other => panic!("Expected BlockedPort for port {port}, got {other:?}"),
        }
    }

    // Permitted ports must pass
    let safe_ports = [80, 443, 8080, 8443, 3000, 4000];
    for port in safe_ports {
        let url_str = format!("http://example.com:{port}/resource");
        let res = ValidatedUrl::parse(&url_str, &policy);
        assert!(
            res.is_ok(),
            "Permitted port {port} should pass validation, got {res:?}"
        );
    }
}

// ==============================================================================
// 6. REDIRECT HOP REVALIDATION AND RELATIVE JOIN ATTACKS
// ==============================================================================

#[test]
fn test_adversarial_redirect_hop_join_revalidation() {
    let policy = UrlSecurityPolicy::default();
    let base_url = ValidatedUrl::parse("https://safe-domain.com/landing", &policy).unwrap();

    let malicious_redirects = [
        // Protocol confusion
        ("file:///etc/shadow", "file scheme redirect"),
        ("ftp://internal.mirror/bin", "ftp scheme redirect"),
        ("gopher://127.0.0.1:70/", "gopher scheme redirect"),
        ("javascript:alert(document.cookie)", "javascript redirect"),
        // Direct SSRF redirects
        ("http://127.0.0.1:8080/admin", "loopback redirect"),
        (
            "http://169.254.169.254/latest/meta-data/",
            "cloud metadata redirect",
        ),
        ("http://10.0.0.1/intranet", "RFC 1918 class A redirect"),
        ("http://192.168.1.1/gateway", "RFC 1918 class C redirect"),
        ("http://172.16.0.1/vpn", "RFC 1918 class B redirect"),
        ("http://[::1]:9000/internal", "IPv6 loopback redirect"),
        // Evasive encodings in redirect
        (
            "http://2130706433/admin",
            "decimal integer 127.0.0.1 redirect",
        ),
        ("http://0x7f000001/admin", "hex integer 127.0.0.1 redirect"),
        (
            "http://0177.0.0.1/admin",
            "octal integer 127.0.0.1 redirect",
        ),
        ("http://127.1/admin", "dotted shorthand redirect"),
        // Port confusion in redirect
        ("https://safe-domain.com:22/ssh", "SSH port redirect"),
        ("https://safe-domain.com:6379/redis", "Redis port redirect"),
        (
            "https://safe-domain.com:2375/docker",
            "Docker port redirect",
        ),
        // Control characters & null bytes
        (
            "https://safe-domain.com/path\0/evil",
            "Null byte in redirect",
        ),
        ("https://safe-domain.com/path\r\nevil", "CRLF in redirect"),
    ];

    for (target, desc) in malicious_redirects {
        let res = base_url.join(target, &policy);
        assert!(
            res.is_err(),
            "Malicious redirect '{desc}' ('{target}') must be rejected by base_url.join"
        );
    }

    // Legitimate relative redirects must pass and resolve correctly
    let safe_relatives = [
        ("/dashboard", "https://safe-domain.com/dashboard"),
        ("articles/2026", "https://safe-domain.com/articles/2026"),
        ("../docs", "https://safe-domain.com/docs"),
        ("?page=2", "https://safe-domain.com/landing?page=2"),
    ];

    for (rel, expected) in safe_relatives {
        let joined = base_url.join(rel, &policy).unwrap();
        assert_eq!(joined.as_str(), expected);
    }
}

// ==============================================================================
// 7. USERINFO REJECTION AND TRACKING PARAMETER SANITIZATION
// ==============================================================================

#[test]
fn test_adversarial_userinfo_and_query_sanitization() {
    let policy = UrlSecurityPolicy::default();

    // Embedded credentials in URLs must be rejected
    let userinfo_cases = [
        "https://admin:password123@example.com/secret",
        "http://root:@example.com/api",
        "http://:nopass@example.com/api",
        "https://user%40domain:pass@example.com/",
    ];

    for raw in userinfo_cases {
        let res = ValidatedUrl::parse(raw, &policy);
        assert_eq!(
            res,
            Err(UrlSecurityError::UserinfoForbidden),
            "URL with credentials '{raw}' must be rejected with UserinfoForbidden"
        );
    }

    // Tracking parameter stripping and deterministic sorting
    let dirty_url = "https://example.com/products?utm_source=twitter&category=books&utm_campaign=winter&page=1&fbclid=abc";
    let cleaned = ValidatedUrl::parse(dirty_url, &policy).unwrap();

    // Query must contain only non-tracking parameters, sorted alphabetically
    assert_eq!(cleaned.query(), Some("category=books&page=1"));
    assert_eq!(
        cleaned.as_str(),
        "https://example.com/products?category=books&page=1"
    );
}
