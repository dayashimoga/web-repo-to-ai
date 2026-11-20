//! Comprehensive integration test suite for redirect security, SSRF evasion mitigation,
//! redirect loop detection, and provenance tracking (`crates/web-extractor`).

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use web_repo_to_ai_url_security::{
    MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy, ValidatedUrl,
};
use web_repo_to_ai_web_extractor::error::WebExtractorError;
use web_repo_to_ai_web_extractor::http::{
    MockHttpTransport, MockResponse, RedirectTracker, WebFetcher,
};
use web_repo_to_ai_web_extractor::WebTarget;

// ==============================================================================
// 1. UNIT & CONTRACT TESTS: REDIRECT TRACKER SSRF REVALIDATION
// ==============================================================================

#[test]
fn test_redirect_tracker_blocks_loopback_destinations() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://public-site.com/start", &policy).unwrap();
    let mut tracker = RedirectTracker::new(5);
    tracker.record_initial(&initial_url);

    let loopback_locations = [
        ("http://127.0.0.1:8080/admin", "127.0.0.1 loopback"),
        ("http://127.0.0.2:9000/internal", "127.0.0.2 loopback range"),
        ("http://localhost:3000/api", "localhost hostname"),
        ("http://[::1]:8080/admin", "IPv6 loopback"),
        ("http://2130706433:8080/admin", "decimal integer 127.0.0.1"),
        ("http://0x7f000001:8080/admin", "hex integer 127.0.0.1"),
        ("http://0177.0.0.1:8080/admin", "octal integer 127.0.0.1"),
        ("http://127.1:8080/admin", "dotted shorthand 127.0.0.1"),
    ];

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    for (loc, desc) in loopback_locations {
        let res = tracker.check_and_record(&initial_url, loc, 302, pinned_ip, &policy);
        assert!(
            res.is_err(),
            "Redirect to '{desc}' ({loc}) must be rejected by RedirectTracker"
        );
    }
}

#[test]
fn test_redirect_tracker_blocks_cloud_metadata_services() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://public-site.com/start", &policy).unwrap();
    let mut tracker = RedirectTracker::new(5);
    tracker.record_initial(&initial_url);

    let metadata_locations = [
        (
            "http://169.254.169.254/latest/meta-data/",
            "AWS instance metadata IP",
        ),
        (
            "http://169.254.169.254:80/latest/user-data",
            "AWS metadata with explicit port",
        ),
        (
            "http://metadata.google.internal/computeMetadata/v1/",
            "GCP metadata hostname",
        ),
        (
            "http://metadata/computeMetadata/v1/",
            "Generic metadata hostname",
        ),
        ("http://instance-data/latest/", "AWS instance-data alias"),
        (
            "http://[::ffff:169.254.169.254]/latest/meta-data/",
            "IPv4-mapped metadata",
        ),
        (
            "http://2852039166/latest/meta-data/",
            "decimal integer metadata",
        ),
        (
            "http://0xa9fea9fe/latest/meta-data/",
            "hex integer metadata",
        ),
        (
            "http://0251.0376.0251.0376/latest/meta-data/",
            "octal integer metadata",
        ),
    ];

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    for (loc, desc) in metadata_locations {
        let res = tracker.check_and_record(&initial_url, loc, 301, pinned_ip, &policy);
        assert!(
            res.is_err(),
            "Redirect to '{desc}' ({loc}) must be rejected by RedirectTracker"
        );
    }
}

#[test]
fn test_redirect_tracker_blocks_private_rfc1918_networks() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://public-site.com/start", &policy).unwrap();
    let mut tracker = RedirectTracker::new(5);
    tracker.record_initial(&initial_url);

    let private_locations = [
        ("http://10.0.0.1/admin", "RFC 1918 10.0.0.0/8"),
        ("http://172.16.0.1/admin", "RFC 1918 172.16.0.0/12"),
        ("http://192.168.1.1/admin", "RFC 1918 192.168.0.0/16"),
        ("http://100.64.0.1/carrier", "CGNAT 100.64.0.0/10"),
        ("http://[fc00::1]/internal", "IPv6 Unique-Local fc00::/7"),
        ("http://[fe80::1]/linklocal", "IPv6 Link-Local fe80::/10"),
    ];

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    for (loc, desc) in private_locations {
        let res = tracker.check_and_record(&initial_url, loc, 307, pinned_ip, &policy);
        assert!(
            res.is_err(),
            "Redirect to '{desc}' ({loc}) must be rejected by RedirectTracker"
        );
    }
}

#[test]
fn test_redirect_tracker_blocks_protocol_confusion_schemes() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://public-site.com/start", &policy).unwrap();
    let mut tracker = RedirectTracker::new(5);
    tracker.record_initial(&initial_url);

    let dangerous_schemes = [
        ("file:///etc/passwd", "file scheme"),
        ("ftp://internal.backup/data", "ftp scheme"),
        ("gopher://127.0.0.1:70/", "gopher scheme"),
        ("javascript:alert(document.domain)", "javascript scheme"),
        ("data:text/html,<h1>PWNED</h1>", "data scheme"),
        ("ws://127.0.0.1:8080/ws", "ws scheme"),
    ];

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    for (loc, desc) in dangerous_schemes {
        let res = tracker.check_and_record(&initial_url, loc, 302, pinned_ip, &policy);
        assert!(
            res.is_err(),
            "Redirect to '{desc}' ({loc}) must be rejected by RedirectTracker"
        );
    }
}

#[test]
fn test_redirect_tracker_blocks_dangerous_administrative_ports() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://public-site.com/start", &policy).unwrap();
    let mut tracker = RedirectTracker::new(5);
    tracker.record_initial(&initial_url);

    let blocked_ports = [22, 25, 53, 445, 1433, 2375, 3306, 5432, 6379, 11211];
    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    for port in blocked_ports {
        let loc = format!("https://public-site.com:{port}/admin");
        let res = tracker.check_and_record(&initial_url, &loc, 302, pinned_ip, &policy);
        assert!(
            res.is_err(),
            "Redirect to blocked port {port} must be rejected by RedirectTracker"
        );
    }
}

#[test]
fn test_redirect_tracker_detects_and_aborts_cyclic_loops() {
    let policy = UrlSecurityPolicy::default();
    let url_a = ValidatedUrl::parse("https://example.com/step-a", &policy).unwrap();
    let mut tracker = RedirectTracker::new(10);
    tracker.record_initial(&url_a);

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));

    // A -> B
    let url_b = tracker
        .check_and_record(&url_a, "/step-b", 302, pinned_ip, &policy)
        .unwrap();
    assert_eq!(url_b.as_str(), "https://example.com/step-b");

    // B -> C
    let url_c = tracker
        .check_and_record(&url_b, "/step-c", 302, pinned_ip, &policy)
        .unwrap();
    assert_eq!(url_c.as_str(), "https://example.com/step-c");

    // C -> A (CYCLE DETECTED!)
    let res = tracker.check_and_record(&url_c, "/step-a", 302, pinned_ip, &policy);
    assert!(
        res.is_err(),
        "Cyclic redirect A -> B -> C -> A must be rejected"
    );
    match res.unwrap_err() {
        WebExtractorError::RedirectLoop { url, hops } => {
            assert!(url.contains("step-a"));
            assert_eq!(hops, 3);
        }
        other => panic!("Expected WebExtractorError::RedirectLoop, got {other:?}"),
    }
}

#[test]
fn test_redirect_tracker_enforces_maximum_hop_ceiling() {
    let policy = UrlSecurityPolicy::default();
    let initial_url = ValidatedUrl::parse("https://example.com/hop-0", &policy).unwrap();
    let max_hops = 5;
    let mut tracker = RedirectTracker::new(max_hops);
    tracker.record_initial(&initial_url);

    let pinned_ip = IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34));
    let mut current_url = initial_url;

    // Follow allowed 5 hops
    for i in 1..=max_hops {
        let next_loc = format!("/hop-{i}");
        current_url = tracker
            .check_and_record(&current_url, &next_loc, 302, pinned_ip, &policy)
            .unwrap();
    }
    assert_eq!(tracker.hop_count(), max_hops);

    // 6th hop must exceed limit and fail with TooManyRedirects
    let res = tracker.check_and_record(&current_url, "/hop-6", 302, pinned_ip, &policy);
    assert!(res.is_err(), "6th redirect must exceed max_hops of 5");
    match res.unwrap_err() {
        WebExtractorError::TooManyRedirects { limit } => {
            assert_eq!(limit, max_hops);
        }
        other => panic!("Expected WebExtractorError::TooManyRedirects, got {other:?}"),
    }
}

// ==============================================================================
// 2. END-TO-END PIPELINE TESTS: WEBFETCHER WITH MOCK TRANSPORT
// ==============================================================================

#[tokio::test]
async fn test_web_fetcher_e2e_blocks_redirect_to_cloud_metadata() {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = MockDnsResolver::new();
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            443,
        )],
    );
    let secure_dns = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let mock = Arc::new(MockHttpTransport::new());

    // Configure initial URL that issues a 302 redirect to AWS cloud metadata
    let mut redirect_headers = HashMap::new();
    redirect_headers.insert(
        "location".to_string(),
        "http://169.254.169.254/latest/meta-data/".to_string(),
    );
    mock.set_response(
        "https://example.com/login",
        MockResponse {
            status: 302,
            status_text: "Found".to_string(),
            headers: redirect_headers,
            body: Vec::new(),
        },
    );

    let fetcher = WebFetcher::new(policy.clone(), secure_dns).with_transport(mock);
    let initial_val = ValidatedUrl::parse("https://example.com/login", &policy).unwrap();
    let target = WebTarget::new(initial_val);
    let res = fetcher.fetch(&target).await;

    // Must fail before fetching metadata!
    assert!(
        res.is_err(),
        "WebFetcher must reject redirect to cloud metadata"
    );
}

#[tokio::test]
async fn test_web_fetcher_e2e_blocks_redirect_to_loopback() {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = MockDnsResolver::new();
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            443,
        )],
    );
    let secure_dns = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let mock = Arc::new(MockHttpTransport::new());

    // Initial URL issues a 301 redirect to loopback IP
    let mut redirect_headers = HashMap::new();
    redirect_headers.insert(
        "location".to_string(),
        "http://127.0.0.1:8080/admin/secrets".to_string(),
    );
    mock.set_response(
        "https://example.com/redirect-to-loopback",
        MockResponse {
            status: 301,
            status_text: "Moved Permanently".to_string(),
            headers: redirect_headers,
            body: Vec::new(),
        },
    );

    let fetcher = WebFetcher::new(policy.clone(), secure_dns).with_transport(mock);
    let initial_val =
        ValidatedUrl::parse("https://example.com/redirect-to-loopback", &policy).unwrap();
    let target = WebTarget::new(initial_val);
    let res = fetcher.fetch(&target).await;

    assert!(res.is_err(), "WebFetcher must reject redirect to loopback");
}

#[tokio::test]
async fn test_web_fetcher_e2e_records_clean_multi_hop_provenance() {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = MockDnsResolver::new();
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            443,
        )],
    );
    let secure_dns = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let mock = Arc::new(MockHttpTransport::new());

    // Hop 1: https://example.com/start -> /docs (301)
    let mut h1 = HashMap::new();
    h1.insert("location".to_string(), "/docs".to_string());
    mock.set_response(
        "https://example.com/start",
        MockResponse {
            status: 301,
            status_text: "Moved Permanently".to_string(),
            headers: h1,
            body: Vec::new(),
        },
    );

    // Hop 2: https://example.com/docs -> /docs/v2 (302)
    let mut h2 = HashMap::new();
    h2.insert(
        "location".to_string(),
        "https://example.com/docs/v2".to_string(),
    );
    mock.set_response(
        "https://example.com/docs",
        MockResponse {
            status: 302,
            status_text: "Found".to_string(),
            headers: h2,
            body: Vec::new(),
        },
    );

    // Final destination: https://example.com/docs/v2 -> 200 OK
    let final_html =
        "<html><body><h1>Documentation v2</h1><p>Clean verified content.</p></body></html>";
    mock.set_html("https://example.com/docs/v2", final_html);

    let fetcher = WebFetcher::new(policy.clone(), secure_dns).with_transport(mock);
    let initial_val = ValidatedUrl::parse("https://example.com/start", &policy).unwrap();
    let target = WebTarget::new(initial_val);
    let fetched = fetcher.fetch(&target).await.unwrap();

    assert_eq!(fetched.url.as_str(), "https://example.com/docs/v2");
    assert_eq!(fetched.status, 200);
    assert_eq!(fetched.redirect_history.len(), 2);

    // Inspect provenance of hop 1
    let hop1 = &fetched.redirect_history[0];
    assert_eq!(hop1.from_url.as_str(), "https://example.com/start");
    assert_eq!(hop1.to_url.as_str(), "https://example.com/docs");
    assert_eq!(hop1.status_code, 301);

    // Inspect provenance of hop 2
    let hop2 = &fetched.redirect_history[1];
    assert_eq!(hop2.from_url.as_str(), "https://example.com/docs");
    assert_eq!(hop2.to_url.as_str(), "https://example.com/docs/v2");
    assert_eq!(hop2.status_code, 302);

    // Verify response body
    let body_str = String::from_utf8(fetched.body).unwrap();
    assert!(body_str.contains("Documentation v2"));
}
