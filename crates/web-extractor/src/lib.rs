//! # Web Extractor (`web-repo-to-ai-web-extractor`)
//!
//! Extracts clean, structured content from web documentation and articles:
//! - Hop-by-hop HTTP fetching with IP pinning and redirect revalidation
//! - Bounded streaming body acquisition
//! - Target URL classification and link resolution
//! - HTML DOM parsing and boilerplate reduction
//! - Documentation hierarchy and code block extraction
//! - Dynamic browser fallback integration for SPA hydration

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod article;
pub mod browser;
pub mod compression;
pub mod docs;
pub mod dom;
pub mod elements;
pub mod error;
pub mod http;
pub mod url;

pub use article::{
    ArticleExtractor, ExtractedArticle, ReadabilityMetrics, ReadabilityOptions, ReadabilityScorer,
};
pub use browser::{
    BlockedSubresource, BrowserRedirect, BrowserRenderRequest, BrowserRenderResponse,
    BrowserTiming, BrowserWorkerClient, BrowserWorkerError, BrowserWorkerHealth, ConsoleMessage,
    DynamicExtractorOptions, DynamicProvenance, DynamicWebExtractor, ExtractedWebpage,
    ExtractionMethod, ExtractionStrategy, PageError, SpaDetector, Viewport, WaitUntil,
    DEFAULT_CLIENT_TIMEOUT, DEFAULT_MIN_CONTENT_DENSITY, DEFAULT_MIN_STATIC_TEXT_LENGTH,
};
pub use compression::{BodyLimits, BoundedDecompressor, ContentEncoding};
pub use docs::{
    Admonition, AdmonitionType, BreadcrumbItem, DocExtractor, DocFramework, DocLink, DocPagination,
    DocSection, DocVersionInfo, ExtractedDocumentation, TocItem,
};
pub use dom::{
    BoilerplateCleaner, BoilerplatePolicy, CleanedDocument, DocumentMetadata, DomParser,
    LinkResolver,
};
pub use elements::{
    detect_syntax, normalize_language_tag, CodeBlockExtractor, ElementsExtractor, ElementsOptions,
    ExtractedCodeBlock, ExtractedElements, ExtractedLink, ExtractedTable, LinkExtractor, LinkType,
    TableAlignment, TableExtractor, ASSET_EXTENSIONS,
};
pub use error::WebExtractorError;
pub use http::{
    BodyByteStream, FetchedResponse, HttpTransport, MockHttpTransport, MockResponse, RedirectHop,
    RedirectTracker, ReqwestTransport, TransportResponse, WebFetcher, DEFAULT_ACCEPT_ENCODING,
    DEFAULT_CHUNK_READ_TIMEOUT, DEFAULT_CONNECT_TIMEOUT, DEFAULT_MAX_BODY_BYTES,
    DEFAULT_MAX_REDIRECTS, DEFAULT_REQUEST_TIMEOUT, DEFAULT_TOTAL_TIMEOUT, DEFAULT_USER_AGENT,
};
pub use url::{classify_url, WebPageType, WebTarget};

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::io::Write;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
    use std::time::Duration;
    use web_repo_to_ai_url_security::{
        MockDnsResolver, SecureDnsResolver, UrlSecurityError, UrlSecurityPolicy, ValidatedUrl,
    };

    #[test]
    fn test_web_extractor_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-web-extractor");
    }

    fn setup_test_fetcher(
        mock_dns: MockDnsResolver,
        mock_http: MockHttpTransport,
    ) -> (WebFetcher, UrlSecurityPolicy) {
        let policy = UrlSecurityPolicy::default();
        let dns_resolver = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
        let fetcher =
            WebFetcher::new(policy.clone(), dns_resolver).with_transport(Arc::new(mock_http));
        (fetcher, policy)
    }

    #[tokio::test]
    async fn test_successful_direct_fetch() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let html_content =
            "<html><head><title>Test</title></head><body><h1>Hello World</h1></body></html>";
        mock_http.set_html("https://example.com/docs", html_content);

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://example.com/docs").unwrap();

        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.status_text, "OK");
        assert!(resp.is_success());
        assert!(resp.is_html());
        assert!(!resp.is_redirect());
        assert_eq!(resp.redirect_count(), 0);
        assert_eq!(resp.text().unwrap(), html_content);
        assert_eq!(resp.pinned_ip, IpAddr::V4(host_ip));
        assert_eq!(resp.url.as_str(), "https://example.com/docs");
        assert_eq!(resp.initial_url.as_str(), "https://example.com/docs");
        assert_eq!(
            resp.header("content-type"),
            Some("text/html; charset=utf-8")
        );
    }

    #[tokio::test]
    async fn test_hop_by_hop_redirect_resolution() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let ip_origin = Ipv4Addr::new(93, 184, 216, 34);
        let ip_dest = Ipv4Addr::new(104, 18, 20, 50);

        mock_dns.set_host(
            "start.example.com",
            vec![SocketAddr::new(IpAddr::V4(ip_origin), 443)],
        );
        mock_dns.set_host(
            "docs.target.com",
            vec![SocketAddr::new(IpAddr::V4(ip_dest), 443)],
        );

        // 301 redirect from start.example.com to docs.target.com
        mock_http.set_redirect(
            "https://start.example.com/old-path",
            "https://docs.target.com/new-path",
            301,
        );
        mock_http.set_html(
            "https://docs.target.com/new-path",
            "<h1>Migrated Documentation</h1>",
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://start.example.com/old-path").unwrap();

        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.url.as_str(), "https://docs.target.com/new-path");
        assert_eq!(
            resp.initial_url.as_str(),
            "https://start.example.com/old-path"
        );
        assert_eq!(resp.redirect_count(), 1);

        let hop = &resp.redirect_history[0];
        assert_eq!(hop.from_url.as_str(), "https://start.example.com/old-path");
        assert_eq!(hop.to_url.as_str(), "https://docs.target.com/new-path");
        assert_eq!(hop.status_code, 301);
        assert_eq!(hop.pinned_ip, IpAddr::V4(ip_origin));
        assert_eq!(resp.pinned_ip, IpAddr::V4(ip_dest));
    }

    #[tokio::test]
    async fn test_redirect_loop_detection() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "loop.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        // A -> B -> A loop
        mock_http.set_redirect("https://loop.example.com/a", "/b", 302);
        mock_http.set_redirect("https://loop.example.com/b", "/a", 302);

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://loop.example.com/a").unwrap();

        let err = fetcher.fetch(&target).await.unwrap_err();
        assert!(
            matches!(err, WebExtractorError::RedirectLoop { hops: 2, .. }),
            "Expected RedirectLoop error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_too_many_redirects_limit() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "chain.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        // Chain /1 -> /2 -> /3 -> /4
        mock_http.set_redirect("https://chain.example.com/1", "/2", 302);
        mock_http.set_redirect("https://chain.example.com/2", "/3", 302);
        mock_http.set_redirect("https://chain.example.com/3", "/4", 302);
        mock_http.set_html("https://chain.example.com/4", "Finished");

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        fetcher = fetcher.with_max_redirects(2); // limit to 2 hops

        let target = WebTarget::from_str_default("https://chain.example.com/1").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(err, WebExtractorError::TooManyRedirects { limit: 2 }),
            "Expected TooManyRedirects limit error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_redirect_to_private_ip_is_blocked_by_ssrf() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let public_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "safe.example.com",
            vec![SocketAddr::new(IpAddr::V4(public_ip), 443)],
        );

        // Redirects to loopback / private IP (e.g. 127.0.0.1 or 169.254.169.254)
        mock_http.set_redirect(
            "https://safe.example.com/bounce",
            "http://169.254.169.254/latest/meta-data/",
            302,
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://safe.example.com/bounce").unwrap();

        let err = fetcher.fetch(&target).await.unwrap_err();

        // Must be caught by SSRF validation on the redirect hop!
        assert!(
            matches!(
                err,
                WebExtractorError::UrlSecurity(
                    UrlSecurityError::BlockedIp { .. } | UrlSecurityError::BlockedHost { .. }
                )
            ),
            "Expected SSRF error on redirect hop, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_redirect_to_invalid_scheme_is_blocked() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let public_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "evil.example.com",
            vec![SocketAddr::new(IpAddr::V4(public_ip), 443)],
        );

        mock_http.set_redirect(
            "https://evil.example.com/xss",
            "javascript:alert(document.cookie)",
            302,
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://evil.example.com/xss").unwrap();

        let err = fetcher.fetch(&target).await.unwrap_err();
        assert!(
            matches!(
                err,
                WebExtractorError::UrlSecurity(UrlSecurityError::ForbiddenScheme { .. })
            ),
            "Expected ForbiddenScheme error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_missing_location_header_error() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let public_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "broken.example.com",
            vec![SocketAddr::new(IpAddr::V4(public_ip), 443)],
        );

        // 302 without Location header
        mock_http.set_response(
            "https://broken.example.com/broken-redirect",
            MockResponse {
                status: 302,
                status_text: "Found".to_string(),
                headers: HashMap::new(),
                body: Vec::new(),
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target =
            WebTarget::from_str_default("https://broken.example.com/broken-redirect").unwrap();

        let err = fetcher.fetch(&target).await.unwrap_err();
        assert!(
            matches!(
                err,
                WebExtractorError::MissingLocationHeader { status: 302, .. }
            ),
            "Expected MissingLocationHeader error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_payload_too_large_ceiling() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let public_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "huge.example.com",
            vec![SocketAddr::new(IpAddr::V4(public_ip), 443)],
        );

        let large_payload = vec![b'A'; 2048]; // 2048 bytes
        mock_http.set_response(
            "https://huge.example.com/data",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers: HashMap::new(),
                body: large_payload,
            },
        );

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        fetcher = fetcher.with_max_body_bytes(1024); // ceiling is 1024 bytes

        let target = WebTarget::from_str_default("https://huge.example.com/data").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(
                err,
                WebExtractorError::PayloadTooLarge {
                    actual: 2048,
                    maximum: 1024
                }
            ),
            "Expected PayloadTooLarge error, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_fetch_with_gzip_decompression() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "compressed.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let html = "<html><body><h1>Gzip Extracted Article Content</h1></body></html>".repeat(20);
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(html.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "gzip".to_string());
        headers.insert("content-length".to_string(), compressed.len().to_string());

        mock_http.set_response(
            "https://compressed.example.com/gzip-page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: compressed.clone(),
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target =
            WebTarget::from_str_default("https://compressed.example.com/gzip-page").unwrap();
        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, html.as_bytes());
        assert_eq!(resp.raw_body_size, compressed.len());
        assert_eq!(resp.content_encoding.as_deref(), Some("gzip"));
    }

    #[tokio::test]
    async fn test_fetch_with_brotli_decompression() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "brotli.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let html = "<html><body><h1>Brotli Documentation Payload</h1></body></html>".repeat(15);
        let mut compressed = Vec::new();
        {
            let mut writer = brotli::CompressorWriter::new(&mut compressed, 4096, 6, 22);
            writer.write_all(html.as_bytes()).unwrap();
        }

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "br".to_string());
        headers.insert("content-length".to_string(), compressed.len().to_string());

        mock_http.set_response(
            "https://brotli.example.com/brotli-page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: compressed.clone(),
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://brotli.example.com/brotli-page").unwrap();
        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, html.as_bytes());
        assert_eq!(resp.raw_body_size, compressed.len());
        assert_eq!(resp.content_encoding.as_deref(), Some("br"));
    }

    #[tokio::test]
    async fn test_fetch_with_zstd_decompression() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "zstd.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let html = "<html><body><h1>Zstandard Fast Compression Page</h1></body></html>".repeat(15);
        let compressed = zstd::encode_all(html.as_bytes(), 3).unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "zstd".to_string());
        headers.insert("content-length".to_string(), compressed.len().to_string());

        mock_http.set_response(
            "https://zstd.example.com/zstd-page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: compressed.clone(),
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://zstd.example.com/zstd-page").unwrap();
        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, html.as_bytes());
        assert_eq!(resp.raw_body_size, compressed.len());
        assert_eq!(resp.content_encoding.as_deref(), Some("zstd"));
    }

    #[tokio::test]
    async fn test_fetch_with_deflate_decompression() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "deflate.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let html = "<html><body><h1>Deflate Wrapped Content Payload</h1></body></html>".repeat(15);
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(html.as_bytes()).unwrap();
        let compressed = encoder.finish().unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "deflate".to_string());
        headers.insert("content-length".to_string(), compressed.len().to_string());

        mock_http.set_response(
            "https://deflate.example.com/deflate-page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: compressed.clone(),
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target =
            WebTarget::from_str_default("https://deflate.example.com/deflate-page").unwrap();
        let resp = fetcher.fetch(&target).await.unwrap();

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, html.as_bytes());
        assert_eq!(resp.raw_body_size, compressed.len());
        assert_eq!(resp.content_encoding.as_deref(), Some("deflate"));
    }

    #[tokio::test]
    async fn test_fetch_compression_bomb_blocked() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "bomb.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        // 100 KB of zeros compresses to ~100 bytes
        let zeros = vec![0u8; 100 * 1024];
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
        encoder.write_all(&zeros).unwrap();
        let bomb_payload = encoder.finish().unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "gzip".to_string());
        headers.insert("content-length".to_string(), bomb_payload.len().to_string());

        mock_http.set_response(
            "https://bomb.example.com/bomb",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: bomb_payload,
            },
        );

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let limits = BodyLimits::default()
            .with_max_expansion_ratio(5.0)
            .with_min_ratio_check_bytes(1024);
        fetcher = fetcher.with_body_limits(limits);

        let target = WebTarget::from_str_default("https://bomb.example.com/bomb").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(err, WebExtractorError::DecompressionBomb { .. }),
            "Expected DecompressionBomb error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_fetch_content_length_early_bailout() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "huge-header.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        // Header declares 50 MB, exceeding 1 MB limit
        headers.insert("content-length".to_string(), "52428800".to_string());

        mock_http.set_response(
            "https://huge-header.example.com/page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: vec![b'A'; 100], // body is small, but declared header is oversized
            },
        );

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        fetcher = fetcher.with_max_body_bytes(1024 * 1024); // 1 MB limit

        let target = WebTarget::from_str_default("https://huge-header.example.com/page").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(
                err,
                WebExtractorError::PayloadTooLarge {
                    actual: 52428800,
                    maximum: 1048576,
                }
            ),
            "Expected PayloadTooLarge early exit, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_fetch_content_length_mismatch_detected() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "mismatch.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        // Declared length is 1000 bytes, but body has only 200 bytes
        headers.insert("content-length".to_string(), "1000".to_string());

        mock_http.set_response(
            "https://mismatch.example.com/page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: vec![b'X'; 200],
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://mismatch.example.com/page").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert_eq!(
            err,
            WebExtractorError::ContentLengthMismatch {
                expected: 1000,
                actual: 200,
            }
        );
    }

    #[tokio::test]
    async fn test_fetch_total_timeout_aborted() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "slow.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        mock_http.set_html("https://slow.example.com/delayed", "<html>Too slow</html>");
        // Simulate a 200ms transport response delay
        mock_http.set_delay(
            "https://slow.example.com/delayed",
            Duration::from_millis(200),
        );

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        // Hard timeout is 20ms
        fetcher = fetcher.with_total_timeout(Duration::from_millis(20));

        let target = WebTarget::from_str_default("https://slow.example.com/delayed").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(err, WebExtractorError::Timeout { .. }),
            "Expected Timeout error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_fetch_chunk_read_timeout_aborted() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "drip.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        mock_http.set_html(
            "https://drip.example.com/slowloris",
            "<html>Slow chunk stream</html>",
        );
        // Simulate a 200ms inter-chunk delay
        mock_http.set_chunk_delay(
            "https://drip.example.com/slowloris",
            Duration::from_millis(200),
        );

        let (mut fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        // Chunk read timeout is 20ms
        fetcher = fetcher.with_chunk_read_timeout(Duration::from_millis(20));

        let target = WebTarget::from_str_default("https://drip.example.com/slowloris").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert!(
            matches!(err, WebExtractorError::Timeout { .. }),
            "Expected Timeout error, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_fetch_unsupported_content_encoding() {
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "unsupported.example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());
        headers.insert("content-encoding".to_string(), "compress".to_string());
        headers.insert("content-length".to_string(), "10".to_string());

        mock_http.set_response(
            "https://unsupported.example.com/page",
            MockResponse {
                status: 200,
                status_text: "OK".to_string(),
                headers,
                body: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            },
        );

        let (fetcher, _) = setup_test_fetcher(mock_dns, mock_http);
        let target = WebTarget::from_str_default("https://unsupported.example.com/page").unwrap();
        let err = fetcher.fetch(&target).await.unwrap_err();

        assert_eq!(
            err,
            WebExtractorError::UnsupportedContentEncoding {
                encoding: "compress".to_string(),
            }
        );
    }

    #[test]
    fn test_dom_parser_article_with_nav_footer_sidebar_pruning() {
        let html = r##"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <title>Async Rust in Production</title>
                <meta name="description" content="Lessons from deploying Tokio and Axum at scale.">
                <!-- Analytics script -->
                <script>window.dataLayer = [];</script>
                <style>.promo { background: yellow; }</style>
            </head>
            <body>
                <header class="site-header">
                    <nav class="main-menu">
                        <a href="/">Home</a>
                        <a href="/blog">Blog</a>
                        <a href="/pricing">Pricing</a>
                    </nav>
                </header>
                <aside class="sidebar-ad banner">
                    <p>Sponsored: Try Cloud Hosting</p>
                </aside>
                <main class="post-content">
                    <h1>Async Rust in Production</h1>
                    <p>Building high-throughput microservices requires strict resource bounds.</p>
                    <h2>Key Architecture Tenets</h2>
                    <ul>
                        <li>Memory bounding on streams</li>
                        <li>Backpressure propagation</li>
                    </ul>
                </main>
                <div class="social-share-buttons">
                    <button>Tweet</button>
                    <button>Share</button>
                </div>
                <footer class="site-footer">
                    <p>Copyright 2026. All rights reserved.</p>
                </footer>
            </body>
            </html>
        "##;

        let parser = DomParser::default();
        let base_url = ValidatedUrl::parse_default("https://example.com/blog/async-rust").unwrap();
        let doc = parser.parse(html, Some(&base_url)).unwrap();

        assert_eq!(
            doc.metadata.title.as_deref(),
            Some("Async Rust in Production")
        );
        assert_eq!(
            doc.metadata.description.as_deref(),
            Some("Lessons from deploying Tokio and Axum at scale.")
        );
        assert!(doc.has_semantic_main);

        // Verify boilerplate is cleanly stripped
        assert!(!doc.cleaned_html.contains("site-header"));
        assert!(!doc.cleaned_html.contains("main-menu"));
        assert!(!doc.cleaned_html.contains("Sponsored: Try Cloud Hosting"));
        assert!(!doc.cleaned_html.contains("sidebar-ad"));
        assert!(!doc.cleaned_html.contains("social-share-buttons"));
        assert!(!doc.cleaned_html.contains("site-footer"));
        assert!(!doc.cleaned_html.contains("window.dataLayer"));

        // Verify core article is preserved
        assert!(doc
            .cleaned_html
            .contains("<h1>Async Rust in Production</h1>"));
        assert!(doc.cleaned_html.contains(
            "<p>Building high-throughput microservices requires strict resource bounds.</p>"
        ));
        assert!(doc
            .cleaned_html
            .contains("<li>Memory bounding on streams</li>"));

        // Verify plain text output
        assert!(doc.cleaned_text.contains("Async Rust in Production"));
        assert!(doc
            .cleaned_text
            .contains("Building high-throughput microservices requires strict resource bounds."));
        assert!(doc.cleaned_text.contains("Memory bounding on streams"));
        assert!(!doc.cleaned_text.contains("Sponsored"));
        assert!(!doc.cleaned_text.contains("Copyright 2026"));
    }

    #[test]
    fn test_dom_parser_exact_code_and_table_preservation() {
        let code_html = "pub async fn handle() -&gt; Result&lt;(), Error&gt; {\n    let stream = fetch();\n    pin_mut!(stream);\n    Ok(())\n}";
        let expected_code_text = "pub async fn handle() -> Result<(), Error> {\n    let stream = fetch();\n    pin_mut!(stream);\n    Ok(())\n}";
        let html = format!(
            r##"
            <article class="documentation">
                <h1>API Specification</h1>
                <p>Example implementation:</p>
                <pre><code class="language-rust">{}</code></pre>
                <table>
                    <thead>
                        <tr><th>Method</th><th>Endpoint</th></tr>
                    </thead>
                    <tbody>
                        <tr><td>GET</td><td>/api/v1/jobs</td></tr>
                    </tbody>
                </table>
            </article>
        "##,
            code_html
        );

        let parser = DomParser::default();
        let doc = parser.parse(&html, None).unwrap();

        // Exact code block preservation
        assert!(doc.cleaned_html.contains(code_html));
        assert!(doc.cleaned_text.contains(expected_code_text));

        // Exact table structure preservation
        assert!(doc.cleaned_html.contains("<table>"));
        assert!(doc.cleaned_html.contains("<th>Method</th>"));
        assert!(doc.cleaned_html.contains("<td>/api/v1/jobs</td>"));
    }

    #[test]
    fn test_dom_parser_base_url_and_relative_link_rewrite() {
        let html = r##"
            <html>
            <head>
                <base href="https://docs.rs/tokio/latest/tokio/">
            </head>
            <body>
                <main>
                    <h1>Tokio API</h1>
                    <p>See <a href="sync/struct.Mutex.html">Mutex</a> or <a href="../index.html">Crate Root</a> or <a href="#methods">Methods</a>.</p>
                    <img src="logo.png" alt="Tokio Logo">
                </main>
            </body>
            </html>
        "##;

        let parser = DomParser::default();
        let doc = parser.parse(html, None).unwrap();

        assert_eq!(
            doc.metadata.base_url.as_deref(),
            Some("https://docs.rs/tokio/latest/tokio/")
        );
        assert!(doc
            .cleaned_html
            .contains("href=\"https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html\""));
        assert!(doc
            .cleaned_html
            .contains("href=\"https://docs.rs/tokio/latest/index.html\""));
        assert!(doc.cleaned_html.contains("href=\"#methods\""));
        assert!(doc
            .cleaned_html
            .contains("src=\"https://docs.rs/tokio/latest/tokio/logo.png\""));
    }

    #[test]
    fn test_dom_parser_custom_policy_overrides() {
        let html = r##"
            <body>
                <nav class="navigation">
                    <a href="/home">Home</a>
                </nav>
                <div class="newsletter-box">
                    <p>Subscribe to our newsletter</p>
                </div>
                <div class="custom-promo keep-this-banner">
                    <p>Important System Announcement</p>
                </div>
                <main>
                    <p>Content text.</p>
                </main>
            </body>
        "##;

        let policy = BoilerplatePolicy::default()
            .with_strip_nav(false)
            .with_blocked_class("newsletter-box")
            .with_allowed_class("keep-this-banner");

        let parser = DomParser::new(policy);
        let doc = parser.parse(html, None).unwrap();

        // Nav is preserved because strip_nav = false
        assert!(doc.cleaned_html.contains("<nav"));
        assert!(doc.cleaned_html.contains("/home"));

        // Newsletter is stripped because of custom_blocked_classes
        assert!(!doc.cleaned_html.contains("Subscribe to our newsletter"));

        // Announcement is preserved because keep-this-banner is in custom_allowed_classes
        assert!(doc.cleaned_html.contains("Important System Announcement"));
    }

    #[test]
    fn test_dom_parser_malformed_html_resilience() {
        // Missing doctype, unclosed tags, messy structure
        let malformed = "<div><h1>Broken Header<p>Paragraph without closing tag<div><span>Nested unclosed<span></div>";
        let parser = DomParser::default();
        let doc = parser.parse(malformed, None).unwrap();

        assert!(!doc.is_empty());
        assert!(doc.cleaned_text.contains("Broken Header"));
        assert!(doc.cleaned_text.contains("Paragraph without closing tag"));
        assert!(doc.cleaned_text.contains("Nested unclosed"));
    }

    #[test]
    fn test_dom_parser_metadata_opengraph_fallback() {
        let html = r##"
            <html>
            <head>
                <meta property="og:title" content="OpenGraph Article Title">
                <meta property="og:description" content="OpenGraph Article Summary">
            </head>
            <body>
                <article>
                    <p>Body text.</p>
                </article>
            </body>
            </html>
        "##;

        let parser = DomParser::default();
        let doc = parser.parse(html, None).unwrap();

        assert_eq!(
            doc.metadata.title.as_deref(),
            Some("OpenGraph Article Title")
        );
        assert_eq!(
            doc.metadata.description.as_deref(),
            Some("OpenGraph Article Summary")
        );
    }

    #[test]
    fn test_article_extractor_standard_blog_post() {
        let html = r##"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Async Systems in Rust</title>
                <meta name="author" content="Dr. Alan Kay">
                <meta name="description" content="An architectural discussion of actor models and async Rust.">
            </head>
            <body>
                <header class="site-header">
                    <nav><a href="/">Home</a><a href="/docs">Docs</a></nav>
                </header>
                <div class="sidebar widget">
                    <h4>Top Stories</h4>
                    <a href="/s1">Story 1</a>
                    <a href="/s2">Story 2</a>
                </div>
                <main class="page-container">
                    <article class="post-entry" id="post-88">
                        <h1>Async Systems in Rust</h1>
                        <p class="byline">Published by <a rel="author" href="/author/alankay">Alan Kay</a> on <time datetime="2026-11-17T10:00:00Z">November 17, 2026</time></p>
                        <p>Modern software architectures increasingly rely on asynchronous event loops and non-blocking I/O to achieve massive concurrency on modest hardware.</p>
                        <p>In this post, we explore how Tokio coordinates task work-stealing, how pinned futures eliminate heap allocations, and how backpressure preserves stability under high load.</p>
                        <p>When engineering web crawlers or data conversion engines, unbounded memory allocation is the primary vulnerability that must be defended against.</p>
                    </article>
                </main>
                <footer class="site-footer">
                    <p>Copyright 2026</p>
                </footer>
            </body>
            </html>
        "##;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(!article.is_fallback);
        assert_eq!(article.candidate_tag, "article");
        assert_eq!(article.candidate_id.as_deref(), Some("post-88"));
        assert_eq!(article.title, "Async Systems in Rust");
        assert_eq!(article.byline.as_deref(), Some("Alan Kay"));
        assert_eq!(
            article.published_time.as_deref(),
            Some("2026-11-17T10:00:00Z")
        );
        assert!(article.word_count >= 50);
        assert!(article.reading_time_minutes >= 1);
        assert!(article.readability_score >= 30.0);
        assert!(article
            .content_text
            .contains("Modern software architectures"));
        assert!(article.content_text.contains("work-stealing"));
        assert!(!article.content_text.contains("Top Stories"));
    }

    #[test]
    fn test_article_extractor_technical_post_with_tables_and_code() {
        let html = r##"
            <html>
            <body>
                <article class="markdown-body">
                    <h1>Memory Bounds in Low-Level Rust</h1>
                    <p>Engineers must carefully evaluate decompression buffers, streaming limits, and allocator configurations when handling untrusted data streams.</p>
                    <pre><code class="language-rust">pub fn verify_budget(allocated: usize, ceiling: usize) -> bool {
    allocated <= ceiling
}</code></pre>
                    <p>The table below summarizes the maximum permissible memory footprints across various monorepo worker profiles:</p>
                    <table>
                        <thead>
                            <tr><th>Worker Type</th><th>RAM Ceiling</th><th>Storage Ceiling</th></tr>
                        </thead>
                        <tbody>
                            <tr><td>API Gateway</td><td>2 GB</td><td>5 GB</td></tr>
                            <tr><td>Browser Worker</td><td>4 GB</td><td>15 GB</td></tr>
                        </tbody>
                    </table>
                    <p>Strict observance of these thresholds guarantees predictable execution on standard 16 GB hardware without out-of-memory kernel intervention.</p>
                </article>
            </body>
            </html>
        "##;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(!article.is_fallback);
        assert_eq!(article.candidate_tag, "article");
        assert!(article
            .content_html
            .contains("<pre><code class=\"language-rust\">"));
        assert!(article.content_html.contains("<table>"));
        assert!(article.content_text.contains("pub fn verify_budget"));
        assert!(article.content_text.contains("Browser Worker"));
    }

    #[test]
    fn test_article_extractor_penalizes_and_rejects_link_heavy_widgets() {
        let html = r##"
            <html>
            <body>
                <div class="navigation-hub">
                    <a href="/cat1">Category 1</a>
                    <a href="/cat2">Category 2</a>
                    <a href="/cat3">Category 3</a>
                    <a href="/cat4">Category 4</a>
                    <a href="/cat5">Category 5</a>
                    <a href="/cat6">Category 6</a>
                </div>
            </body>
            </html>
        "##;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        // High link density elements should be rejected or trigger fallback
        assert!(article.is_fallback || article.readability_score < 10.0);
    }

    #[test]
    fn test_article_extractor_byline_and_time_extraction() {
        let html = r##"
            <article class="entry-content">
                <h2>Security Architecture Review</h2>
                <div class="post-meta">
                    <span class="author-name">Jane Doe, Principal Engineer</span>
                    <time datetime="2026-10-31">Halloween 2026</time>
                </div>
                <p>This comprehensive security analysis evaluates SSRF vectors, IPv4-mapped IPv6 encapsulation, and DNS rebinding mitigations across microservices.</p>
                <p>We enforce strict boundary checks at every layer, from socket connection establishment to TLS certificate verification and payload streaming.</p>
                <p>All network calls must resolve through an authenticated and verified pin-down mechanism to eliminate time-of-check to time-of-use exploits.</p>
            </article>
        "##;

        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(!article.is_fallback);
        assert_eq!(article.title, "Security Architecture Review");
        assert_eq!(
            article.byline.as_deref(),
            Some("Jane Doe, Principal Engineer")
        );
        assert_eq!(article.published_time.as_deref(), Some("2026-10-31"));
    }

    #[test]
    fn test_article_extractor_custom_thresholds_and_classes() {
        let html = r##"
            <div class="custom-blog-container">
                <h2>Custom Knowledge Node</h2>
                <p>This is a custom formatted knowledge node that uses proprietary class conventions rather than standard article tags.</p>
                <p>It contains multiple sentences of informational prose, designed to test custom class weight overrides and threshold adjustments.</p>
            </div>
        "##;

        let custom_options = ReadabilityOptions::new()
            .with_min_word_count(15)
            .with_min_score_threshold(10.0)
            .add_positive_class("custom-blog-container");

        let extractor = ArticleExtractor::new().with_options(custom_options);
        let article = extractor.extract(html, None).unwrap();

        assert!(!article.is_fallback);
        assert_eq!(article.candidate_tag, "div");
        assert_eq!(
            article.candidate_class.as_deref(),
            Some("custom-blog-container")
        );
        assert!(article.readability_score >= 10.0);
    }

    #[test]
    fn test_article_extractor_fallback_on_plain_document() {
        let html = "<html><head><title>Unformatted</title></head><body>Just a raw plain sentence without tags.</body></html>";
        let extractor = ArticleExtractor::new();
        let article = extractor.extract(html, None).unwrap();

        assert!(article.is_fallback);
        assert_eq!(article.title, "Unformatted");
        assert!(article.content_text.contains("Just a raw plain sentence"));
    }

    #[test]
    fn test_doc_extractor_mdbook_documentation_page() {
        let html = r##"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Writing Tests - The Rust Book</title>
                <meta name="generator" content="mdBook 0.4.36">
            </head>
            <body>
                <nav id="sidebar" class="sidebar">
                    <ol class="chapter">
                        <li><a href="ch01.html">Chapter 1</a></li>
                        <li><a href="ch11.html" class="active">Chapter 11: Writing Tests</a></li>
                    </ol>
                </nav>
                <div id="content" class="content">
                    <main id="content">
                        <h1 id="writing-tests">How to Write Tests</h1>
                        <p>Tests are Rust functions that verify that the non-test code is functioning in the expected manner.</p>
                        <h2 id="anatomy-of-a-test">The Anatomy of a Test Function</h2>
                        <p>At its simplest, a test in Rust is a function that's annotated with the test attribute.</p>
                        <pre><code class="language-rust">#[test]
fn it_works() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}</code></pre>
                        <h2 id="checking-results">Checking Results with the assert_eq! Macro</h2>
                        <p>The assert_eq! and assert_ne! macros compare two arguments for equality or inequality respectively.</p>
                    </main>
                </div>
                <nav class="nav-wrapper">
                    <a rel="prev" href="ch10.html" class="nav-chapters previous" title="Previous chapter: Generics">Previous</a>
                    <a rel="next" href="ch12.html" class="nav-chapters next" title="Next chapter: An I/O Project">Next</a>
                </nav>
            </body>
            </html>
        "##;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::MdBook);
        assert_eq!(doc.title, "How to Write Tests");
        assert_eq!(doc.table_of_contents.len(), 1);
        assert_eq!(doc.table_of_contents[0].title, "How to Write Tests");
        assert_eq!(doc.table_of_contents[0].children.len(), 2);
        assert_eq!(
            doc.table_of_contents[0].children[0].title,
            "The Anatomy of a Test Function"
        );
        assert_eq!(
            doc.table_of_contents[0].children[1].title,
            "Checking Results with the assert_eq! Macro"
        );

        assert!(doc.pagination.prev.is_some());
        assert_eq!(
            doc.pagination.prev.as_ref().unwrap().title,
            "Previous chapter: Generics"
        );
        assert!(doc.pagination.next.is_some());

        assert!(doc.content_text.contains("assert_eq!(result, 4)"));
        assert!(doc.content_html.contains("<code class=\"language-rust\">"));
    }

    #[test]
    fn test_doc_extractor_sphinx_with_admonitions_and_sections() {
        let html = r##"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Sphinx Configuration Guide</title>
                <meta name="generator" content="Sphinx 7.2.6">
            </head>
            <body>
                <div class="sphinxsidebar">
                    <div class="sphinxsidebarwrapper">
                        <h3>Table of Contents</h3>
                    </div>
                </div>
                <div class="document">
                    <div class="documentwrapper">
                        <div class="body" role="main">
                            <h1 id="configuration-options">Configuration Options</h1>
                            <p>This chapter covers all build-time configuration directives available in Sphinx.</p>
                            <div class="admonition warning">
                                <p class="admonition-title">Warning</p>
                                <p>Deprecations will raise errors when strict mode is active.</p>
                            </div>
                            <h2 id="html-theme">HTML Theme Settings</h2>
                            <p>Choose an appropriate theme for mobile responsiveness.</p>
                            <div class="admonition note">
                                <p class="admonition-title">Note</p>
                                <p>Alabaster is the default theme provided with Sphinx distributions.</p>
                            </div>
                        </div>
                    </div>
                </div>
            </body>
            </html>
        "##;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::Sphinx);
        assert_eq!(doc.title, "Configuration Options");
        assert_eq!(doc.table_of_contents.len(), 1);
        assert_eq!(doc.table_of_contents[0].children.len(), 1);

        assert_eq!(doc.admonitions.len(), 2);
        assert_eq!(doc.admonitions[0].kind, AdmonitionType::Warning);
        assert!(doc.admonitions[0]
            .content
            .contains("Deprecations will raise errors"));
        assert_eq!(doc.admonitions[1].kind, AdmonitionType::Note);
        assert!(doc.admonitions[1]
            .content
            .contains("Alabaster is the default theme"));
    }

    #[test]
    fn test_doc_extractor_rustdoc_reference_layout() {
        let html = r##"
            <!DOCTYPE html>
            <html class="rustdoc">
            <head>
                <title>tokio::sync::mpsc - Rust</title>
                <meta name="generator" content="rustdoc">
            </head>
            <body>
                <nav class="sidebar"><div class="sidebar-crate"><h2><a href="../../index.html">tokio</a></h2></div></nav>
                <main>
                    <div class="main-heading"><h1>Module tokio::sync::mpsc</h1></div>
                    <section id="main-content" class="content">
                        <h2 id="overview">Overview</h2>
                        <p>A multi-producer, single-consumer channel for message passing across asynchronous tasks.</p>
                        <h2 id="macros">Macros</h2>
                        <p>No macros defined in this module.</p>
                    </section>
                </main>
            </body>
            </html>
        "##;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::Rustdoc);
        assert!(doc.title.contains("Overview") || doc.title.contains("Module tokio"));
        assert!(doc.content_text.contains("multi-producer, single-consumer"));
    }

    #[test]
    fn test_doc_extractor_mkdocs_material_pagination_and_breadcrumbs() {
        let html = r##"
            <!DOCTYPE html>
            <html>
            <head>
                <title>Getting Started - MkDocs</title>
                <meta name="generator" content="mkdocs-1.5.3, mkdocs-material-9.5.3">
            </head>
            <body>
                <nav class="md-header__breadcrumb" aria-label="breadcrumbs">
                    <ol class="breadcrumb">
                        <li><a href="/">Home</a></li>
                        <li><a href="/user-guide/">User Guide</a></li>
                        <li>Getting Started</li>
                    </ol>
                </nav>
                <article class="md-content__inner md-typeset">
                    <h1 id="getting-started">Getting Started with MkDocs</h1>
                    <p>MkDocs is a fast, simple and downright gorgeous static site generator geared towards building project documentation.</p>
                    <div class="admonition tip">
                        <p class="admonition-title">Tip</p>
                        <p>Run mkdocs serve to preview documentation locally.</p>
                    </div>
                </article>
                <footer class="md-footer">
                    <nav class="md-footer__inner">
                        <a href="/index.html" class="md-footer__link md-footer__link--prev" rel="prev">
                            <div class="md-footer__title">Previous: Introduction</div>
                        </a>
                        <a href="/deploy.html" class="md-footer__link md-footer__link--next" rel="next">
                            <div class="md-footer__title">Next: Deployment</div>
                        </a>
                    </nav>
                </footer>
            </body>
            </html>
        "##;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::MkDocs);
        assert_eq!(doc.title, "Getting Started with MkDocs");
        assert_eq!(doc.breadcrumbs.len(), 3);
        assert_eq!(doc.breadcrumbs[1].label, "User Guide");

        assert!(doc.pagination.prev.is_some());
        assert!(doc.pagination.next.is_some());
        assert_eq!(doc.admonitions.len(), 1);
        assert_eq!(doc.admonitions[0].kind, AdmonitionType::Tip);
    }

    #[test]
    fn test_doc_extractor_generic_fallback_when_unrecognized() {
        let html = r##"
            <html>
            <head><title>Uncategorized Manual</title></head>
            <body>
                <main>
                    <h1>Custom Manual</h1>
                    <p>This is a custom internal system manual with no specific framework generator tags.</p>
                    <h2 id="sec-1">Section 1</h2>
                    <p>Operational instructions for database backups.</p>
                </main>
            </body>
            </html>
        "##;

        let extractor = DocExtractor::new();
        let doc = extractor.extract(html, None).unwrap();

        assert_eq!(doc.framework, DocFramework::Generic);
        assert_eq!(doc.title, "Custom Manual");
        assert_eq!(doc.table_of_contents.len(), 1);
        assert_eq!(doc.table_of_contents[0].children.len(), 1);
        assert!(doc.content_text.contains("Operational instructions"));
    }

    #[test]
    fn test_elements_table_complex_matrix_and_markdown() {
        let html = r##"
            <table>
                <caption>Quarterly Cloud Architecture Costs</caption>
                <thead>
                    <tr>
                        <th align="left">Service</th>
                        <th align="center">Region</th>
                        <th style="text-align: right">Cost (USD)</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>Compute Instances</td>
                        <td>us-east-1</td>
                        <td>$1,250.00</td>
                    </tr>
                    <tr>
                        <td>Block Storage</td>
                        <td>eu-central-1</td>
                        <td>$340.50</td>
                    </tr>
                </tbody>
            </table>
        "##;

        let extractor = ElementsExtractor::default();
        let elements = extractor.extract(html, None).unwrap();

        assert_eq!(elements.tables.len(), 1);
        let table = &elements.tables[0];
        assert_eq!(
            table.caption.as_deref(),
            Some("Quarterly Cloud Architecture Costs")
        );
        assert_eq!(table.headers, vec!["Service", "Region", "Cost (USD)"]);
        assert_eq!(table.num_rows, 3);
        assert_eq!(table.num_cols, 3);
        assert_eq!(
            table.column_alignments,
            vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right
            ]
        );

        let md = table.to_markdown();
        assert!(md.contains("**Quarterly Cloud Architecture Costs**"));
        assert!(md.contains("| Service | Region | Cost (USD) |"));
        assert!(md.contains("| :--- | :---: | ---: |"));
        assert!(md.contains("| Compute Instances | us-east-1 | $1,250.00 |"));
    }

    #[test]
    fn test_elements_code_blocks_and_syntax_detection() {
        let html = r##"
            <div class="content">
                <pre><code class="language-rust">
pub async fn fetch_manifest(id: &str) -> Result<Manifest, Error> {
    let url = format!("https://api.example.com/manifest/{id}");
    reqwest::get(&url).await?.json().await
}
                </code></pre>

                <pre><code>
SELECT u.id, u.email, COUNT(j.id) as total_jobs
FROM users u
JOIN jobs j ON j.user_id = u.id
WHERE u.status = 'active'
GROUP BY u.id, u.email;
                </code></pre>
            </div>
        "##;

        let extractor = ElementsExtractor::default();
        let elements = extractor.extract(html, None).unwrap();

        assert_eq!(elements.code_blocks.len(), 2);
        let rust_block = &elements.code_blocks[0];
        assert_eq!(rust_block.language.as_deref(), Some("rust"));
        assert!(rust_block.code.contains("pub async fn fetch_manifest"));

        let sql_block = &elements.code_blocks[1];
        assert_eq!(sql_block.language.as_deref(), Some("sql"));
        assert!(sql_block.code.contains("SELECT u.id, u.email"));
    }

    #[test]
    fn test_elements_code_line_number_gutters_stripped() {
        let html = r##"
            <pre><code>1  // Entry point
2  fn main() {
3      println!("Hello World");
4  }</code></pre>
        "##;

        let extractor = ElementsExtractor::default();
        let elements = extractor.extract(html, None).unwrap();

        assert_eq!(elements.code_blocks.len(), 1);
        let block = &elements.code_blocks[0];
        assert!(block.has_line_numbers_stripped);
        assert_eq!(
            block.code,
            "// Entry point\nfn main() {\n    println!(\"Hello World\");\n}"
        );
        assert_eq!(block.language.as_deref(), Some("rust"));
    }

    #[test]
    fn test_elements_link_extraction_and_classification() {
        let base_url = ValidatedUrl::parse_default("https://antigravity.dev/docs/intro").unwrap();
        let html = r##"
            <nav>
                <a href="/docs/architecture" title="System Architecture">Architecture</a>
                <a href="https://github.com/dayashimoga/web-repo-to-ai" rel="noopener noreferrer">GitHub Repo</a>
                <a href="#quickstart">Quickstart</a>
                <a href="/data/dataset.parquet">Dataset Download</a>
                <a href="/archive/release-v1.zip">Release Archive</a>
                <a href="mailto:security@antigravity.dev">Security Contact</a>
                <a href="javascript:void(0)">Noop Link</a>
            </nav>
        "##;

        let extractor = ElementsExtractor::default();
        let elements = extractor.extract(html, Some(&base_url)).unwrap();

        assert_eq!(elements.links.len(), 6); // javascript: link is stripped

        let internal = &elements.links[0];
        assert_eq!(internal.url, "https://antigravity.dev/docs/architecture");
        assert_eq!(internal.link_type, LinkType::Internal);
        assert_eq!(internal.title.as_deref(), Some("System Architecture"));

        let external = &elements.links[1];
        assert_eq!(
            external.url,
            "https://github.com/dayashimoga/web-repo-to-ai"
        );
        assert_eq!(external.link_type, LinkType::External);
        assert_eq!(external.rel, vec!["noopener", "noreferrer"]);

        let anchor = &elements.links[2];
        assert_eq!(anchor.url, "#quickstart");
        assert_eq!(anchor.link_type, LinkType::Anchor);

        let asset1 = &elements.links[3];
        assert_eq!(asset1.url, "https://antigravity.dev/data/dataset.parquet");
        assert_eq!(asset1.link_type, LinkType::Asset);

        let asset2 = &elements.links[4];
        assert_eq!(asset2.url, "https://antigravity.dev/archive/release-v1.zip");
        assert_eq!(asset2.link_type, LinkType::Asset);

        let special = &elements.links[5];
        assert_eq!(special.url, "mailto:security@antigravity.dev");
        assert_eq!(special.link_type, LinkType::Special);
    }

    #[test]
    fn test_elements_extractor_integrated_full_pipeline() {
        let base = ValidatedUrl::parse_default("https://example.org/api-ref").unwrap();
        let html = r##"
            <!DOCTYPE html>
            <html>
            <head><title>API Reference & Benchmarks</title></head>
            <body>
                <h1>API Specification</h1>
                <p>Refer to our <a href="/getting-started">Getting Started</a> guide.</p>
                
                <pre><code class="language-python">
import httpx

async def call_api(token: str):
    headers = {"Authorization": f"Bearer {token}"}
    async with httpx.AsyncClient() as client:
        return await client.get("https://api.example.org/v1/jobs", headers=headers)
                </code></pre>

                <table>
                    <thead>
                        <tr>
                            <th>Endpoint</th>
                            <th>Method</th>
                            <th>Auth</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>/v1/convert</td>
                            <td>POST</td>
                            <td>Required</td>
                        </tr>
                    </tbody>
                </table>

                <p>Download client SDK: <a href="/downloads/python-sdk.tar.gz">SDK Archive</a></p>
            </body>
            </html>
        "##;

        let options = ElementsOptions::default().with_inline_code(true);
        let extractor = ElementsExtractor::new(options);
        let elements = extractor.extract(html, Some(&base)).unwrap();

        assert_eq!(elements.tables.len(), 1);
        assert_eq!(elements.tables[0].num_rows, 2);
        assert_eq!(elements.tables[0].num_cols, 3);

        assert_eq!(elements.code_blocks.len(), 1);
        assert_eq!(elements.code_blocks[0].language.as_deref(), Some("python"));

        assert_eq!(elements.links.len(), 2);
        assert_eq!(elements.links[0].link_type, LinkType::Internal);
        assert_eq!(elements.links[1].link_type, LinkType::Asset);
    }

    #[test]
    fn test_browser_worker_ipc_protocol_and_error_mapping() {
        use web_repo_to_ai_core::error::CoreError;

        let req = BrowserRenderRequest::new("https://spa.example.com/dashboard")
            .with_wait_until(WaitUntil::NetworkIdle)
            .with_timeout_ms(25000)
            .with_wait_for_selector("#dashboard-root")
            .with_viewport(1440, 900)
            .with_user_agent("Mozilla/5.0 web-repo-to-ai")
            .with_header("X-Client-Id", "worker-1")
            .with_javascript(true)
            .with_block_subresource_ips(true);

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("spa.example.com"));
        assert!(json.contains("networkidle"));
        assert!(json.contains("25000"));
        assert!(json.contains("dashboard-root"));

        let deserialized: BrowserRenderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, req);

        // Verify error mapping hierarchy
        let browser_err = BrowserWorkerError::BlockedUrl("127.0.0.1 blocked by SSRF".to_string());
        let web_err: WebExtractorError = browser_err.into();
        assert!(matches!(web_err, WebExtractorError::BrowserWorker(_)));

        let core_err: CoreError = web_err.into();
        assert!(matches!(core_err, CoreError::Extraction(_)));
        assert!(core_err.to_string().contains("SSRF"));
    }
}
