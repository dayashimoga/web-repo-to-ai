//! # Universal Webpage Extraction Golden Dataset Suite
//!
//! Comprehensive regression test suite validating universal webpage extraction across
//! 8 real-world HTML archetypes:
//! 1. Technical Blog Article (Markdown headings, author byline, dates, code syntax, links, OpenGraph, JSON-LD)
//! 2. Docusaurus Documentation (breadcrumbs, admonitions, TOC, pagination, versioning, framework detection)
//! 3. Complex Technical Tables & API Specifications (rectangular normalization, GFM Markdown tables, alignments)
//! 4. Polyglot Code Tutorial with Line Gutters (gutter stripping, multi-language detection, whitespace fidelity)
//! 5. Noisy Malformed HTML (unclosed tags, ad containers, cookie banners, tracking scripts pruned)
//! 6. Dynamic SPA Hydration Lifecycle (SPA shell detection, browser hydration, dynamic provenance)
//! 7. Canonical Resolution & SimHash Near-Deduplication (canonical tags, exact/near duplicate classification)
//! 8. Sparse Stub Diagnostics & Quality Penalties (low density, missing metadata, CDM acceptance gating)

use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

use chrono::Utc;
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};
use web_repo_to_ai_web_extractor::{
    AdmonitionType, ArticleExtractor, BoilerplatePolicy, CanonicalSource, CanonicalUrlResolver,
    CompletenessLevel, DedupOptions, DocExtractor, DocFramework, DuplicateClassification,
    DuplicateRegistry, DynamicProvenance, ElementsExtractor, ExtractedElements, ExtractionMethod,
    HttpProvenance, MetadataExtractor, ProvenanceCitations, QualityScorer, SimHasher,
    SourceAttribution, SpaDetector, TableAlignment,
};

// ============================================================================
// ARCHETYPE 1: Technical Blog Article
// ============================================================================

#[test]
fn test_golden_archetype_1_technical_blog_article() {
    let policy = UrlSecurityPolicy::default();
    let base_url = ValidatedUrl::parse("https://blog.rust-lang.org/2026/async-deep-dive", &policy)
        .expect("valid url");

    let raw_html = r#"
        <!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="utf-8">
            <title>Zero-Cost Asynchronous Architecture in Rust - The Rust Programming Language</title>
            <meta name="description" content="In-depth analysis of async runtimes, cooperative scheduling, and executor ergonomics.">
            <meta name="author" content="Mara Bos & Jane Engineer">
            <meta property="og:title" content="Zero-Cost Asynchronous Architecture in Rust">
            <meta property="og:description" content="In-depth analysis of async runtimes and cooperative scheduling.">
            <meta property="og:type" content="article">
            <meta property="article:published_time" content="2026-11-17T09:00:00Z">
            <meta name="twitter:card" content="summary_large_image">
            <meta name="twitter:creator" content="@rustlang">
            <link rel="canonical" href="https://blog.rust-lang.org/2026/async-deep-dive">
            <script type="application/ld+json">
            {
                "@context": "https://schema.org",
                "@type": "BlogPosting",
                "headline": "Zero-Cost Asynchronous Architecture in Rust",
                "author": {"@type": "Person", "name": "Mara Bos & Jane Engineer"},
                "datePublished": "2026-11-17T09:00:00Z",
                "dateModified": "2026-11-17T11:00:00Z"
            }
            </script>
        </head>
        <body>
            <header>
                <nav><a href="/">Home</a> | <a href="/blog">Blog</a></nav>
            </header>
            <main>
                <article class="post">
                    <h1>Zero-Cost Asynchronous Architecture in Rust</h1>
                    <p class="byline">Published on <time datetime="2026-11-17T09:00:00Z">November 17, 2026</time> by Mara Bos & Jane Engineer</p>

                    <p>Asynchronous programming in Rust represents a state-machine transformation over native futures without mandatory heap allocation or runtime overhead.</p>

                    <h2>1. Cooperative Task Scheduling</h2>
                    <p>In a work-stealing runtime like Tokio, worker threads dequeue tasks from local run-queues before attempting to steal from peer worker queues.</p>

                    <pre><code class="language-rust">
pub async fn process_job<T: AsRef<str>>(input: T) -> Result<usize, CoreError> {
    let payload = input.as_ref();
    tokio::task::yield_now().await;
    Ok(payload.len())
}
                    </code></pre>

                    <h3>2. Polling Invariants</h3>
                    <p>Futures must uphold strict wake-on-read invariants. Calling <code>cx.waker().wake_by_ref()</code> signals readiness without thread contention.</p>

                    <blockquote>
                        "Zero-cost abstractions mean what you don't use, you don't pay for, and what you do use, you couldn't write better by hand."
                    </blockquote>

                    <ul>
                        <li>Deterministic poll progress</li>
                        <li>Work-stealing execution queues</li>
                        <li>Bounded task latency</li>
                    </ul>

                    <p>For further architectural specifications, explore the <a href="/docs/executors">official executor documentation</a>.</p>
                </article>
            </main>
            <footer>
                <p>&copy; 2026 The Rust Project Developers</p>
            </footer>
        </body>
        </html>
    "#;

    let dom_parser = web_repo_to_ai_web_extractor::DomParser::new(BoilerplatePolicy::default());
    let cleaned = dom_parser
        .parse(raw_html, Some(&base_url))
        .expect("DOM parsing succeeds");

    // Article extraction
    let article_extractor = ArticleExtractor::default();
    let art = article_extractor
        .extract(raw_html, Some(&base_url))
        .expect("article extraction succeeds");

    assert_eq!(art.title, "Zero-Cost Asynchronous Architecture in Rust");
    assert!(art.byline.as_ref().is_some_and(|b| b.contains("Mara Bos")));
    assert!(art.published_time.is_some());
    assert!(art.word_count >= 80);

    // Elements extraction
    let elements_extractor = ElementsExtractor::default();
    let elements = elements_extractor
        .extract(raw_html, Some(&base_url))
        .expect("elements extraction succeeds");

    assert_eq!(elements.code_blocks.len(), 1);
    let cb = &elements.code_blocks[0];
    assert_eq!(cb.language.as_deref(), Some("rust"));
    assert!(cb.code.contains("pub async fn process_job"));
    assert!(cb.code.contains("tokio::task::yield_now().await;"));

    // Canonical & Attribution
    let canonical_res = CanonicalUrlResolver::resolve(
        &base_url,
        &base_url,
        Some("https://blog.rust-lang.org/2026/async-deep-dive"),
        Some("https://blog.rust-lang.org/2026/async-deep-dive"),
        None,
        &DedupOptions::default(),
        &policy,
    );
    assert_eq!(
        canonical_res.canonical_url.as_str(),
        "https://blog.rust-lang.org/2026/async-deep-dive"
    );
    assert_eq!(canonical_res.source, CanonicalSource::HtmlLinkRelCanonical);
    assert!(canonical_res.is_high_confidence());

    // Quality Scoring
    let attribution = SourceAttribution {
        canonical_url: canonical_res.canonical_url.as_str().to_string(),
        canonical_resolution: canonical_res,
        initial_url: base_url.as_str().to_string(),
        retrieved_at: Utc::now(),
        load_duration_ms: 35,
        extraction_method: ExtractionMethod::Static,
        http_provenance: Some(HttpProvenance {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            content_type: Some("text/html".to_string()),
            content_length: Some(raw_html.len() as u64),
            etag: Some("\"blog-async-v1\"".to_string()),
            last_modified: Some("2026-11-17T09:00:00Z".to_string()),
            cache_control: Some("public, max-age=3600".to_string()),
            server: Some("nginx".to_string()),
            pinned_ip: Some(IpAddr::V4(Ipv4Addr::new(104, 21, 44, 18))),
            redirect_history: Vec::new(),
        }),
        content_hashes: web_repo_to_ai_web_extractor::ContentHashes::compute(
            raw_html.as_bytes(),
            &cleaned.cleaned_text,
        ),
        metadata: MetadataExtractor::extract(raw_html, &cleaned.metadata),
        browser_diagnostics: None,
        citations: ProvenanceCitations {
            concise: "[Rust Blog](https://blog.rust-lang.org/2026/async-deep-dive)".to_string(),
            markdown_callout: "> Source: https://blog.rust-lang.org/2026/async-deep-dive"
                .to_string(),
            plaintext: "Source: https://blog.rust-lang.org/2026/async-deep-dive".to_string(),
        },
        warnings: Vec::new(),
    };

    let provenance = DynamicProvenance {
        method: ExtractionMethod::Static,
        http_status: 200,
        load_time_ms: 35,
        final_url: base_url.as_str().to_string(),
        redirects: Vec::new(),
        console_messages: Vec::new(),
        page_errors: Vec::new(),
        blocked_subresources: Vec::new(),
        warnings: Vec::new(),
    };

    let scorer = QualityScorer::new();
    let quality = scorer.assess_components(
        &cleaned,
        Some(&art),
        None,
        &elements,
        &attribution,
        &provenance,
        raw_html,
    );

    assert!(
        quality.overall_score >= 0.85,
        "Technical blog should achieve Complete grade (score: {})",
        quality.overall_score
    );
    assert_eq!(quality.completeness_level, CompletenessLevel::Complete);
    assert!(quality.is_acceptable_for_cdm);
    assert!(quality.metrics.heading_hierarchy_valid);
    assert_eq!(quality.metrics.heading_count, 3);
}

// ============================================================================
// ARCHETYPE 2: Docusaurus Documentation Page
// ============================================================================

#[test]
fn test_golden_archetype_2_docusaurus_documentation() {
    let policy = UrlSecurityPolicy::default();
    let base_url = ValidatedUrl::parse("https://docs.example.com/guide/architecture", &policy)
        .expect("valid url");

    let raw_html = r#"
        <!DOCTYPE html>
        <html lang="en" data-theme="dark">
        <head>
            <meta charset="utf-8">
            <meta name="generator" content="Docusaurus v3.1.1">
            <title>Architecture Guide | Example Docs</title>
            <link rel="canonical" href="https://docs.example.com/guide/architecture">
        </head>
        <body class="docs-doc-page">
            <nav class="navbar"><div class="navbar__brand">Example Docs</div></nav>
            <div class="main-wrapper docsWrapper_node">
                <aside class="theme-doc-sidebar-container docSidebarContainer_node">
                    <ul class="theme-doc-sidebar-menu"><li><a href="/guide/intro">Intro</a></li></ul>
                </aside>
                <main class="docMainContainer_node">
                    <nav class="theme-doc-breadcrumbs breadcrumbsContainer_node" aria-label="breadcrumbs">
                        <ul class="breadcrumbs">
                            <li class="breadcrumbs__item"><a href="/docs">Docs</a></li>
                            <li class="breadcrumbs__item"><a href="/docs/guide">Guide</a></li>
                            <li class="breadcrumbs__item breadcrumbs__item--active">Architecture</li>
                        </ul>
                    </nav>

                    <article>
                        <header><h1>System Architecture Specification</h1></header>

                        <div class="theme-admonition theme-admonition-tip admonition_node alert alert--success">
                            <div class="admonitionHeading_node">TIP</div>
                            <div class="admonitionContent_node"><p>Deploy worker containers with shared memory configured to at least 2 GB.</p></div>
                        </div>

                        <h2>Data Plane Components</h2>
                        <p>The processing data plane utilizes zero-copy streaming buffers across all pipeline stages.</p>

                        <div class="theme-admonition theme-admonition-warning admonition_node alert alert--warning">
                            <div class="admonitionHeading_node">WARNING</div>
                            <div class="admonitionContent_node"><p>Never configure storage mount volumes with noexec permissions.</p></div>
                        </div>

                        <h2>Control Plane Coordination</h2>
                        <p>PostgreSQL advisory locks guarantee single-leader execution during horizontal scale out.</p>

                        <nav class="pagination-nav">
                            <a class="pagination-nav__link pagination-nav__link--prev" href="/guide/intro">
                                <div class="pagination-nav__label">Previous: Introduction</div>
                            </a>
                            <a class="pagination-nav__link pagination-nav__link--next" href="/guide/storage">
                                <div class="pagination-nav__label">Next: Storage Engine</div>
                            </a>
                        </nav>
                    </article>
                </main>
            </div>
            <div class="cookie-banner"><p>We use cookies.</p></div>
        </body>
        </html>
    "#;

    let dom_parser = web_repo_to_ai_web_extractor::DomParser::new(BoilerplatePolicy::default());
    let cleaned = dom_parser
        .parse(raw_html, Some(&base_url))
        .expect("DOM parsing succeeds");

    let doc_extractor = DocExtractor::default();
    let doc = doc_extractor
        .extract(raw_html, Some(&base_url))
        .expect("doc extraction succeeds");

    assert!(cleaned.has_semantic_main);
    assert_eq!(doc.framework, DocFramework::Docusaurus);
    assert_eq!(doc.title, "System Architecture Specification");
    assert_eq!(doc.breadcrumbs.len(), 3);
    assert_eq!(doc.breadcrumbs[0].label, "Docs");
    assert_eq!(doc.breadcrumbs[2].label, "Architecture");
    assert_eq!(doc.admonitions.len(), 2);
    assert_eq!(doc.admonitions[0].kind, AdmonitionType::Tip);
    assert_eq!(doc.admonitions[1].kind, AdmonitionType::Warning);
    assert!(doc.pagination.prev.is_some());
    assert!(doc.pagination.next.is_some());
}

// ============================================================================
// ARCHETYPE 3: Complex Technical Tables & API Specifications
// ============================================================================

#[test]
fn test_golden_archetype_3_api_specification_tables() {
    let policy = UrlSecurityPolicy::default();
    let base_url =
        ValidatedUrl::parse("https://api.example.com/v1/reference", &policy).expect("valid url");

    let raw_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>API Reference & Endpoints</title></head>
        <body>
            <main>
                <h1>REST API Specification v1</h1>
                <p>All endpoints accept and return application/json envelopes following RFC 7807 problem details.</p>

                <h2>Core Job Endpoints</h2>
                <table>
                    <caption>Core Job Orchestration Matrix</caption>
                    <thead>
                        <tr>
                            <th align="left">Method</th>
                            <th align="left">Path</th>
                            <th align="center">Auth Required</th>
                            <th align="right">Rate Limit (req/min)</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>POST</td>
                            <td>/api/v1/jobs</td>
                            <td>Yes</td>
                            <td>60</td>
                        </tr>
                        <tr>
                            <td>GET</td>
                            <td>/api/v1/jobs/{id}</td>
                            <td>No</td>
                            <td>120</td>
                        </tr>
                        <tr>
                            <td>DELETE</td>
                            <td>/api/v1/jobs/{id}</td>
                            <td>Yes</td>
                            <td>30</td>
                        </tr>
                    </tbody>
                </table>

                <h2>HTTP Error Dictionary</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Status Code</th>
                            <th>Error Code</th>
                            <th>Retryable</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr><td>400</td><td>INVALID_PAYLOAD</td><td>No</td></tr>
                        <tr><td>429</td><td>RATE_LIMIT_EXCEEDED</td><td>Yes</td></tr>
                        <tr><td>503</td><td>SERVICE_UNAVAILABLE</td><td>Yes</td></tr>
                    </tbody>
                </table>
            </main>
        </body>
        </html>
    "#;

    let elements_extractor = ElementsExtractor::default();
    let elements = elements_extractor
        .extract(raw_html, Some(&base_url))
        .expect("elements extraction succeeds");

    assert_eq!(elements.tables.len(), 2, "Expected 2 normalized tables");

    // Table 1 checks
    let t1 = &elements.tables[0];
    assert!(t1.has_headers);
    assert_eq!(t1.num_cols, 4);
    assert_eq!(t1.rows.len(), 3);
    assert_eq!(t1.column_alignments[0], TableAlignment::Left);
    assert_eq!(t1.column_alignments[2], TableAlignment::Center);
    assert_eq!(t1.column_alignments[3], TableAlignment::Right);
    let md1 = t1.to_markdown();
    assert!(md1.contains("| Method | Path | Auth Required | Rate Limit (req/min) |"));
    assert!(md1.contains("| :--- | :--- | :---: | ---: |"));
    assert!(md1.contains("| POST | /api/v1/jobs | Yes | 60 |"));

    // Table 2 checks
    let t2 = &elements.tables[1];
    assert!(t2.has_headers);
    assert_eq!(t2.num_cols, 3);
    assert_eq!(t2.rows.len(), 3);
    let md2 = t2.to_markdown();
    assert!(md2.contains("| Status Code | Error Code | Retryable |"));
}

// ============================================================================
// ARCHETYPE 4: Polyglot Code Tutorial with Line Number Gutters
// ============================================================================

#[test]
fn test_golden_archetype_4_polyglot_code_with_line_gutters() {
    let policy = UrlSecurityPolicy::default();
    let base_url = ValidatedUrl::parse("https://tutorial.example.com/concurrency", &policy)
        .expect("valid url");

    let raw_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Polyglot Concurrency Tutorial</title></head>
        <body>
            <main>
                <h1>Comparing Concurrency Across Systems Languages</h1>

                <h2>Rust Task Spawning</h2>
                <pre><code class="language-rust">1  use tokio::task;
2
3  pub async fn run_worker() {
4      task::spawn(async {
5          println!("Hello from worker");
6      });
7  }</code></pre>

                <h2>Python Coroutine</h2>
                <pre><code>#!/usr/bin/env python3
import asyncio

async def fetch_item(item_id: int):
    await asyncio.sleep(0.01)
    return {"id": item_id}</code></pre>

                <h2>PostgreSQL Job Schema</h2>
                <pre><code class="language-sql">CREATE TABLE conversion_jobs (
    id UUID PRIMARY KEY,
    target_url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING'
);</code></pre>
            </main>
        </body>
        </html>
    "#;

    let elements_extractor = ElementsExtractor::default();
    let elements = elements_extractor
        .extract(raw_html, Some(&base_url))
        .expect("elements extraction succeeds");

    assert_eq!(elements.code_blocks.len(), 3);

    // Block 1 (Rust with stripped gutters)
    let b1 = &elements.code_blocks[0];
    assert_eq!(b1.language.as_deref(), Some("rust"));
    assert!(b1.has_line_numbers_stripped);
    assert!(b1.code.starts_with("use tokio::task;"));
    assert!(!b1.code.contains("1  use tokio"));

    // Block 2 (Python via shebang detection)
    let b2 = &elements.code_blocks[1];
    assert_eq!(b2.language.as_deref(), Some("python"));
    assert!(b2.code.contains("import asyncio"));

    // Block 3 (SQL)
    let b3 = &elements.code_blocks[2];
    assert_eq!(b3.language.as_deref(), Some("sql"));
    assert!(b3.code.contains("CREATE TABLE conversion_jobs"));
}

// ============================================================================
// ARCHETYPE 5: Noisy Malformed HTML
// ============================================================================

#[test]
fn test_golden_archetype_5_noisy_malformed_html() {
    let policy = UrlSecurityPolicy::default();
    let base_url =
        ValidatedUrl::parse("https://news.example.com/article", &policy).expect("valid url");

    let raw_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Unclosed Tags & Boilerplate Everywhere</title></head>
        <body>
            <div id="cookie-banner" class="cookie-banner modal">
                <p>Accept our tracking cookies! <button>OK</button></p>
            </div>
            <div class="ad-leaderboard banner-ad">
                <a href="http://adserver.example.com">Cheap Flights!</a>
            </div>
            <aside class="social-share-buttons">
                <button>Tweet</button><button>Share</button>
            </aside>
            <article id="main-content">
                <h1>High-Throughput Parsing in Memory-Constrained Environments</h1>
                <p>Modern streaming parsers process HTML without allocating the full document tree.
                <p>By discarding intermediate nodes early, peak memory remains strictly bounded.
                <div><span>Inner unclosed span with text.</div>
            </article>
            <div class="newsletter-signup">
                <h3>Subscribe!</h3>
            </div>
            <script src="https://analytics.example.com/tracker.js"></script>
        </body>
        </html>
    "#;

    let dom_parser = web_repo_to_ai_web_extractor::DomParser::new(BoilerplatePolicy::default());
    let cleaned = dom_parser
        .parse(raw_html, Some(&base_url))
        .expect("DOM parsing recovers gracefully from malformed HTML");

    assert!(
        cleaned.removed_nodes_count > 0,
        "Expected removed nodes, got {}",
        cleaned.removed_nodes_count
    );
    assert!(!cleaned.cleaned_text.contains("Accept our tracking cookies"));
    assert!(!cleaned.cleaned_text.contains("Cheap Flights"));
    assert!(cleaned
        .cleaned_text
        .contains("High-Throughput Parsing in Memory-Constrained Environments"));
    assert!(cleaned
        .cleaned_text
        .contains("Modern streaming parsers process HTML"));
}

// ============================================================================
// ARCHETYPE 6: Dynamic SPA Hydration Lifecycle
// ============================================================================

#[test]
fn test_golden_archetype_6_spa_hydration_lifecycle() {
    let policy = UrlSecurityPolicy::default();
    let base_url =
        ValidatedUrl::parse("https://app.example.com/dashboard", &policy).expect("valid url");

    // Static skeleton HTML
    let static_skeleton = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Loading Application...</title></head>
        <body>
            <div id="root"></div>
            <noscript>You need to enable JavaScript to view this application.</noscript>
        </body>
        </html>
    "#;

    let dom_parser = web_repo_to_ai_web_extractor::DomParser::new(BoilerplatePolicy::default());
    let cleaned_static = dom_parser
        .parse(static_skeleton, Some(&base_url))
        .expect("DOM parse static succeeds");

    let detector = SpaDetector::default();
    assert!(
        detector.is_fallback_needed(&cleaned_static, static_skeleton),
        "SPA skeleton must trigger dynamic fallback"
    );

    // Hydrated dynamic DOM
    let hydrated_html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Analytics Dashboard | App</title></head>
        <body>
            <main>
                <h1>Real-Time Cluster Telemetry</h1>
                <p>Cluster operational status: 48 worker nodes healthy across 3 regions.</p>
                <div class="metrics">
                    <p>Throughput: 142,500 requests per second. P99 latency: 1.4 ms.</p>
                </div>
            </main>
        </body>
        </html>
    "#;

    let cleaned_hydrated = dom_parser
        .parse(hydrated_html, Some(&base_url))
        .expect("DOM parse hydrated succeeds");

    assert!(
        !detector.is_fallback_needed(&cleaned_hydrated, hydrated_html),
        "Hydrated DOM should not require further fallback"
    );
    assert!(cleaned_hydrated
        .cleaned_text
        .contains("Real-Time Cluster Telemetry"));
}

// ============================================================================
// ARCHETYPE 7: Canonical Resolution & SimHash Near-Deduplication
// ============================================================================

#[test]
fn test_golden_archetype_7_canonical_and_simhash_dedup() {
    let policy = UrlSecurityPolicy::default();
    let _url_a = ValidatedUrl::parse("https://example.com/docs/intro", &policy).unwrap();
    let url_b =
        ValidatedUrl::parse("https://example.com/docs/intro?utm_source=twitter", &policy).unwrap();

    let _page_a_html = r#"
        <html>
        <head><link rel="canonical" href="https://example.com/docs/intro"></head>
        <body><main><h1>Introduction to Engine</h1><p>The universal conversion engine extracts web pages and repositories into AI context documents deterministically.</p></main></body>
        </html>
    "#;

    let _page_b_html = r#"
        <html>
        <head><link rel="canonical" href="https://example.com/docs/intro"></head>
        <body><main><h1>Introduction to Engine</h1><p>The universal conversion engine extracts web pages and repositories into AI context documents deterministically.</p></main></body>
        </html>
    "#;

    let text_a = "Introduction to Universal Conversion Engine. The universal conversion engine extracts web pages, documentation hierarchies, and software repositories into AI context documents deterministically. It preserves syntax highlighting, structural headings, and cryptographic provenance hashes across all output formats.";
    let page_c_near_duplicate_text = "Introduction to Universal Conversion Platform. The universal conversion engine extracts web pages, documentation hierarchies, and software repositories into AI context documents deterministically. It preserves syntax highlighting, structural headings, and cryptographic provenance hashes across all output formats.";
    let page_d_unique_text = "PostgreSQL Database Administration. Comprehensive manual covering vacuum tuning, write ahead logging, and streaming replication topology.";

    // 1. Canonical Resolution
    let res_b = CanonicalUrlResolver::resolve(
        &url_b,
        &url_b,
        Some("https://example.com/docs/intro"),
        None,
        None,
        &DedupOptions::default(),
        &policy,
    );
    assert_eq!(
        res_b.canonical_url.as_str(),
        "https://example.com/docs/intro"
    );
    assert_eq!(res_b.source, CanonicalSource::HtmlLinkRelCanonical);

    // 2. Registry Deduplication
    let mut registry = DuplicateRegistry::new();

    let hash_a = blake3::hash(text_a.as_bytes()).to_hex().to_string();
    let simhash_a = SimHasher::compute(text_a);

    // Register original Page A
    let first = registry.check_and_register("https://example.com/docs/intro", &hash_a, text_a);
    assert_eq!(first, DuplicateClassification::Unique);

    // Page B has same canonical URL
    let second = registry.check_and_register("https://example.com/docs/intro", &hash_a, text_a);
    assert!(matches!(
        second,
        DuplicateClassification::ExactUrlDuplicate { .. }
    ));

    // Page C has slight word variation (Near-Duplicate)
    let hash_c = blake3::hash(page_c_near_duplicate_text.as_bytes())
        .to_hex()
        .to_string();
    let simhash_c = SimHasher::compute(page_c_near_duplicate_text);
    let distance = SimHasher::hamming_distance(simhash_a, simhash_c);
    assert!(
        distance <= 8,
        "Hamming distance should be <= 8 for near-duplicate"
    );

    let third = registry.check_and_register(
        "https://example.com/docs/intro-variant",
        &hash_c,
        page_c_near_duplicate_text,
    );
    assert!(matches!(
        third,
        DuplicateClassification::NearDuplicate { .. }
    ));

    // Page D is distinct (Unique)
    let hash_d = blake3::hash(page_d_unique_text.as_bytes())
        .to_hex()
        .to_string();
    let fourth = registry.check_and_register(
        "https://example.com/docs/postgres",
        &hash_d,
        page_d_unique_text,
    );
    assert_eq!(fourth, DuplicateClassification::Unique);
}

// ============================================================================
// ARCHETYPE 8: Sparse Stub Diagnostics & Quality Penalties
// ============================================================================

#[test]
fn test_golden_archetype_8_sparse_stub_diagnostics() {
    let policy = UrlSecurityPolicy::default();
    let base_url = ValidatedUrl::parse("https://example.com/stub", &policy).unwrap();

    let stub_html = "<html><body><p>Under construction.</p></body></html>";

    let dom_parser = web_repo_to_ai_web_extractor::DomParser::new(BoilerplatePolicy::default());
    let cleaned = dom_parser.parse(stub_html, Some(&base_url)).unwrap();

    let provenance = DynamicProvenance {
        method: ExtractionMethod::Static,
        http_status: 200,
        load_time_ms: 10,
        final_url: base_url.as_str().to_string(),
        redirects: Vec::new(),
        console_messages: Vec::new(),
        page_errors: Vec::new(),
        blocked_subresources: Vec::new(),
        warnings: Vec::new(),
    };

    let attribution = SourceAttribution {
        canonical_url: base_url.as_str().to_string(),
        canonical_resolution: web_repo_to_ai_web_extractor::CanonicalResolution {
            canonical_url: base_url.clone(),
            source: CanonicalSource::OriginalRequest,
            raw_candidate: None,
            is_cross_domain: false,
            confidence: 50,
            warnings: Vec::new(),
        },
        initial_url: base_url.as_str().to_string(),
        retrieved_at: Utc::now(),
        load_duration_ms: 10,
        extraction_method: ExtractionMethod::Static,
        http_provenance: None,
        content_hashes: web_repo_to_ai_web_extractor::ContentHashes::compute(
            stub_html.as_bytes(),
            &cleaned.cleaned_text,
        ),
        metadata: web_repo_to_ai_web_extractor::CapturedMetadata::default(),
        browser_diagnostics: None,
        citations: ProvenanceCitations {
            concise: "Stub".to_string(),
            markdown_callout: "> Stub".to_string(),
            plaintext: "Stub".to_string(),
        },
        warnings: Vec::new(),
    };

    let elements = ExtractedElements::default();
    let scorer = QualityScorer::new();
    let quality = scorer.assess_components(
        &cleaned,
        None,
        None,
        &elements,
        &attribution,
        &provenance,
        stub_html,
    );

    assert!(
        quality.overall_score < 0.40,
        "Sparse stub score should be heavily penalized"
    );
    assert!(
        quality.completeness_level == CompletenessLevel::Partial
            || quality.completeness_level == CompletenessLevel::Degraded
    );
    assert!(!quality.is_acceptable_for_cdm);
    assert!(quality.warning_codes().contains(&"CONTENT_VERY_SPARSE"));
    assert!(quality.warning_codes().contains(&"METADATA_MISSING_TITLE"));
    assert!(quality.warning_codes().contains(&"METADATA_MISSING_AUTHOR"));
}
