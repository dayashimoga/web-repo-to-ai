//! # Internal Hyperlink Discovery Integration Tests
//!
//! Validates automated internal hyperlink discovery, base URL resolution,
//! content vs boilerplate navigation classification, pagination detection,
//! asset extension filtering, fragment and tracking parameter stripping,
//! subdomain policies, `rel="nofollow"` policies, SSRF zero-trust rejection,
//! and mock HTTP end-to-end crawling discovery.

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use url::Url;
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::http::transport::{MockHttpTransport, MockResponse};
use web_repo_to_ai_web_extractor::http::WebFetcher;
use web_repo_to_ai_web_extractor::InternalLinkDiscoverer;

/// Helper to construct a test `InternalLinkDiscoverer` backed by in-memory `MockHttpTransport` and `MockDnsResolver`.
fn setup_test_link_discoverer() -> (InternalLinkDiscoverer, MockHttpTransport, UrlSecurityPolicy) {
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
        "blog.example.com",
        vec![
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 36)), 80),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 36)), 443),
        ],
    );
    mock_dns.set_host(
        "external-domain.com",
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

    let discoverer =
        InternalLinkDiscoverer::new(policy.clone(), dns_resolver).with_fetcher(fetcher);

    (discoverer, mock_http, policy)
}

// ============================================================================
// TEST 1: Basic Relative and Absolute Link Resolution
// ============================================================================

#[test]
fn test_basic_relative_and_absolute_link_resolution() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/section/page").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
        <main>
            <a href="/about">About Us</a>
            <a href="subpage">Sub Page</a>
            <a href="../contact">Contact Us</a>
            <a href="https://example.com/pricing" title="Check Our Pricing">Pricing</a>
            <a href="//example.com/legal">Legal Terms</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 5);
    let urls = report.destination_urls();

    assert!(urls.contains(&Url::parse("https://example.com/about").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/section/subpage").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/contact").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/pricing").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/legal").unwrap()));

    let pricing_link = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/pricing")
        .unwrap();
    assert_eq!(pricing_link.anchor_text, "Pricing");
    assert_eq!(pricing_link.title.as_deref(), Some("Check Our Pricing"));
    assert!(pricing_link.is_content_link);
}

// ============================================================================
// TEST 2: Base Href Tag Resolution
// ============================================================================

#[test]
fn test_base_href_tag_resolution() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/blog/2026/article").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <base href="https://example.com/docs/v2/">
        <title>Docs</title>
    </head>
    <body>
        <main>
            <a href="getting-started">Getting Started</a>
            <a href="/root-link">Root Link</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(
        report.base_url_used,
        Url::parse("https://example.com/docs/v2/").unwrap()
    );
    assert_eq!(report.internal_count(), 2);

    let urls = report.destination_urls();
    assert!(urls.contains(&Url::parse("https://example.com/docs/v2/getting-started").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/root-link").unwrap()));
}

// ============================================================================
// TEST 3: Content vs Navigation Boilerplate Classification
// ============================================================================

#[test]
fn test_content_vs_navigation_boilerplate_classification() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <header>
            <nav>
                <a href="/nav-home">Home</a>
                <a href="/nav-features">Features</a>
            </nav>
        </header>

        <aside class="sidebar-wrapper">
            <a href="/sidebar-item">Sidebar Topic</a>
        </aside>

        <main>
            <article>
                <h1>Deep Dive</h1>
                <p>Learn more about our <a href="/content-architecture">Architecture Design</a>.</p>
            </article>
        </main>

        <footer>
            <div class="footer-links">
                <a href="/footer-privacy">Privacy</a>
            </div>
        </footer>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 5);

    let arch_link = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/content-architecture")
        .unwrap();
    assert!(arch_link.is_content_link);

    let home_link = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/nav-home")
        .unwrap();
    assert!(!home_link.is_content_link);

    let sidebar_link = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/sidebar-item")
        .unwrap();
    assert!(!sidebar_link.is_content_link);

    let privacy_link = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/footer-privacy")
        .unwrap();
    assert!(!privacy_link.is_content_link);
}

// ============================================================================
// TEST 4: Pagination Link Detection
// ============================================================================

#[test]
fn test_pagination_link_detection() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/articles").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="/articles/page/2" rel="next">Next Page</a>
            <a href="/articles/page/1" rel="prev">Previous Page</a>
            <a href="/articles/page/3" class="pagination-link">3</a>
            <a href="/articles/page/4">Older</a>
            <a href="/articles/page/5">»</a>
            <a href="/article/regular-link">Read More</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 6);

    let page2 = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/articles/page/2")
        .unwrap();
    assert!(page2.is_pagination);

    let page1 = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/articles/page/1")
        .unwrap();
    assert!(page1.is_pagination);

    let page3 = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/articles/page/3")
        .unwrap();
    assert!(page3.is_pagination);

    let page4 = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/articles/page/4")
        .unwrap();
    assert!(page4.is_pagination);

    let page5 = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/articles/page/5")
        .unwrap();
    assert!(page5.is_pagination);

    let regular = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/article/regular-link")
        .unwrap();
    assert!(!regular.is_pagination);
}

// ============================================================================
// TEST 5: Asset Extension Filtering
// ============================================================================

#[test]
fn test_asset_extension_filtering() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/downloads").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="/files/whitepaper.pdf">Download PDF</a>
            <a href="/files/release.zip">Download ZIP</a>
            <a href="/images/chart.png">View Chart</a>
            <a href="/archives/data.tar.gz">Download Archive</a>
            <a href="/overview">Overview Page</a>
        </main>
    </body>
    </html>
    "#;

    // 1. With exclude_assets: true (default)
    let report = discoverer.discover_from_html(html, &source_url).unwrap();
    assert_eq!(report.internal_count(), 1);
    assert_eq!(report.assets_count, 4);
    assert_eq!(report.internal_links[0].url.path(), "/overview");

    // 2. With exclude_assets: false
    let discoverer_with_assets = discoverer
        .clone()
        .with_config(discoverer.config().clone().with_exclude_assets(false));
    let report_all = discoverer_with_assets
        .discover_from_html(html, &source_url)
        .unwrap();
    assert_eq!(report_all.internal_count(), 5);
    assert_eq!(report_all.assets_count, 0);
}

// ============================================================================
// TEST 6: Anchor Fragment Stripping and In-Page Anchors
// ============================================================================

#[test]
fn test_anchor_fragment_stripping_and_in_page_anchors() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/guide").unwrap();

    let html = r##"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="#introduction">Skip to Intro</a>
            <a href="#quickstart">Quickstart</a>
            <a href="/tutorial#step-one">Tutorial Step 1</a>
            <a href="/tutorial#step-two">Tutorial Step 2</a>
        </main>
    </body>
    </html>
    "##;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    // 2 pure hash anchors skipped
    assert_eq!(report.anchors_count, 2);

    // /tutorial#step-one and /tutorial#step-two normalized to /tutorial and deduplicated!
    assert_eq!(report.internal_count(), 1);
    assert_eq!(
        report.internal_links[0].url.as_str(),
        "https://example.com/tutorial"
    );
}

// ============================================================================
// TEST 7: Marketing Tracking Parameter Stripping
// ============================================================================

#[test]
fn test_marketing_tracking_parameter_stripping() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/shop").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="/item?id=123&utm_source=twitter&utm_medium=cpc&utm_campaign=winter_sale">Product 123</a>
            <a href="/item?id=456&fbclid=IwAR123abc&ref=header_promo">Product 456</a>
            <a href="/blog?gclid=xyz789">Blog Home</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 3);

    let prod123 = report
        .internal_links
        .iter()
        .find(|l| l.anchor_text == "Product 123")
        .unwrap();
    assert_eq!(prod123.url.as_str(), "https://example.com/item?id=123");

    let prod456 = report
        .internal_links
        .iter()
        .find(|l| l.anchor_text == "Product 456")
        .unwrap();
    assert_eq!(prod456.url.as_str(), "https://example.com/item?id=456");

    let blog = report
        .internal_links
        .iter()
        .find(|l| l.anchor_text == "Blog Home")
        .unwrap();
    assert_eq!(blog.url.as_str(), "https://example.com/blog");
}

// ============================================================================
// TEST 8: Subdomain Policy Allowance
// ============================================================================

#[test]
fn test_subdomain_policy() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="https://example.com/about">About</a>
            <a href="https://docs.example.com/v1">Docs Subdomain</a>
            <a href="https://blog.example.com/post">Blog Subdomain</a>
            <a href="https://external-domain.com/partner">External Partner</a>
        </main>
    </body>
    </html>
    "#;

    // 1. Subdomains disallowed by default
    let report_strict = discoverer.discover_from_html(html, &source_url).unwrap();
    assert_eq!(report_strict.internal_count(), 1);
    assert_eq!(report_strict.external_links_count, 3);
    assert_eq!(
        report_strict.internal_links[0].url.as_str(),
        "https://example.com/about"
    );

    // 2. Subdomains allowed
    let discoverer_subdomains = discoverer
        .clone()
        .with_config(discoverer.config().clone().with_allow_subdomains(true));
    let report_sub = discoverer_subdomains
        .discover_from_html(html, &source_url)
        .unwrap();
    assert_eq!(report_sub.internal_count(), 3);
    assert_eq!(report_sub.external_links_count, 1); // external-domain.com still external
}

// ============================================================================
// TEST 9: Respect Rel="nofollow" Policy
// ============================================================================

#[test]
fn test_respect_nofollow_policy() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/forum").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="/trusted">Trusted Page</a>
            <a href="/user-post-1" rel="nofollow">User Generated 1</a>
            <a href="/user-post-2" rel="ugc nofollow external">User Generated 2</a>
        </main>
    </body>
    </html>
    "#;

    // 1. By default, respect_nofollow is false (crawls all internal links regardless of nofollow)
    let report_default = discoverer.discover_from_html(html, &source_url).unwrap();
    assert_eq!(report_default.internal_count(), 3);
    assert_eq!(report_default.dropped_nofollow_count, 0);

    // 2. With respect_nofollow: true
    let discoverer_nofollow = discoverer
        .clone()
        .with_config(discoverer.config().clone().with_respect_nofollow(true));
    let report_filtered = discoverer_nofollow
        .discover_from_html(html, &source_url)
        .unwrap();
    assert_eq!(report_filtered.internal_count(), 1);
    assert_eq!(report_filtered.dropped_nofollow_count, 2);
    assert_eq!(report_filtered.internal_links[0].url.path(), "/trusted");
}

// ============================================================================
// TEST 10: Zero-Trust SSRF and Scheme Rejection
// ============================================================================

#[test]
fn test_zero_trust_ssrf_and_scheme_rejection() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <a href="javascript:alert('pwned')">JS Alert</a>
            <a href="mailto:admin@example.com">Email Us</a>
            <a href="tel:+123456789">Call Us</a>
            <a href="data:text/html,<b>Hello</b>">Data URI</a>
            <a href="http://127.0.0.1/admin">Loopback SSRF</a>
            <a href="http://169.254.169.254/latest/meta-data/">Cloud Metadata SSRF</a>
            <a href="http://192.168.1.1/router">Private LAN SSRF</a>
            <a href="/safe-link">Legitimate Link</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 1);
    assert_eq!(report.internal_links[0].url.path(), "/safe-link");
}

// ============================================================================
// TEST 11: Max Links Per Page Limit
// ============================================================================

#[test]
fn test_max_links_per_page_cap() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/sitemap-like-page").unwrap();

    let mut links_html = String::new();
    for i in 1..=50 {
        links_html.push_str(&format!(r#"<a href="/page/{i}">Page {i}</a>\n"#));
    }

    let html = format!(r#"<!DOCTYPE html><html><body><main>{links_html}</main></body></html>"#);

    let capped_discoverer = discoverer
        .clone()
        .with_config(discoverer.config().clone().with_max_links_per_page(5));

    let report = capped_discoverer
        .discover_from_html(&html, &source_url)
        .unwrap();

    assert_eq!(report.internal_count(), 5);
    assert!(!report.warnings.is_empty());
    assert!(report.warnings[0].contains("Reached maximum internal links per page limit"));
}

// ============================================================================
// TEST 12: Surrounding Context Snippet Extraction
// ============================================================================

#[test]
fn test_surrounding_context_snippet_extraction() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/docs").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <main>
            <p>For high performance async network pipelines, consult the <a href="/tokio-guide">Tokio Integration Guide</a> written by our engineering team.</p>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    assert_eq!(report.internal_count(), 1);
    let link = &report.internal_links[0];
    assert_eq!(link.anchor_text, "Tokio Integration Guide");
    assert!(link.context_snippet.is_some());
    let snippet = link.context_snippet.as_ref().unwrap();
    assert!(snippet.contains("For high performance async network pipelines"));
    assert!(snippet.contains("written by our engineering team"));
}

// ============================================================================
// TEST 13: Deduplication and Priority Sorting
// ============================================================================

#[test]
fn test_deduplication_and_priority_sorting() {
    let (discoverer, _, _) = setup_test_link_discoverer();
    let source_url = Url::parse("https://example.com/").unwrap();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <body>
        <nav>
            <a href="/common-doc">Documentation (Nav)</a>
            <a href="/zebra-nav">Zebra Nav</a>
        </nav>
        <main>
            <p>Read the <a href="/common-doc">Full Documentation</a> inside the article.</p>
            <p>Check <a href="/alpha-content">Alpha Content</a>.</p>
            <a href="/pagination-page" rel="next">Next</a>
        </main>
    </body>
    </html>
    "#;

    let report = discoverer.discover_from_html(html, &source_url).unwrap();

    // 4 unique links: /common-doc, /zebra-nav, /alpha-content, /pagination-page
    assert_eq!(report.internal_count(), 4);

    // /common-doc appeared in nav and main, so its is_content_link should be promoted to true!
    let common = report
        .internal_links
        .iter()
        .find(|l| l.url.path() == "/common-doc")
        .unwrap();
    assert!(common.is_content_link);

    // Order: content links first, then pagination, then nav/boilerplate
    let paths: Vec<&str> = report.internal_links.iter().map(|l| l.url.path()).collect();
    assert_eq!(paths[0], "/alpha-content");
    assert_eq!(paths[1], "/common-doc");
    assert_eq!(paths[2], "/pagination-page");
    assert_eq!(paths[3], "/zebra-nav");
}

// ============================================================================
// TEST 14: End-to-End Discovery from URL via Mock Transport
// ============================================================================

#[tokio::test]
async fn test_discover_from_url_e2e_with_mock() {
    let (discoverer, mock_http, _) = setup_test_link_discoverer();

    let html = r#"
    <!DOCTYPE html>
    <html>
    <head><title>Mocked Site</title></head>
    <body>
        <main>
            <a href="/docs/intro">Intro Docs</a>
            <a href="/api/reference">API Reference</a>
            <a href="https://external-domain.com/blog">External Blog</a>
        </main>
    </body>
    </html>
    "#;

    mock_http.set_response(
        "https://example.com/portal",
        MockResponse {
            status: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            body: html.as_bytes().to_vec(),
        },
    );

    let report = discoverer
        .discover_from_url("https://example.com/portal")
        .await
        .unwrap();

    assert_eq!(report.internal_count(), 2);
    assert_eq!(report.external_links_count, 1);
    assert_eq!(report.source_url.as_str(), "https://example.com/portal");

    let urls = report.destination_urls();
    assert!(urls.contains(&Url::parse("https://example.com/docs/intro").unwrap()));
    assert!(urls.contains(&Url::parse("https://example.com/api/reference").unwrap()));
}
