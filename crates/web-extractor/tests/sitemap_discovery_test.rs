//! # Sitemap Discovery and Ingestion Integration Tests
//!
//! Validates automated sitemap discovery, `robots.txt` parsing, XML `<urlset>`, `<sitemapindex>` recursion,
//! RSS/Atom syndication feeds, plain-text sitemaps, Gzip decompression, SSRF security validation,
//! cross-domain filtering, and loop prevention.

use std::collections::HashMap;
use std::io::Write;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use chrono::{TimeZone, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use url::Url;
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::http::transport::{MockHttpTransport, MockResponse};
use web_repo_to_ai_web_extractor::http::WebFetcher;
use web_repo_to_ai_web_extractor::sitemap::{
    decode_xml_entities, is_gzipped, parse_datetime, parse_robots_txt, parse_sitemap,
    ChangeFrequency, SitemapDiscoverer, SitemapDiscoveryConfig, SitemapType,
};

/// Helper to construct a test `SitemapDiscoverer` backed by in-memory `MockHttpTransport` and `MockDnsResolver`.
fn setup_test_discoverer() -> (SitemapDiscoverer, MockHttpTransport, UrlSecurityPolicy) {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = MockDnsResolver::new();
    mock_dns.set_host(
        "example.com",
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 80),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)), 443),
        ],
    );
    mock_dns.set_host(
        "docs.example.com",
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 35)), 80),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 35)), 443),
        ],
    );
    mock_dns.set_host(
        "api.example.com",
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 36)), 80),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 36)), 443),
        ],
    );
    mock_dns.set_host(
        "other-domain.org",
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10)), 80),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 10)), 443),
        ],
    );

    let dns_resolver = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
    let mock_http = MockHttpTransport::new();
    let fetcher = Arc::new(
        WebFetcher::new(policy.clone(), dns_resolver.clone())
            .with_transport(Arc::new(mock_http.clone())),
    );

    let discoverer = SitemapDiscoverer::new(policy.clone(), dns_resolver).with_fetcher(fetcher);

    (discoverer, mock_http, policy)
}

// ============================================================================
// TEST 1: Standard XML <urlset> Parsing
// ============================================================================

#[test]
fn test_parse_standard_xml_urlset() {
    let sitemap_url = Url::parse("https://example.com/sitemap.xml").unwrap();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url>
            <loc>https://example.com/docs/intro</loc>
            <lastmod>2026-11-19T08:30:00Z</lastmod>
            <changefreq>daily</changefreq>
            <priority>0.9</priority>
        </url>
        <url>
            <loc><![CDATA[https://example.com/catalog?category=books&sort=desc]]></loc>
            <lastmod>2026-11-15</lastmod>
            <changefreq>weekly</changefreq>
            <priority>0.7</priority>
        </url>
        <url>
            <loc>https://example.com/about&amp;team</loc>
            <changefreq>never</changefreq>
            <priority>0.3</priority>
        </url>
    </urlset>"#;

    let parsed = parse_sitemap(&sitemap_url, xml.as_bytes()).expect("parse success");
    assert_eq!(parsed.sitemap_type, SitemapType::UrlSet);
    assert_eq!(parsed.entries.len(), 3);
    assert!(parsed.child_sitemaps.is_empty());

    // Entry 1
    let e0 = &parsed.entries[0];
    assert_eq!(e0.url.as_str(), "https://example.com/docs/intro");
    assert_eq!(
        e0.last_modified,
        Some(Utc.with_ymd_and_hms(2026, 11, 19, 8, 30, 0).unwrap())
    );
    assert_eq!(e0.change_frequency, Some(ChangeFrequency::Daily));
    assert_eq!(e0.priority, Some(0.9));

    // Entry 2 (CDATA with query parameters)
    let e1 = &parsed.entries[1];
    assert_eq!(
        e1.url.as_str(),
        "https://example.com/catalog?category=books&sort=desc"
    );
    assert_eq!(
        e1.last_modified,
        Some(Utc.with_ymd_and_hms(2026, 11, 15, 0, 0, 0).unwrap())
    );
    assert_eq!(e1.change_frequency, Some(ChangeFrequency::Weekly));
    assert_eq!(e1.priority, Some(0.7));

    // Entry 3 (XML entity decoding)
    let e2 = &parsed.entries[2];
    assert_eq!(e2.url.as_str(), "https://example.com/about&team");
    assert_eq!(e2.change_frequency, Some(ChangeFrequency::Never));
    assert_eq!(e2.priority, Some(0.3));
}

// ============================================================================
// TEST 2: XML Namespace Prefixes and Tolerant Parsing
// ============================================================================

#[test]
fn test_parse_xml_with_namespaces_and_whitespace() {
    let sitemap_url = Url::parse("https://example.com/sitemap.xml").unwrap();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sitemap:urlset xmlns:sitemap="http://www.sitemaps.org/schemas/sitemap/0.9"
                    xmlns:xhtml="http://www.w3.org/1999/xhtml">
        <sitemap:url>
            <sitemap:loc>
                https://example.com/page-one
            </sitemap:loc>
            <sitemap:lastmod>2026-10-01</sitemap:lastmod>
            <sitemap:priority>1.5</sitemap:priority>
        </sitemap:url>
        <sitemap:url>
            <sitemap:loc>https://example.com/page-two</sitemap:loc>
            <sitemap:changefreq>MONTHLY</sitemap:changefreq>
            <sitemap:priority>-0.5</sitemap:priority>
        </sitemap:url>
    </sitemap:urlset>"#;

    let parsed = parse_sitemap(&sitemap_url, xml.as_bytes()).expect("parse success");
    assert_eq!(parsed.entries.len(), 2);

    // Assert priority clamped: 1.5 -> 1.0, -0.5 -> 0.0
    assert_eq!(parsed.entries[0].priority, Some(1.0));
    assert_eq!(parsed.entries[1].priority, Some(0.0));
    assert_eq!(
        parsed.entries[1].change_frequency,
        Some(ChangeFrequency::Monthly)
    );
}

// ============================================================================
// TEST 3: Gzipped Sitemap Decompression (.xml.gz)
// ============================================================================

#[test]
fn test_parse_gzipped_sitemap() {
    let sitemap_url = Url::parse("https://example.com/sitemap.xml.gz").unwrap();
    let raw_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url><loc>https://example.com/gzipped-page-1</loc></url>
        <url><loc>https://example.com/gzipped-page-2</loc></url>
    </urlset>"#;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(raw_xml.as_bytes()).unwrap();
    let gzipped_bytes = encoder.finish().unwrap();

    assert!(is_gzipped(&gzipped_bytes));

    let parsed = parse_sitemap(&sitemap_url, &gzipped_bytes).expect("decompressed and parsed");
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(
        parsed.entries[0].url.as_str(),
        "https://example.com/gzipped-page-1"
    );
    assert_eq!(
        parsed.entries[1].url.as_str(),
        "https://example.com/gzipped-page-2"
    );
}

// ============================================================================
// TEST 4: Sitemap Index Parsing (<sitemapindex>)
// ============================================================================

#[test]
fn test_parse_sitemap_index_document() {
    let index_url = Url::parse("https://example.com/sitemap_index.xml").unwrap();
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <sitemap>
            <loc>https://example.com/sitemap-articles.xml</loc>
            <lastmod>2026-11-19T00:00:00Z</lastmod>
        </sitemap>
        <sitemap>
            <loc>https://example.com/sitemap-docs.xml.gz</loc>
            <lastmod>2026-11-18T12:00:00Z</lastmod>
        </sitemap>
    </sitemapindex>"#;

    let parsed = parse_sitemap(&index_url, xml.as_bytes()).expect("parse success");
    assert_eq!(parsed.sitemap_type, SitemapType::SitemapIndex);
    assert_eq!(parsed.child_sitemaps.len(), 2);
    assert!(parsed.entries.is_empty());

    assert_eq!(
        parsed.child_sitemaps[0].sitemap_url.as_str(),
        "https://example.com/sitemap-articles.xml"
    );
    assert_eq!(
        parsed.child_sitemaps[1].sitemap_url.as_str(),
        "https://example.com/sitemap-docs.xml.gz"
    );
}

// ============================================================================
// TEST 5: Plain Text Sitemap Parsing
// ============================================================================

#[test]
fn test_parse_plain_text_sitemap() {
    let sitemap_url = Url::parse("https://example.com/sitemap.txt").unwrap();
    let text = r#"
    # Comment line to ignore
    https://example.com/page1
    https://example.com/page2
    # Another comment
    https://example.com/page3
    "#;

    let parsed = parse_sitemap(&sitemap_url, text.as_bytes()).expect("parse success");
    assert_eq!(parsed.sitemap_type, SitemapType::PlainText);
    assert_eq!(parsed.entries.len(), 3);
    assert_eq!(parsed.entries[0].url.as_str(), "https://example.com/page1");
    assert_eq!(parsed.entries[1].url.as_str(), "https://example.com/page2");
    assert_eq!(parsed.entries[2].url.as_str(), "https://example.com/page3");
}

// ============================================================================
// TEST 6: RSS 2.0 Syndication Feed Sitemap Parsing
// ============================================================================

#[test]
fn test_parse_rss_feed_sitemap() {
    let feed_url = Url::parse("https://example.com/feed.xml").unwrap();
    let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
    <rss version="2.0">
        <channel>
            <title>Example Engineering Blog</title>
            <link>https://example.com/</link>
            <item>
                <title>Async Rust in Production</title>
                <link>https://example.com/blog/async-rust</link>
                <pubDate>Thu, 19 Nov 2026 10:00:00 GMT</pubDate>
            </item>
            <item>
                <title>Svelte 5 Runes Explained</title>
                <link>https://example.com/blog/svelte-runes</link>
                <pubDate>Wed, 18 Nov 2026 14:00:00 GMT</pubDate>
            </item>
        </channel>
    </rss>"#;

    let parsed = parse_sitemap(&feed_url, rss.as_bytes()).expect("parse success");
    assert_eq!(parsed.sitemap_type, SitemapType::RssFeed);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(
        parsed.entries[0].url.as_str(),
        "https://example.com/blog/async-rust"
    );
    assert_eq!(
        parsed.entries[0].last_modified,
        Some(Utc.with_ymd_and_hms(2026, 11, 19, 10, 0, 0).unwrap())
    );
    assert_eq!(
        parsed.entries[1].url.as_str(),
        "https://example.com/blog/svelte-runes"
    );
}

// ============================================================================
// TEST 7: Atom Feed Sitemap Parsing
// ============================================================================

#[test]
fn test_parse_atom_feed_sitemap() {
    let feed_url = Url::parse("https://example.com/atom.xml").unwrap();
    let atom = r#"<?xml version="1.0" encoding="utf-8"?>
    <feed xmlns="http://www.w3.org/2005/Atom">
        <title>Atom Feed</title>
        <entry>
            <title>Post Alpha</title>
            <link href="https://example.com/posts/alpha" rel="alternate"/>
            <updated>2026-11-19T12:00:00Z</updated>
        </entry>
        <entry>
            <title>Post Beta</title>
            <link href="https://example.com/posts/beta"/>
            <updated>2026-11-18T10:00:00Z</updated>
        </entry>
    </feed>"#;

    let parsed = parse_sitemap(&feed_url, atom.as_bytes()).expect("parse success");
    assert_eq!(parsed.sitemap_type, SitemapType::AtomFeed);
    assert_eq!(parsed.entries.len(), 2);
    assert_eq!(
        parsed.entries[0].url.as_str(),
        "https://example.com/posts/alpha"
    );
    assert_eq!(
        parsed.entries[0].last_modified,
        Some(Utc.with_ymd_and_hms(2026, 11, 19, 12, 0, 0).unwrap())
    );
    assert_eq!(
        parsed.entries[1].url.as_str(),
        "https://example.com/posts/beta"
    );
}

// ============================================================================
// TEST 8: Robots.txt Parsing with Multiple Directives & Comments
// ============================================================================

#[test]
fn test_parse_robots_txt_directives() {
    let base_url = Url::parse("https://example.com/").unwrap();
    let robots_txt = r#"
    User-agent: *
    Disallow: /admin/
    Disallow: /private/

    # Primary Sitemaps
    Sitemap: https://example.com/sitemap-main.xml
    sitemap: https://example.com/sitemap-products.xml # inline comment
    SITEMAP: /sitemap-relative.xml

    # Duplicate should be ignored
    Sitemap: https://example.com/sitemap-main.xml
    "#;

    let sitemaps = parse_robots_txt(robots_txt, &base_url);
    assert_eq!(sitemaps.len(), 3);
    assert_eq!(sitemaps[0].as_str(), "https://example.com/sitemap-main.xml");
    assert_eq!(
        sitemaps[1].as_str(),
        "https://example.com/sitemap-products.xml"
    );
    assert_eq!(
        sitemaps[2].as_str(),
        "https://example.com/sitemap-relative.xml"
    );
}

// ============================================================================
// TEST 9: End-to-End Discovery via robots.txt and Recursive Index
// ============================================================================

#[tokio::test]
async fn test_e2e_discovery_via_robots_txt_and_sitemap_index() {
    let (discoverer, mock_http, _) = setup_test_discoverer();

    // 1. robots.txt declares an index sitemap
    mock_http.set_response(
        "https://example.com/robots.txt",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: b"User-agent: *\nSitemap: https://example.com/sitemap_index.xml\n".to_vec(),
        },
    );

    // 2. sitemap_index.xml declares 2 child sitemaps
    let index_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <sitemap><loc>https://example.com/sitemaps/core.xml</loc></sitemap>
        <sitemap><loc>https://example.com/sitemaps/blog.xml</loc></sitemap>
    </sitemapindex>"#;
    mock_http.set_response(
        "https://example.com/sitemap_index.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: index_xml.as_bytes().to_vec(),
        },
    );

    // 3. Child sitemap 1: core.xml
    let core_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url>
            <loc>https://example.com/features</loc>
            <priority>1.0</priority>
        </url>
        <url>
            <loc>https://example.com/pricing</loc>
            <priority>0.8</priority>
        </url>
    </urlset>"#;
    mock_http.set_response(
        "https://example.com/sitemaps/core.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: core_xml.as_bytes().to_vec(),
        },
    );

    // 4. Child sitemap 2: blog.xml
    let blog_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url>
            <loc>https://example.com/blog/intro</loc>
            <priority>0.6</priority>
        </url>
    </urlset>"#;
    mock_http.set_response(
        "https://example.com/sitemaps/blog.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: blog_xml.as_bytes().to_vec(),
        },
    );

    let report = discoverer
        .discover("https://example.com/")
        .await
        .expect("discovery succeeded");

    assert_eq!(report.target_domain, "example.com");
    assert_eq!(report.sitemaps_inspected.len(), 3); // index + core + blog
    assert_eq!(report.url_count(), 3);

    // Verify ordering by priority descending: features (1.0) -> pricing (0.8) -> blog/intro (0.6)
    assert_eq!(
        report.unique_urls[0].url.as_str(),
        "https://example.com/features"
    );
    assert_eq!(
        report.unique_urls[1].url.as_str(),
        "https://example.com/pricing"
    );
    assert_eq!(
        report.unique_urls[2].url.as_str(),
        "https://example.com/blog/intro"
    );
}

// ============================================================================
// TEST 10: Well-Known Path Probing Fallback
// ============================================================================

#[tokio::test]
async fn test_well_known_probing_when_no_robots_txt() {
    let (discoverer, mock_http, _) = setup_test_discoverer();

    // robots.txt returns 404
    mock_http.set_response(
        "https://example.com/robots.txt",
        MockResponse {
            status: 404,
            status_text: "Not Found".to_string(),
            headers: HashMap::new(),
            body: b"Not found".to_vec(),
        },
    );

    // Well-known /sitemap.xml returns 200
    let sitemap_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url><loc>https://example.com/fallback-page</loc></url>
    </urlset>"#;
    mock_http.set_response(
        "https://example.com/sitemap.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: sitemap_xml.as_bytes().to_vec(),
        },
    );

    let report = discoverer
        .discover("https://example.com/")
        .await
        .expect("discovery succeeded");

    assert_eq!(report.sitemaps_inspected.len(), 1);
    assert_eq!(
        report.sitemaps_inspected[0].as_str(),
        "https://example.com/sitemap.xml"
    );
    assert_eq!(report.url_count(), 1);
    assert_eq!(
        report.unique_urls[0].url.as_str(),
        "https://example.com/fallback-page"
    );
}

// ============================================================================
// TEST 11: Cross-Domain Security Filtering Defense
// ============================================================================

#[tokio::test]
async fn test_cross_domain_filtering_security() {
    let (discoverer, mock_http, _) = setup_test_discoverer();

    // Sitemap on example.com attempts to inject URLs for other-domain.org
    let malicious_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url><loc>https://example.com/legitimate-page</loc></url>
        <url><loc>https://other-domain.org/injected-page</loc></url>
        <url><loc>https://evil.com/phishing</loc></url>
    </urlset>"#;

    mock_http.set_response(
        "https://example.com/sitemap.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: malicious_xml.as_bytes().to_vec(),
        },
    );

    // Disable robots query and directly probe sitemap.xml
    let config = SitemapDiscoveryConfig::default()
        .with_check_robots_txt(false)
        .with_probe_well_known(true)
        .with_host_policy(true, false); // enforce strict same host

    let discoverer = discoverer.with_config(config);
    let report = discoverer
        .discover("https://example.com/")
        .await
        .expect("discovery completed");

    // Only 1 legitimate URL should be accepted, 2 cross-domain rejected
    assert_eq!(report.url_count(), 1);
    assert_eq!(
        report.unique_urls[0].url.as_str(),
        "https://example.com/legitimate-page"
    );
    assert_eq!(report.filtered_cross_domain_count, 2);
}

// ============================================================================
// TEST 12: Subdomain Policy Support
// ============================================================================

#[tokio::test]
async fn test_subdomain_policy_allowance() {
    let (discoverer, mock_http, _) = setup_test_discoverer();

    let sitemap_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <url><loc>https://example.com/main</loc></url>
        <url><loc>https://docs.example.com/guide</loc></url>
        <url><loc>https://api.example.com/v1/spec</loc></url>
        <url><loc>https://other-domain.org/external</loc></url>
    </urlset>"#;

    mock_http.set_response(
        "https://example.com/sitemap.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: sitemap_xml.as_bytes().to_vec(),
        },
    );

    let config = SitemapDiscoveryConfig::default()
        .with_check_robots_txt(false)
        .with_host_policy(true, true); // allow subdomains!

    let discoverer = discoverer.with_config(config);
    let report = discoverer
        .discover("https://example.com/")
        .await
        .expect("discovery completed");

    // docs.example.com and api.example.com accepted; other-domain.org rejected
    assert_eq!(report.url_count(), 3);
    assert_eq!(report.filtered_cross_domain_count, 1);
}

// ============================================================================
// TEST 13: Circular Sitemap Index Loop Defense
// ============================================================================

#[tokio::test]
async fn test_circular_sitemap_index_loop_prevention() {
    let (discoverer, mock_http, _) = setup_test_discoverer();

    // index-a references index-b
    let xml_a = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <sitemap><loc>https://example.com/index-b.xml</loc></sitemap>
    </sitemapindex>"#;

    // index-b references index-a (cycle!)
    let xml_b = r#"<?xml version="1.0" encoding="UTF-8"?>
    <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        <sitemap><loc>https://example.com/index-a.xml</loc></sitemap>
    </sitemapindex>"#;

    mock_http.set_response(
        "https://example.com/sitemap.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: xml_a.as_bytes().to_vec(),
        },
    );
    mock_http.set_response(
        "https://example.com/index-b.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: xml_b.as_bytes().to_vec(),
        },
    );
    mock_http.set_response(
        "https://example.com/index-a.xml",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: xml_a.as_bytes().to_vec(),
        },
    );

    let config = SitemapDiscoveryConfig::default().with_check_robots_txt(false);
    let discoverer = discoverer.with_config(config);

    // Must terminate cleanly without stack overflow or infinite loop
    let report = discoverer
        .discover("https://example.com/")
        .await
        .expect("cycle handled cleanly");

    assert!(report.sitemaps_inspected.len() <= 3);
}

// ============================================================================
// TEST 14: Datetime and Entity Decoding Utilities
// ============================================================================

#[test]
fn test_sitemap_utility_functions() {
    // Entity decoding
    assert_eq!(
        decode_xml_entities("a &amp; b &lt; c &gt; d"),
        "a & b < c > d"
    );
    assert_eq!(decode_xml_entities("&#38;&#60;&#62;"), "&<>");
    assert_eq!(decode_xml_entities("&#x26;&#x3c;&#x3e;"), "&<>");

    // Datetime parsing
    assert!(parse_datetime("2026-11-19T08:30:00Z").is_some());
    assert!(parse_datetime("2026-11-19").is_some());
    assert!(parse_datetime("2026-11").is_some());
    assert!(parse_datetime("2026").is_some());
    assert!(parse_datetime("Thu, 19 Nov 2026 10:00:00 GMT").is_some());
    assert!(parse_datetime("invalid-date-format").is_none());
}
