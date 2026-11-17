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

pub mod compression;
pub mod error;
pub mod http;
pub mod url;

pub use compression::{BodyLimits, BoundedDecompressor, ContentEncoding};
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
        MockDnsResolver, SecureDnsResolver, UrlSecurityError, UrlSecurityPolicy,
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
}
