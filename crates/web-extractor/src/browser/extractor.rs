//! # Dynamic Web Extractor & Post-Render Hydration Engine
//!
//! Orchestrates the complete web extraction pipeline:
//! 1. Fast streaming static HTTP acquisition
//! 2. Content density & SPA shell heuristic analysis
//! 3. Dynamic browser worker invocation for JS-rendered pages
//! 4. Post-render DOM cleaning, article scoring, doc parsing, and element extraction

use std::sync::Arc;
use std::time::Instant;

use serde::{Deserialize, Serialize};
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::client::BrowserWorkerClient;
use super::model::{
    BlockedSubresource, BrowserRedirect, BrowserRenderRequest, BrowserRenderResponse,
    ConsoleMessage, PageError, WaitUntil,
};
use crate::article::{ArticleExtractor, ExtractedArticle, ReadabilityOptions};
use crate::docs::{DocExtractor, ExtractedDocumentation};
use crate::dom::{CleanedDocument, DomParser};
use crate::elements::{ElementsExtractor, ElementsOptions, ExtractedElements};
use crate::error::WebExtractorError;
use crate::http::{FetchedResponse, WebFetcher};
use crate::provenance::{ProvenanceBuilder, ProvenanceWarning, SourceAttribution, WarningSeverity};
use crate::quality::{ExtractionQualityReport, QualityScorer};
use crate::url::WebTarget;

/// Default minimum character threshold for static text before evaluating SPA fallback.
pub const DEFAULT_MIN_STATIC_TEXT_LENGTH: usize = 150;

/// Default minimum content density ratio (text length / HTML bytes).
pub const DEFAULT_MIN_CONTENT_DENSITY: f64 = 0.05;

/// Strategy dictating when dynamic browser rendering is invoked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExtractionStrategy {
    /// Attempts fast static extraction first; falls back to browser rendering if sparse or SPA shell.
    #[default]
    Auto,
    /// Executes only static acquisition; never invokes browser worker.
    StaticOnly,
    /// Skips static acquisition; immediately dispatches to browser worker for dynamic rendering.
    ForceDynamic,
}

/// The terminal execution method used to extract the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtractionMethod {
    /// Extracted directly via static HTTP acquisition.
    Static,
    /// Extracted via Playwright / Chromium dynamic browser hydration.
    DynamicBrowser,
}

/// Configurable options controlling dynamic extraction and post-render hydration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicExtractorOptions {
    /// Acquisition and fallback strategy.
    pub strategy: ExtractionStrategy,
    /// Navigation wait condition for browser rendering.
    pub wait_until: WaitUntil,
    /// Navigation timeout in milliseconds for browser rendering.
    pub timeout_ms: u64,
    /// Optional CSS selector to wait for during dynamic rendering.
    pub wait_for_selector: Option<String>,
    /// Whether JavaScript execution is enabled in browser worker.
    pub javascript_enabled: bool,
    /// Whether SSRF subresource request blocking is enforced.
    pub block_subresource_ips: bool,
    /// Minimum text character count threshold before triggering dynamic fallback.
    pub min_static_text_length: usize,
    /// Minimum content density ratio before triggering dynamic fallback.
    pub min_content_density: f64,
}

impl Default for DynamicExtractorOptions {
    fn default() -> Self {
        Self {
            strategy: ExtractionStrategy::Auto,
            wait_until: WaitUntil::NetworkIdle,
            timeout_ms: 30000,
            wait_for_selector: None,
            javascript_enabled: true,
            block_subresource_ips: true,
            min_static_text_length: DEFAULT_MIN_STATIC_TEXT_LENGTH,
            min_content_density: DEFAULT_MIN_CONTENT_DENSITY,
        }
    }
}

/// Comprehensive acquisition and rendering provenance for the extracted page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DynamicProvenance {
    /// Whether the document was extracted statically or via dynamic browser rendering.
    pub method: ExtractionMethod,
    /// Terminal HTTP response status code.
    pub http_status: u16,
    /// Total elapsed time in milliseconds for acquisition and hydration.
    pub load_time_ms: u64,
    /// Terminal canonical URL after all redirects.
    pub final_url: String,
    /// Hop-by-hop redirects encountered during acquisition.
    pub redirects: Vec<BrowserRedirect>,
    /// Console messages captured from the page.
    pub console_messages: Vec<ConsoleMessage>,
    /// Unhandled JavaScript exceptions captured from the page.
    pub page_errors: Vec<PageError>,
    /// Subresources blocked by SSRF/sandbox limits.
    pub blocked_subresources: Vec<BlockedSubresource>,
    /// Diagnostic warnings generated during acquisition or fallback degradation.
    pub warnings: Vec<String>,
}

/// Unified extracted webpage representation combining sanitized DOM, articles, docs, and structured elements.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtractedWebpage {
    /// Canonical URL of the extracted webpage.
    pub url: String,
    /// Page title extracted from `<title>`, OpenGraph, or browser metadata.
    pub title: String,
    /// Sanitized DOM document with boilerplate stripped.
    pub cleaned_doc: CleanedDocument,
    /// Extracted article content if readability heuristics succeeded.
    pub article: Option<ExtractedArticle>,
    /// Extracted documentation hierarchy if doc framework was detected.
    pub documentation: Option<ExtractedDocumentation>,
    /// Structured content elements: tables, code blocks, hyperlinks.
    pub elements: ExtractedElements,
    /// Comprehensive extraction provenance and runtime diagnostics.
    pub provenance: DynamicProvenance,
    /// Complete source attribution, cryptographic hashes, HTTP provenance, and citations.
    pub attribution: SourceAttribution,
    /// Multidimensional extraction completeness and quality assessment.
    pub quality: ExtractionQualityReport,
}

/// Heuristic analyzer detecting whether a static HTML document is an SPA shell requiring browser rendering.
#[derive(Debug, Clone)]
pub struct SpaDetector {
    /// Minimum text character count threshold.
    pub min_text_length: usize,
    /// Minimum content density ratio.
    pub min_content_density: f64,
}

impl Default for SpaDetector {
    fn default() -> Self {
        Self {
            min_text_length: DEFAULT_MIN_STATIC_TEXT_LENGTH,
            min_content_density: DEFAULT_MIN_CONTENT_DENSITY,
        }
    }
}

impl SpaDetector {
    /// Evaluates whether a cleaned document and raw HTML suggest an SPA shell requiring browser rendering.
    #[must_use]
    pub fn is_fallback_needed(&self, cleaned_doc: &CleanedDocument, raw_html: &str) -> bool {
        // 1. If text is completely empty, definitely fallback if raw HTML had some markup
        if cleaned_doc.is_empty() {
            return !raw_html.trim().is_empty();
        }

        let text_len = cleaned_doc.text_length();
        let html_lower = raw_html.to_ascii_lowercase();

        // 2. Strong explicit SPA hydration roots
        let has_empty_spa_root = html_lower.contains("id=\"root\"")
            || html_lower.contains("id=\"app\"")
            || html_lower.contains("id=\"__next\"")
            || html_lower.contains("id=\"svelte\"")
            || html_lower.contains("<app-root");

        // 3. Noscript warning indicating JS is mandatory
        let has_noscript_warning = html_lower.contains("you need to enable javascript")
            || html_lower.contains("javascript is required")
            || html_lower.contains("please enable javascript")
            || html_lower.contains("enable javascript to run");

        if has_noscript_warning && text_len < 300 {
            return true;
        }

        if has_empty_spa_root && text_len < self.min_text_length {
            return true;
        }

        // 4. Low density check: substantial HTML markup (>2KB) but tiny extracted text
        let raw_len = raw_html.len();
        if raw_len > 2000 {
            let density = (text_len as f64) / (raw_len as f64);
            if density < self.min_content_density && text_len < self.min_text_length {
                return true;
            }
        }

        false
    }
}

/// Dynamic web extraction and hydration orchestrator.
#[derive(Debug, Clone)]
pub struct DynamicWebExtractor {
    policy: UrlSecurityPolicy,
    options: DynamicExtractorOptions,
    spa_detector: SpaDetector,
    dom_parser: DomParser,
    article_extractor: ArticleExtractor,
    doc_extractor: DocExtractor,
    elements_extractor: ElementsExtractor,
    fetcher: Option<Arc<WebFetcher>>,
    browser_client: Option<BrowserWorkerClient>,
}

impl DynamicWebExtractor {
    /// Constructs an extractor with the given URL policy and default options.
    #[must_use]
    pub fn new(policy: UrlSecurityPolicy) -> Self {
        let spa_detector = SpaDetector::default();
        let dom_parser = DomParser::default().with_url_security_policy(policy.clone());
        let article_extractor = ArticleExtractor::default();
        let doc_extractor = DocExtractor::default().with_policy(policy.clone());
        let elements_extractor =
            ElementsExtractor::default().with_url_security_policy(policy.clone());

        Self {
            policy,
            options: DynamicExtractorOptions::default(),
            spa_detector,
            dom_parser,
            article_extractor,
            doc_extractor,
            elements_extractor,
            fetcher: None,
            browser_client: None,
        }
    }

    /// Sets custom extraction options.
    #[must_use]
    pub fn with_options(mut self, options: DynamicExtractorOptions) -> Self {
        self.spa_detector.min_text_length = options.min_static_text_length;
        self.spa_detector.min_content_density = options.min_content_density;
        self.options = options;
        self
    }

    /// Configures the static `WebFetcher`.
    #[must_use]
    pub fn with_fetcher(mut self, fetcher: Arc<WebFetcher>) -> Self {
        self.fetcher = Some(fetcher);
        self
    }

    /// Configures the `BrowserWorkerClient`.
    #[must_use]
    pub fn with_browser_client(mut self, client: BrowserWorkerClient) -> Self {
        self.browser_client = Some(client);
        self
    }

    /// Sets custom `ReadabilityOptions` for article extraction.
    #[must_use]
    pub fn with_article_options(mut self, options: ReadabilityOptions) -> Self {
        self.article_extractor = ArticleExtractor::new().with_options(options);
        self
    }

    /// Sets custom `ElementsOptions` for structured elements extraction.
    #[must_use]
    pub fn with_elements_options(mut self, options: ElementsOptions) -> Self {
        self.elements_extractor =
            ElementsExtractor::new(options).with_url_security_policy(self.policy.clone());
        self
    }

    /// Executes the full extraction pipeline for a given target URL.
    pub async fn extract(&self, target_url: &str) -> Result<ExtractedWebpage, WebExtractorError> {
        let start_time = Instant::now();
        let validated_url = ValidatedUrl::parse(target_url, &self.policy)?;

        match self.options.strategy {
            ExtractionStrategy::ForceDynamic => {
                self.render_and_hydrate(&validated_url, start_time).await
            }
            ExtractionStrategy::StaticOnly => self.extract_static(&validated_url, start_time).await,
            ExtractionStrategy::Auto => {
                // 1. Attempt fast streaming static acquisition
                match self.extract_static(&validated_url, start_time).await {
                    Ok(mut static_page) => {
                        // 2. Evaluate if dynamic browser fallback is needed
                        let raw_html = static_page.cleaned_doc.cleaned_html.clone();
                        let fallback_needed = self
                            .spa_detector
                            .is_fallback_needed(&static_page.cleaned_doc, &raw_html);

                        if !fallback_needed {
                            return Ok(static_page);
                        }

                        // 3. Dynamic fallback is indicated (SPA / empty shell)
                        if self.browser_client.is_none() {
                            static_page.provenance.warnings.push(
                                "Dynamic fallback indicated but browser worker client is not configured."
                                    .to_string(),
                            );
                            static_page.attribution.warnings.push(ProvenanceWarning::new(
                                "BROWSER_CLIENT_UNCONFIGURED",
                                "Dynamic fallback indicated but browser worker client is not configured.",
                                WarningSeverity::Warning,
                            ));
                            return Ok(static_page);
                        }

                        // 4. Dispatch to browser worker for dynamic hydration
                        match self.render_and_hydrate(&validated_url, start_time).await {
                            Ok(hydrated_page) => Ok(hydrated_page),
                            Err(dyn_err) => {
                                // Graceful degradation: preserve static extraction result with diagnostic warning
                                static_page.provenance.warnings.push(format!(
                                    "Dynamic browser rendering failed: {dyn_err}; degraded to static extraction."
                                ));
                                static_page.attribution.warnings.push(ProvenanceWarning::new(
                                    "BROWSER_DEGRADED_TO_STATIC",
                                    format!("Dynamic browser rendering failed: {dyn_err}; degraded to static extraction."),
                                    WarningSeverity::Critical,
                                ));
                                Ok(static_page)
                            }
                        }
                    }
                    Err(static_err) => {
                        // If static fetch completely failed, attempt browser worker if configured
                        if self.browser_client.is_some() {
                            self.render_and_hydrate(&validated_url, start_time).await
                        } else {
                            Err(static_err)
                        }
                    }
                }
            }
        }
    }

    /// Executes static acquisition via `WebFetcher` and runs parsing pipelines.
    async fn extract_static(
        &self,
        target: &ValidatedUrl,
        start_time: Instant,
    ) -> Result<ExtractedWebpage, WebExtractorError> {
        let fetcher = self
            .fetcher
            .as_ref()
            .ok_or_else(|| WebExtractorError::ExtractionFailed {
                details: "WebFetcher is required for static acquisition but was not provided."
                    .to_string(),
            })?;

        let web_target = WebTarget::new(target.clone());
        let fetched: FetchedResponse = fetcher.fetch(&web_target).await?;

        let html = fetched.text_lossy().into_owned();
        let canonical_url = fetched.url.as_str().to_string();

        let cleaned_doc = self.dom_parser.parse(&html, Some(target))?;

        let article = self
            .article_extractor
            .extract_from_cleaned(&cleaned_doc, Some(target))
            .ok();
        let documentation = self.doc_extractor.extract(&html, Some(target)).ok();
        let elements = self.elements_extractor.extract(&html, Some(target))?;

        let title = cleaned_doc
            .metadata
            .title
            .clone()
            .unwrap_or_else(|| "Untitled Document".to_string());

        let redirects = fetched
            .redirect_history
            .iter()
            .map(|hop| BrowserRedirect {
                from: hop.from_url.as_str().to_string(),
                to: hop.to_url.as_str().to_string(),
                status: hop.status_code,
            })
            .collect();

        let provenance = DynamicProvenance {
            method: ExtractionMethod::Static,
            http_status: fetched.status,
            load_time_ms: start_time.elapsed().as_millis() as u64,
            final_url: canonical_url,
            redirects,
            console_messages: Vec::new(),
            page_errors: Vec::new(),
            blocked_subresources: Vec::new(),
            warnings: Vec::new(),
        };

        let attribution = ProvenanceBuilder::build_static(
            &fetched,
            &cleaned_doc,
            &html,
            start_time,
            &self.policy,
        );

        let quality = QualityScorer::new().assess_components(
            &cleaned_doc,
            article.as_ref(),
            documentation.as_ref(),
            &elements,
            &attribution,
            &provenance,
            &html,
        );

        Ok(ExtractedWebpage {
            url: target.as_str().to_string(),
            title,
            cleaned_doc,
            article,
            documentation,
            elements,
            provenance,
            attribution,
            quality,
        })
    }

    /// Dispatches a rendering request to `services/browser-worker` and hydrates the resulting DOM.
    async fn render_and_hydrate(
        &self,
        target: &ValidatedUrl,
        start_time: Instant,
    ) -> Result<ExtractedWebpage, WebExtractorError> {
        let client =
            self.browser_client
                .as_ref()
                .ok_or_else(|| {
                    WebExtractorError::ExtractionFailed {
                details:
                    "BrowserWorkerClient is required for dynamic rendering but was not provided."
                        .to_string(),
            }
                })?;

        let mut req = BrowserRenderRequest::new(target.as_str())
            .with_wait_until(self.options.wait_until)
            .with_timeout_ms(self.options.timeout_ms)
            .with_javascript(self.options.javascript_enabled)
            .with_block_subresource_ips(self.options.block_subresource_ips);

        if let Some(ref sel) = self.options.wait_for_selector {
            req = req.with_wait_for_selector(sel);
        }

        let resp: BrowserRenderResponse = client.render(&req).await?;

        // Hydrate and parse rendered DOM
        let final_url = if resp.url.is_empty() {
            target.as_str().to_string()
        } else {
            resp.url.clone()
        };

        let canonical_target =
            ValidatedUrl::parse(&final_url, &self.policy).unwrap_or_else(|_| target.clone());

        let cleaned_doc = self.dom_parser.parse(&resp.html, Some(&canonical_target))?;

        let article = self
            .article_extractor
            .extract_from_cleaned(&cleaned_doc, Some(&canonical_target))
            .ok();
        let documentation = self
            .doc_extractor
            .extract(&resp.html, Some(&canonical_target))
            .ok();
        let elements = self
            .elements_extractor
            .extract(&resp.html, Some(&canonical_target))?;

        let title = if !resp.title.is_empty() {
            resp.title.clone()
        } else if let Some(ref meta_title) = cleaned_doc.metadata.title {
            meta_title.clone()
        } else {
            "Untitled Document".to_string()
        };

        let provenance = DynamicProvenance {
            method: ExtractionMethod::DynamicBrowser,
            http_status: resp.status_code,
            load_time_ms: start_time.elapsed().as_millis() as u64,
            final_url: final_url.clone(),
            redirects: resp.redirects.clone(),
            console_messages: resp.console_messages.clone(),
            page_errors: resp.page_errors.clone(),
            blocked_subresources: resp.blocked_subresources.clone(),
            warnings: Vec::new(),
        };

        let attribution = ProvenanceBuilder::build_dynamic(
            target.as_str(),
            &resp,
            &cleaned_doc,
            start_time,
            &self.policy,
        );

        let quality = QualityScorer::new().assess_components(
            &cleaned_doc,
            article.as_ref(),
            documentation.as_ref(),
            &elements,
            &attribution,
            &provenance,
            &resp.html,
        );

        Ok(ExtractedWebpage {
            url: final_url,
            title,
            cleaned_doc,
            article,
            documentation,
            elements,
            provenance,
            attribution,
            quality,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::policy::BoilerplatePolicy;
    use crate::http::MockHttpTransport;
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use web_repo_to_ai_url_security::MockDnsResolver;
    use web_repo_to_ai_url_security::SecureDnsResolver;

    #[test]
    fn test_spa_detector_heuristics() {
        let detector = SpaDetector::default();
        let policy = BoilerplatePolicy::default();
        let parser = DomParser::new(policy).with_url_security_policy(UrlSecurityPolicy::default());

        // 1. Standard rich article - no fallback needed
        let rich_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Rust Concurrency</title></head>
                <body>
                    <article>
                        <h1>Fearless Concurrency in Rust</h1>
                        <p>Rust provides strong compile-time guarantees preventing data races and memory corruption across threads.</p>
                        <p>The type system enforces Send and Sync traits statically, ensuring shared data is synchronized safely.</p>
                        <p>By leveraging ownership and borrowing, developers write multi-threaded applications with confidence.</p>
                    </article>
                </body>
            </html>
        "#;
        let rich_doc = parser.parse(rich_html, None).unwrap();
        assert!(!detector.is_fallback_needed(&rich_doc, rich_html));

        // 2. React root with noscript warning - fallback needed
        let react_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>App</title></head>
                <body>
                    <div id="root"></div>
                    <noscript>You need to enable JavaScript to run this app.</noscript>
                    <script src="/static/js/bundle.js"></script>
                </body>
            </html>
        "#;
        let react_doc = parser.parse(react_html, None).unwrap();
        assert!(detector.is_fallback_needed(&react_doc, react_html));

        // 3. Vue app skeleton - fallback needed
        let vue_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Vue SPA</title></head>
                <body>
                    <div id="app"></div>
                    <script src="/main.js"></script>
                </body>
            </html>
        "#;
        let vue_doc = parser.parse(vue_html, None).unwrap();
        assert!(detector.is_fallback_needed(&vue_doc, vue_html));
    }

    #[tokio::test]
    async fn test_dynamic_extractor_auto_rich_article_static_path() {
        let policy = UrlSecurityPolicy::default();
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let article_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Understanding Tokio</title></head>
                <body>
                    <main>
                        <h1>Understanding Tokio Async Runtimes</h1>
                        <p>Tokio is an asynchronous runtime for the Rust programming language that delivers reliable, network applications.</p>
                        <p>It provides an event-driven, non-blocking I/O platform with a multi-threaded work-stealing scheduler.</p>
                    </main>
                </body>
            </html>
        "#;

        mock_http.set_html("https://example.com/tokio", article_html);

        let dns_resolver = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
        let fetcher = Arc::new(
            WebFetcher::new(policy.clone(), dns_resolver).with_transport(Arc::new(mock_http)),
        );

        let extractor = DynamicWebExtractor::new(policy).with_fetcher(fetcher);

        let result = extractor
            .extract("https://example.com/tokio")
            .await
            .unwrap();

        assert_eq!(result.provenance.method, ExtractionMethod::Static);
        assert_eq!(result.title, "Understanding Tokio");
        assert!(result
            .cleaned_doc
            .cleaned_text
            .contains("Tokio is an asynchronous runtime"));
        assert!(result.article.is_some());
        assert_eq!(
            result.attribution.canonical_url,
            "https://example.com/tokio"
        );
        assert_eq!(
            result.attribution.extraction_method,
            ExtractionMethod::Static
        );
        assert_eq!(result.attribution.content_hashes.raw_html_sha256.len(), 64);
        assert_eq!(result.attribution.content_hashes.raw_html_blake3.len(), 64);
        assert!(result.attribution.markdown_citation().contains("Source"));
        assert!(result.quality.overall_score > 0.0);
        assert!(result.quality.is_acceptable_for_cdm);
        assert_eq!(result.quality.metrics.heading_count, 1);
    }

    #[tokio::test]
    async fn test_dynamic_extractor_auto_spa_fallback_to_browser_worker() {
        let policy = UrlSecurityPolicy::default();
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        // Static fetch returns bare SPA skeleton
        let spa_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Loading...</title></head>
                <body>
                    <div id="root"></div>
                    <noscript>You need to enable JavaScript to run this app.</noscript>
                </body>
            </html>
        "#;

        mock_http.set_html("https://example.com/spa", spa_html);

        let dns_resolver = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
        let fetcher = Arc::new(
            WebFetcher::new(policy.clone(), dns_resolver).with_transport(Arc::new(mock_http)),
        );

        // Spawn mock browser worker HTTP server returning hydrated content
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let base_url = format!("http://{}", addr);

        tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                tokio::spawn(async move {
                    let mut buf = [0u8; 4096];
                    let n = socket.read(&mut buf).await.unwrap_or(0);
                    let req_str = String::from_utf8_lossy(&buf[..n]);

                    if req_str.starts_with("POST /render") {
                        let body = r#"{
                            "url":"https://example.com/spa",
                            "status_code":200,
                            "html":"<!DOCTYPE html><html><head><title>Hydrated SPA Page</title></head><body><main><h1>Client-Side Hydration Complete</h1><p>Dynamic interactive dashboard loaded with analytics.</p></main></body></html>",
                            "title":"Hydrated SPA Page",
                            "load_time_ms":150,
                            "redirects":[],
                            "console_messages":[],
                            "page_errors":[],
                            "blocked_subresources":[],
                            "timing":{"navigation_start":1,"dom_content_loaded":2,"load":3}
                        }"#;
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = socket.write_all(resp.as_bytes()).await;
                    }
                });
            }
        });

        let browser_client = BrowserWorkerClient::new(&base_url);
        let extractor = DynamicWebExtractor::new(policy)
            .with_fetcher(fetcher)
            .with_browser_client(browser_client);

        let result = extractor.extract("https://example.com/spa").await.unwrap();

        assert_eq!(result.provenance.method, ExtractionMethod::DynamicBrowser);
        assert_eq!(result.title, "Hydrated SPA Page");
        assert!(result
            .cleaned_doc
            .cleaned_text
            .contains("Client-Side Hydration Complete"));
        assert!(result.article.is_some());
        assert_eq!(result.attribution.canonical_url, "https://example.com/spa");
        assert_eq!(
            result.attribution.extraction_method,
            ExtractionMethod::DynamicBrowser
        );
        assert!(result.attribution.browser_diagnostics.is_some());
        assert_eq!(result.attribution.content_hashes.raw_html_blake3.len(), 64);
        assert!(result.quality.overall_score > 0.0);
        assert!(result.quality.is_acceptable_for_cdm);
    }

    #[tokio::test]
    async fn test_dynamic_extractor_degrades_gracefully_when_browser_fails() {
        let policy = UrlSecurityPolicy::default();
        let mock_dns = MockDnsResolver::new();
        let mock_http = MockHttpTransport::new();

        let host_ip = Ipv4Addr::new(93, 184, 216, 34);
        mock_dns.set_host(
            "example.com",
            vec![SocketAddr::new(IpAddr::V4(host_ip), 443)],
        );

        let fallback_html = r#"
            <!DOCTYPE html>
            <html>
                <head><title>Sparse Page</title></head>
                <body>
                    <div id="root">
                        <p>Minimal fallback text available statically.</p>
                    </div>
                </body>
            </html>
        "#;

        mock_http.set_html("https://example.com/sparse", fallback_html);

        let dns_resolver = SecureDnsResolver::with_resolver(policy.clone(), Arc::new(mock_dns));
        let fetcher = Arc::new(
            WebFetcher::new(policy.clone(), dns_resolver).with_transport(Arc::new(mock_http)),
        );

        // Point browser worker client to non-existent server (port 1) to simulate failure
        let browser_client = BrowserWorkerClient::new("http://127.0.0.1:1");
        let extractor = DynamicWebExtractor::new(policy)
            .with_fetcher(fetcher)
            .with_browser_client(browser_client);

        let result = extractor
            .extract("https://example.com/sparse")
            .await
            .unwrap();

        // Must degrade gracefully to static extraction and record diagnostic warning
        assert_eq!(result.provenance.method, ExtractionMethod::Static);
        assert!(result
            .cleaned_doc
            .cleaned_text
            .contains("Minimal fallback text available"));
        assert!(!result.provenance.warnings.is_empty());
        assert!(result.provenance.warnings[0].contains("Dynamic browser rendering failed"));
        assert_eq!(
            result.attribution.canonical_url,
            "https://example.com/sparse"
        );
        assert_eq!(
            result.attribution.extraction_method,
            ExtractionMethod::Static
        );
        assert!(result
            .attribution
            .warning_codes()
            .contains(&"BROWSER_DEGRADED_TO_STATIC"));
        assert!(result.quality.overall_score > 0.0);
    }
}
