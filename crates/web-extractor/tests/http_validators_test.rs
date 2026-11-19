//! # HTTP Caching Validators and Conditional Request Integration Tests
//!
//! Validates RFC 7231 date parsing and formatting, RFC 7232 ETag strong and weak comparisons,
//! RFC 7234 Cache-Control directive parsing and freshness calculations, conditional header injection,
//! and WebFetcher 304 Not Modified / 200 OK execution.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::{
    format_http_date, parse_http_date, Etag, HttpCacheControl, HttpValidator, MockHttpTransport,
    MockResponse, WebFetcher, WebTarget,
};

#[test]
fn test_etag_parsing_and_wire_value() {
    // 1. Strong ETag
    let strong = Etag::parse("\"33a64df551425fcc55e4d42a148795d9f25f89d4\"").unwrap();
    assert_eq!(strong.value, "33a64df551425fcc55e4d42a148795d9f25f89d4");
    assert!(!strong.is_weak);
    assert_eq!(
        strong.wire_value(),
        "\"33a64df551425fcc55e4d42a148795d9f25f89d4\""
    );

    // 2. Weak ETag with uppercase W/
    let weak = Etag::parse("W/\"0815\"").unwrap();
    assert_eq!(weak.value, "0815");
    assert!(weak.is_weak);
    assert_eq!(weak.wire_value(), "W/\"0815\"");

    // 3. Weak ETag with lowercase w/
    let weak_lower = Etag::parse("w/\"my-version-1\"").unwrap();
    assert_eq!(weak_lower.value, "my-version-1");
    assert!(weak_lower.is_weak);

    // 4. Invalid / Empty
    assert!(Etag::parse("").is_none());
    assert!(Etag::parse("   ").is_none());
    assert!(Etag::parse("\"\"").is_none());
}

#[test]
fn test_etag_rfc7232_comparisons() {
    let strong1 = Etag::parse("\"12345\"").unwrap();
    let strong2 = Etag::parse("\"12345\"").unwrap();
    let strong_diff = Etag::parse("\"67890\"").unwrap();
    let weak1 = Etag::parse("W/\"12345\"").unwrap();
    let weak2 = Etag::parse("W/\"12345\"").unwrap();

    // Strong comparison requires: neither is weak, identical opaque tags
    assert!(strong1.matches(&strong2, true));
    assert!(!strong1.matches(&strong_diff, true));
    assert!(!strong1.matches(&weak1, true));
    assert!(!weak1.matches(&strong1, true));
    assert!(!weak1.matches(&weak2, true)); // Weak comparison only for W/

    // Weak comparison requires: identical opaque tags, regardless of weak flags
    assert!(strong1.matches(&strong2, false));
    assert!(strong1.matches(&weak1, false));
    assert!(weak1.matches(&strong1, false));
    assert!(weak1.matches(&weak2, false));
    assert!(!weak1.matches(&strong_diff, false));
}

#[test]
fn test_http_date_parsing_and_formatting() {
    let expected_utc = Utc.with_ymd_and_hms(1994, 11, 6, 8, 49, 37).unwrap();

    // 1. IMF-fixdate
    let imf = "Sun, 06 Nov 1994 08:49:37 GMT";
    let parsed_imf = parse_http_date(imf).expect("Must parse standard IMF-fixdate");
    assert_eq!(parsed_imf, expected_utc);

    // Roundtrip formatting
    let formatted = format_http_date(&parsed_imf);
    assert_eq!(formatted, imf);

    // 2. RFC 850 format
    let rfc850 = "Sunday, 06-Nov-94 08:49:37 GMT";
    let parsed_850 = parse_http_date(rfc850).expect("Must parse RFC 850 date");
    assert_eq!(parsed_850, expected_utc);

    // 3. ANSI C asctime format
    let asctime = "Sun Nov  6 08:49:37 1994";
    let parsed_asctime = parse_http_date(asctime).expect("Must parse asctime format");
    assert_eq!(parsed_asctime, expected_utc);

    // 4. Invalid dates
    assert!(parse_http_date("not-a-date").is_none());
    assert!(parse_http_date("").is_none());
}

#[test]
fn test_cache_control_directives_and_freshness() {
    let header = "public, max-age=3600, must-revalidate, s-maxage=7200";
    let cc = HttpCacheControl::parse(header);

    assert!(cc.is_public);
    assert!(!cc.is_private);
    assert!(!cc.no_cache);
    assert!(!cc.no_store);
    assert!(cc.must_revalidate);
    assert_eq!(cc.max_age, Some(Duration::from_secs(3600)));
    assert_eq!(cc.s_maxage, Some(Duration::from_secs(7200)));

    let t0 = Utc::now();
    let t_fresh = t0 + chrono::Duration::seconds(1800); // 30 min later
    let t_stale = t0 + chrono::Duration::seconds(4000); // > 1 hour later

    assert!(cc.is_fresh(t0, t_fresh));
    assert!(!cc.is_fresh(t0, t_stale));

    // no-cache and no-store never fresh
    let no_cache = HttpCacheControl::parse("no-cache, max-age=86400");
    assert!(no_cache.no_cache);
    assert!(!no_cache.is_fresh(t0, t0 + chrono::Duration::seconds(10)));
    assert!(no_cache.requires_revalidation());

    let no_store = HttpCacheControl::parse("no-store");
    assert!(no_store.no_store);
    assert!(!no_store.is_fresh(t0, t0 + chrono::Duration::seconds(10)));
}

#[test]
fn test_http_validator_from_headers_and_conditional_headers() {
    let mut headers = HashMap::new();
    headers.insert("etag".to_string(), "\"b2c3d4-xyz\"".to_string());
    headers.insert(
        "last-modified".to_string(),
        "Wed, 21 Oct 2026 07:28:00 GMT".to_string(),
    );
    headers.insert("cache-control".to_string(), "max-age=600".to_string());

    let t0 = Utc::now();
    let validator = HttpValidator::from_headers(&headers, t0);

    assert!(validator.etag.is_some());
    assert_eq!(validator.etag.as_ref().unwrap().value, "b2c3d4-xyz");
    assert!(validator.last_modified.is_some());
    assert_eq!(
        validator.raw_last_modified.as_deref(),
        Some("Wed, 21 Oct 2026 07:28:00 GMT")
    );
    assert!(validator.expires_at.is_some());

    // Conditional headers injection
    let cond = validator.conditional_headers();
    assert_eq!(
        cond.get("if-none-match").map(String::as_str),
        Some("\"b2c3d4-xyz\"")
    );
    assert_eq!(
        cond.get("if-modified-since").map(String::as_str),
        Some("Wed, 21 Oct 2026 07:28:00 GMT")
    );
}

#[tokio::test]
async fn test_web_fetcher_conditional_fetch_not_modified_304() {
    let mock_dns = MockDnsResolver::new();
    let host_ip = Ipv4Addr::new(93, 184, 216, 34);
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
    );

    let mock_transport = Arc::new(MockHttpTransport::new());

    // Configure Mock response: Return HTTP 304 Not Modified with ETag header
    let mut resp_headers = HashMap::new();
    resp_headers.insert("etag".to_string(), "\"v1.0-abc\"".to_string());
    resp_headers.insert("cache-control".to_string(), "max-age=1800".to_string());

    let mock_304 = MockResponse {
        status: 304,
        status_text: "Not Modified".to_string(),
        headers: resp_headers,
        body: Vec::new(),
    };

    mock_transport.set_response("https://example.com/docs", mock_304);

    let policy = UrlSecurityPolicy::default();
    let secure_dns = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let fetcher = WebFetcher::new(policy, secure_dns).with_transport(mock_transport.clone());

    let target = WebTarget::from_str_default("https://example.com/docs").unwrap();

    // Prepare existing cached validator
    let mut cached_headers = HashMap::new();
    cached_headers.insert("etag".to_string(), "\"v1.0-abc\"".to_string());
    let validator = HttpValidator::from_headers(&cached_headers, Utc::now());

    // Execute conditional fetch
    let outcome = fetcher
        .fetch_conditional(&target, Some(&validator))
        .await
        .expect("Fetch conditional must succeed");

    assert!(outcome.is_not_modified());
    assert_eq!(outcome.status(), 304);

    let refreshed_validator = outcome
        .validator()
        .expect("Validator must be parsed from 304");
    assert_eq!(refreshed_validator.etag.as_ref().unwrap().value, "v1.0-abc");
    assert!(refreshed_validator.cache_control.is_some());
    assert_eq!(
        refreshed_validator.cache_control.unwrap().max_age,
        Some(Duration::from_secs(1800))
    );
}

#[tokio::test]
async fn test_web_fetcher_conditional_fetch_modified_200() {
    let mock_dns = MockDnsResolver::new();
    let host_ip = Ipv4Addr::new(93, 184, 216, 34);
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
    );

    let mock_transport = Arc::new(MockHttpTransport::new());

    // Configure Mock response: Return HTTP 200 OK with new body and updated ETag
    let mut resp_headers = HashMap::new();
    resp_headers.insert("content-type".to_string(), "text/html".to_string());
    resp_headers.insert("etag".to_string(), "\"v2.0-new\"".to_string());
    resp_headers.insert(
        "last-modified".to_string(),
        "Thu, 19 Nov 2026 10:00:00 GMT".to_string(),
    );

    let body_html = "<html><body><h1>Updated Version 2.0</h1></body></html>";
    let mock_200 = MockResponse {
        status: 200,
        status_text: "OK".to_string(),
        headers: resp_headers,
        body: body_html.as_bytes().to_vec(),
    };

    mock_transport.set_response("https://example.com/docs", mock_200);

    let policy = UrlSecurityPolicy::default();
    let secure_dns = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let fetcher = WebFetcher::new(policy, secure_dns).with_transport(mock_transport.clone());

    let target = WebTarget::from_str_default("https://example.com/docs").unwrap();

    // Prepare stale validator
    let mut stale_headers = HashMap::new();
    stale_headers.insert("etag".to_string(), "\"v1.0-old\"".to_string());
    let validator = HttpValidator::from_headers(&stale_headers, Utc::now());

    // Execute conditional fetch
    let outcome = fetcher
        .fetch_conditional(&target, Some(&validator))
        .await
        .expect("Fetch conditional must succeed");

    assert!(outcome.is_modified());
    assert_eq!(outcome.status(), 200);

    let resp = outcome.into_response().expect("Must have response payload");
    assert_eq!(resp.text().unwrap(), body_html);
    assert_eq!(resp.etag.as_deref(), Some("\"v2.0-new\""));
    assert_eq!(
        resp.last_modified.as_deref(),
        Some("Thu, 19 Nov 2026 10:00:00 GMT")
    );
}
