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

pub mod error;
pub mod http;
pub mod url;

pub use error::WebExtractorError;
pub use http::{
    BodyByteStream, FetchedResponse, HttpTransport, MockHttpTransport, MockResponse, RedirectHop,
    RedirectTracker, ReqwestTransport, TransportResponse, WebFetcher, DEFAULT_MAX_BODY_BYTES,
    DEFAULT_MAX_REDIRECTS, DEFAULT_REQUEST_TIMEOUT, DEFAULT_USER_AGENT,
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
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use std::sync::Arc;
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
}
