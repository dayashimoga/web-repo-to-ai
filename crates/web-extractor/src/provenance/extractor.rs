//! # Metadata Extractor & Provenance Builder
//!
//! Orchestrates the aggregation of HTML metadata, OpenGraph, Twitter Cards, Schema.org JSON-LD,
//! HTTP headers, cryptographic content hashes, and structured citations.

use std::collections::HashMap;
use std::time::Instant;

use chrono::Utc;
use serde_json::Value;
use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

use super::model::{
    BrowserDiagnostics, CapturedMetadata, ContentHashes, HttpProvenance, ProvenanceCitations,
    ProvenanceWarning, RedirectHopRecord, SourceAttribution, WarningSeverity,
};
use crate::browser::model::BrowserRenderResponse;
use crate::browser::ExtractionMethod;
use crate::dom::{CleanedDocument, DocumentMetadata};
use crate::http::FetchedResponse;

/// Extractor for structured document metadata and Schema.org JSON-LD blocks.
#[derive(Debug, Clone, Default)]
pub struct MetadataExtractor;

impl MetadataExtractor {
    /// Extracts comprehensive document metadata combining HTML `<head>`, OpenGraph, Twitter, and Schema.org.
    #[must_use]
    pub fn extract(html: &str, dom_meta: &DocumentMetadata) -> CapturedMetadata {
        let json_ld_blocks = Self::extract_json_ld_blocks(html);
        let parsed_json_ld = Self::parse_json_ld(&json_ld_blocks);

        // Precedence: OpenGraph -> Twitter -> DOM meta -> JSON-LD -> fallback
        let title = dom_meta
            .og_title
            .as_ref()
            .or(dom_meta.twitter_title.as_ref())
            .or(dom_meta.title.as_ref())
            .or(parsed_json_ld.title.as_ref())
            .cloned();

        let description = dom_meta
            .og_description
            .as_ref()
            .or(dom_meta.twitter_description.as_ref())
            .or(dom_meta.description.as_ref())
            .or(parsed_json_ld.description.as_ref())
            .cloned();

        let author = parsed_json_ld
            .author
            .as_ref()
            .or(dom_meta.author.as_ref())
            .cloned();

        let published_date = parsed_json_ld
            .published_date
            .as_ref()
            .or(dom_meta.published_time.as_ref())
            .cloned();

        let modified_date = parsed_json_ld.modified_date.clone();

        let language = dom_meta
            .language
            .as_ref()
            .or(parsed_json_ld.language.as_ref())
            .cloned();

        let site_name = dom_meta
            .og_site_name
            .as_ref()
            .or(parsed_json_ld.site_name.as_ref())
            .cloned();

        let license = Self::extract_license(html).or(parsed_json_ld.license);

        let mut opengraph = HashMap::new();
        if let Some(ref t) = dom_meta.og_title {
            opengraph.insert("og:title".to_string(), t.clone());
        }
        if let Some(ref d) = dom_meta.og_description {
            opengraph.insert("og:description".to_string(), d.clone());
        }
        if let Some(ref img) = dom_meta.og_image {
            opengraph.insert("og:image".to_string(), img.clone());
        }
        if let Some(ref s) = dom_meta.og_site_name {
            opengraph.insert("og:site_name".to_string(), s.clone());
        }

        let mut twitter = HashMap::new();
        if let Some(ref t) = dom_meta.twitter_title {
            twitter.insert("twitter:title".to_string(), t.clone());
        }
        if let Some(ref d) = dom_meta.twitter_description {
            twitter.insert("twitter:description".to_string(), d.clone());
        }

        CapturedMetadata {
            title,
            description,
            author,
            published_date,
            modified_date,
            language,
            site_name,
            keywords: dom_meta.keywords.clone(),
            license,
            canonical_link: dom_meta.canonical_url.clone(),
            opengraph,
            twitter,
            json_ld: json_ld_blocks,
        }
    }

    /// Queries HTML for all `<script type="application/ld+json">` text blocks.
    #[must_use]
    pub fn extract_json_ld_blocks(html: &str) -> Vec<String> {
        let mut blocks = Vec::new();
        let lower = html.to_ascii_lowercase();
        let target = "application/ld+json";

        let mut cursor = 0;
        while let Some(pos) = lower[cursor..].find(target) {
            let actual_pos = cursor + pos;
            // Find opening tag end
            if let Some(tag_end) = html[actual_pos..].find('>') {
                let start_idx = actual_pos + tag_end + 1;
                // Find closing </script>
                let script_close = "</script>";
                if let Some(close_pos) = lower[start_idx..].find(script_close) {
                    let end_idx = start_idx + close_pos;
                    let script_body = html[start_idx..end_idx].trim();
                    if !script_body.is_empty() {
                        blocks.push(script_body.to_string());
                    }
                    cursor = end_idx + script_close.len();
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        blocks
    }

    /// Parses raw JSON-LD blocks into candidate metadata fields.
    fn parse_json_ld(blocks: &[String]) -> ParsedJsonLd {
        let mut parsed = ParsedJsonLd::default();

        for block in blocks {
            if let Ok(val) = serde_json::from_str::<Value>(block) {
                Self::extract_from_json_val(&val, &mut parsed);
            }
        }

        parsed
    }

    fn extract_from_json_val(val: &Value, parsed: &mut ParsedJsonLd) {
        match val {
            Value::Object(map) => {
                if parsed.title.is_none() {
                    if let Some(headline) = map.get("headline").and_then(Value::as_str) {
                        parsed.title = Some(headline.to_string());
                    } else if let Some(name) = map.get("name").and_then(Value::as_str) {
                        parsed.title = Some(name.to_string());
                    }
                }

                if parsed.description.is_none() {
                    if let Some(desc) = map.get("description").and_then(Value::as_str) {
                        parsed.description = Some(desc.to_string());
                    }
                }

                if parsed.author.is_none() {
                    if let Some(auth_val) = map.get("author") {
                        if let Some(auth_str) = auth_val.as_str() {
                            parsed.author = Some(auth_str.to_string());
                        } else if let Some(auth_obj) = auth_val.as_object() {
                            if let Some(name) = auth_obj.get("name").and_then(Value::as_str) {
                                parsed.author = Some(name.to_string());
                            }
                        } else if let Some(auth_arr) = auth_val.as_array() {
                            if let Some(first) = auth_arr.first().and_then(Value::as_object) {
                                if let Some(name) = first.get("name").and_then(Value::as_str) {
                                    parsed.author = Some(name.to_string());
                                }
                            }
                        }
                    }
                }

                if parsed.published_date.is_none() {
                    if let Some(pub_date) = map.get("datePublished").and_then(Value::as_str) {
                        parsed.published_date = Some(pub_date.to_string());
                    }
                }

                if parsed.modified_date.is_none() {
                    if let Some(mod_date) = map.get("dateModified").and_then(Value::as_str) {
                        parsed.modified_date = Some(mod_date.to_string());
                    }
                }

                if parsed.site_name.is_none() {
                    if let Some(pub_val) = map.get("publisher") {
                        if let Some(name) = pub_val.get("name").and_then(Value::as_str) {
                            parsed.site_name = Some(name.to_string());
                        }
                    }
                }

                if parsed.license.is_none() {
                    if let Some(lic) = map.get("license").and_then(Value::as_str) {
                        parsed.license = Some(lic.to_string());
                    }
                }

                if parsed.language.is_none() {
                    if let Some(lang) = map.get("inLanguage").and_then(Value::as_str) {
                        parsed.language = Some(lang.to_string());
                    }
                }

                // If @graph array is present, descend into each node
                if let Some(Value::Array(graph)) = map.get("@graph") {
                    for item in graph {
                        Self::extract_from_json_val(item, parsed);
                    }
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    Self::extract_from_json_val(item, parsed);
                }
            }
            _ => {}
        }
    }

    /// Extracts copyright/license link or meta tag.
    fn extract_license(html: &str) -> Option<String> {
        let lower = html.to_ascii_lowercase();
        // Check for <link rel="license" href="...">
        if let Some(idx) = lower.find("rel=\"license\"") {
            let start = idx.saturating_sub(50);
            let end = (idx + 150).min(html.len());
            let snippet = &lower[start..end];
            if let Some(href_idx) = snippet.find("href=\"") {
                let val_start = href_idx + 6;
                if let Some(val_end) = snippet[val_start..].find('\"') {
                    return Some(snippet[val_start..val_start + val_end].to_string());
                }
            }
        }

        None
    }
}

#[derive(Default)]
struct ParsedJsonLd {
    title: Option<String>,
    description: Option<String>,
    author: Option<String>,
    published_date: Option<String>,
    modified_date: Option<String>,
    language: Option<String>,
    site_name: Option<String>,
    license: Option<String>,
}

/// Builder responsible for synthesizing `SourceAttribution` from HTTP or browser responses.
pub struct ProvenanceBuilder;

impl ProvenanceBuilder {
    /// Builds `SourceAttribution` for static HTTP acquisition.
    #[must_use]
    pub fn build_static(
        fetched: &FetchedResponse,
        cleaned_doc: &CleanedDocument,
        raw_html: &str,
        start_time: Instant,
        policy: &UrlSecurityPolicy,
    ) -> SourceAttribution {
        let retrieved_at = Utc::now();
        let load_duration_ms = start_time.elapsed().as_millis() as u64;

        let content_hashes =
            ContentHashes::compute(fetched.body_bytes(), &cleaned_doc.cleaned_text);
        let captured_metadata = MetadataExtractor::extract(raw_html, &cleaned_doc.metadata);

        let mut warnings = Vec::new();

        // 1. Resolve canonical URL & compare against fetched URL
        let terminal_url = fetched.url.as_str().to_string();
        let mut canonical_url = terminal_url.clone();

        if let Some(ref link) = captured_metadata.canonical_link {
            if let Ok(parsed_canonical) = ValidatedUrl::parse(link, policy) {
                let parsed_str = parsed_canonical.as_str();
                if parsed_str.trim_end_matches('/') != terminal_url.trim_end_matches('/') {
                    warnings.push(ProvenanceWarning::new(
                        "CANONICAL_URL_MISMATCH",
                        format!(
                            "HTML rel=canonical '{parsed_str}' differs from HTTP final URL '{terminal_url}'."
                        ),
                        WarningSeverity::Warning,
                    ));
                }
                canonical_url = parsed_str.to_string();
            }
        }

        // 2. HTTP Cache validator warnings
        if fetched.etag.is_none() && fetched.last_modified.is_none() {
            warnings.push(ProvenanceWarning::new(
                "MISSING_HTTP_CACHE_VALIDATORS",
                "HTTP response does not declare ETag or Last-Modified caching headers.".to_string(),
                WarningSeverity::Info,
            ));
        }

        // 3. Metadata completeness warning
        if captured_metadata.author.is_none() {
            warnings.push(ProvenanceWarning::new(
                "MISSING_AUTHOR",
                "Document author could not be identified from HTML metadata or JSON-LD."
                    .to_string(),
                WarningSeverity::Info,
            ));
        }

        // 4. Redirect history conversion
        let redirect_history = fetched
            .redirect_history
            .iter()
            .map(|hop| RedirectHopRecord {
                from_url: hop.from_url.as_str().to_string(),
                to_url: hop.to_url.as_str().to_string(),
                status_code: hop.status_code,
                pinned_ip: Some(hop.pinned_ip),
                timestamp: hop.timestamp,
            })
            .collect();

        let http_provenance = HttpProvenance {
            status_code: fetched.status,
            status_text: fetched.status_text.clone(),
            headers: fetched.headers.clone(),
            content_type: fetched.content_type.clone(),
            content_length: fetched.content_length,
            etag: fetched.etag.clone(),
            last_modified: fetched.last_modified.clone(),
            cache_control: fetched.headers.get("cache-control").cloned(),
            server: fetched.headers.get("server").cloned(),
            pinned_ip: Some(fetched.pinned_ip),
            redirect_history,
        };

        let citations = Self::format_citations(
            &canonical_url,
            retrieved_at,
            ExtractionMethod::Static,
            &content_hashes,
        );

        SourceAttribution {
            canonical_url,
            initial_url: fetched.initial_url.as_str().to_string(),
            retrieved_at,
            load_duration_ms,
            extraction_method: ExtractionMethod::Static,
            http_provenance: Some(http_provenance),
            content_hashes,
            metadata: captured_metadata,
            browser_diagnostics: None,
            citations,
            warnings,
        }
    }

    /// Builds `SourceAttribution` for dynamic browser rendering.
    #[must_use]
    pub fn build_dynamic(
        initial_url: &str,
        resp: &BrowserRenderResponse,
        cleaned_doc: &CleanedDocument,
        start_time: Instant,
        policy: &UrlSecurityPolicy,
    ) -> SourceAttribution {
        let retrieved_at = Utc::now();
        let load_duration_ms = start_time.elapsed().as_millis() as u64;

        let content_hashes =
            ContentHashes::compute(resp.html.as_bytes(), &cleaned_doc.cleaned_text);
        let captured_metadata = MetadataExtractor::extract(&resp.html, &cleaned_doc.metadata);

        let mut warnings = Vec::new();

        let terminal_url = if resp.url.is_empty() {
            initial_url.to_string()
        } else {
            resp.url.clone()
        };

        let mut canonical_url = terminal_url.clone();
        if let Some(ref link) = captured_metadata.canonical_link {
            if let Ok(parsed_canonical) = ValidatedUrl::parse(link, policy) {
                let parsed_str = parsed_canonical.as_str();
                if parsed_str.trim_end_matches('/') != terminal_url.trim_end_matches('/') {
                    warnings.push(ProvenanceWarning::new(
                        "CANONICAL_URL_MISMATCH",
                        format!(
                            "HTML rel=canonical '{parsed_str}' differs from browser final URL '{terminal_url}'."
                        ),
                        WarningSeverity::Warning,
                    ));
                }
                canonical_url = parsed_str.to_string();
            }
        }

        if !resp.blocked_subresources.is_empty() {
            warnings.push(ProvenanceWarning::new(
                "BLOCKED_SUBRESOURCES_PRESENT",
                format!(
                    "{} subresource requests were intercepted and blocked by sandbox policy.",
                    resp.blocked_subresources.len()
                ),
                WarningSeverity::Warning,
            ));
        }

        let redirect_history = resp
            .redirects
            .iter()
            .map(|r| RedirectHopRecord {
                from_url: r.from.clone(),
                to_url: r.to.clone(),
                status_code: r.status,
                pinned_ip: None,
                timestamp: retrieved_at,
            })
            .collect();

        let browser_diagnostics = BrowserDiagnostics {
            method: ExtractionMethod::DynamicBrowser,
            timing: Some(resp.timing.clone()),
            console_messages: resp.console_messages.clone(),
            page_errors: resp.page_errors.clone(),
            blocked_subresources: resp.blocked_subresources.clone(),
        };

        let citations = Self::format_citations(
            &canonical_url,
            retrieved_at,
            ExtractionMethod::DynamicBrowser,
            &content_hashes,
        );

        let http_provenance = HttpProvenance {
            status_code: resp.status_code,
            status_text: if resp.status_code == 200 {
                "OK".to_string()
            } else {
                format!("HTTP {}", resp.status_code)
            },
            headers: HashMap::new(),
            content_type: Some("text/html; charset=utf-8".to_string()),
            content_length: Some(resp.html.len() as u64),
            etag: None,
            last_modified: None,
            cache_control: None,
            server: None,
            pinned_ip: None,
            redirect_history,
        };

        SourceAttribution {
            canonical_url,
            initial_url: initial_url.to_string(),
            retrieved_at,
            load_duration_ms,
            extraction_method: ExtractionMethod::DynamicBrowser,
            http_provenance: Some(http_provenance),
            content_hashes,
            metadata: captured_metadata,
            browser_diagnostics: Some(browser_diagnostics),
            citations,
            warnings,
        }
    }

    /// Formats citations into concise, markdown callout, and plaintext styles.
    #[must_use]
    pub fn format_citations(
        canonical_url: &str,
        retrieved_at: chrono::DateTime<Utc>,
        method: ExtractionMethod,
        hashes: &ContentHashes,
    ) -> ProvenanceCitations {
        let iso_time = retrieved_at.to_rfc3339();
        let method_str = match method {
            ExtractionMethod::Static => "Static HTTP",
            ExtractionMethod::DynamicBrowser => "Dynamic Browser (Playwright / Chromium)",
        };

        let concise = format!(
            "Source: {canonical_url} | Retrieved: {iso_time} | BLAKE3: {}",
            hashes.raw_html_blake3
        );

        let markdown_callout = format!(
            "> [!NOTE]\n> **Source**: [{canonical_url}]({canonical_url})  \n> **Retrieved**: {iso_time}  \n> **Method**: {method_str}  \n> **Content Hash (BLAKE3)**: `{}`  \n> **Cleaned Text Hash (BLAKE3)**: `{}`",
            hashes.raw_html_blake3, hashes.cleaned_text_blake3
        );

        let plaintext = format!(
            "Source: {canonical_url}\nRetrieved: {iso_time}\nMethod: {method_str}\nContent Hash (BLAKE3): {}\nCleaned Text Hash (BLAKE3): {}",
            hashes.raw_html_blake3, hashes.cleaned_text_blake3
        );

        ProvenanceCitations {
            concise,
            markdown_callout,
            plaintext,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use web_repo_to_ai_url_security::UrlSecurityPolicy;

    #[test]
    fn test_content_hashes_computation() {
        let raw_html = b"<!DOCTYPE html><html><body><h1>Hello World</h1></body></html>";
        let cleaned_text = "Hello World";

        let hashes = ContentHashes::compute(raw_html, cleaned_text);

        assert_eq!(hashes.raw_html_sha256.len(), 64);
        assert_eq!(hashes.raw_html_blake3.len(), 64);
        assert_eq!(hashes.cleaned_text_sha256.len(), 64);
        assert_eq!(hashes.cleaned_text_blake3.len(), 64);

        // Deterministic check
        let hashes2 = ContentHashes::compute(raw_html, cleaned_text);
        assert_eq!(hashes, hashes2);
    }

    #[test]
    fn test_metadata_extractor_json_ld_and_cascade() {
        let html = r#"
            <!DOCTYPE html>
            <html lang="en">
                <head>
                    <title>HTML Fallback Title</title>
                    <meta property="og:title" content="OpenGraph Dominant Title">
                    <meta property="og:description" content="OpenGraph Description">
                    <meta property="og:site_name" content="Tech Docs">
                    <link rel="license" href="https://creativecommons.org/licenses/by/4.0/">
                    <script type="application/ld+json">
                    {
                        "@context": "https://schema.org",
                        "@type": "TechArticle",
                        "headline": "JSON-LD Headline",
                        "author": {
                            "@type": "Person",
                            "name": "Ferris Crab"
                        },
                        "datePublished": "2026-11-17T09:00:00Z",
                        "dateModified": "2026-11-17T11:00:00Z"
                    }
                    </script>
                </head>
                <body>
                    <p>Sample content</p>
                </body>
            </html>
        "#;

        let dom_meta = DocumentMetadata {
            title: Some("HTML Fallback Title".to_string()),
            og_title: Some("OpenGraph Dominant Title".to_string()),
            og_description: Some("OpenGraph Description".to_string()),
            og_site_name: Some("Tech Docs".to_string()),
            language: Some("en".to_string()),
            ..Default::default()
        };

        let meta = MetadataExtractor::extract(html, &dom_meta);

        assert_eq!(meta.title.as_deref(), Some("OpenGraph Dominant Title"));
        assert_eq!(meta.description.as_deref(), Some("OpenGraph Description"));
        assert_eq!(meta.author.as_deref(), Some("Ferris Crab"));
        assert_eq!(meta.published_date.as_deref(), Some("2026-11-17T09:00:00Z"));
        assert_eq!(meta.modified_date.as_deref(), Some("2026-11-17T11:00:00Z"));
        assert_eq!(meta.site_name.as_deref(), Some("Tech Docs"));
        assert_eq!(
            meta.license.as_deref(),
            Some("https://creativecommons.org/licenses/by/4.0/")
        );
        assert_eq!(meta.json_ld.len(), 1);
    }

    #[test]
    fn test_provenance_builder_static_attribution_and_warnings() {
        let policy = UrlSecurityPolicy::default();
        let target_url =
            ValidatedUrl::parse("https://example.com/blog/rust-tips", &policy).unwrap();

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "text/html".to_string());

        let html = "<!DOCTYPE html><html><head><link rel=\"canonical\" href=\"https://example.com/rust-tips\"></head><body><h1>Content</h1></body></html>";

        let fetched = FetchedResponse {
            url: target_url.clone(),
            initial_url: target_url,
            status: 200,
            status_text: "OK".to_string(),
            headers,
            content_type: Some("text/html".to_string()),
            content_length: Some(html.len() as u64),
            content_encoding: None,
            raw_body_size: html.len(),
            etag: None,
            last_modified: None,
            redirect_history: Vec::new(),
            body: html.as_bytes().to_vec(),
            pinned_ip: IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            fetched_at: Utc::now(),
        };

        let dom_meta = DocumentMetadata {
            canonical_url: Some("https://example.com/rust-tips".to_string()),
            ..Default::default()
        };

        let cleaned_doc = CleanedDocument {
            metadata: dom_meta,
            raw_html_bytes: html.len(),
            cleaned_html: html.to_string(),
            cleaned_text: "Content".to_string(),
            removed_nodes_count: 0,
            has_semantic_main: false,
        };

        let attribution =
            ProvenanceBuilder::build_static(&fetched, &cleaned_doc, html, Instant::now(), &policy);

        assert_eq!(attribution.canonical_url, "https://example.com/rust-tips");
        assert_eq!(attribution.extraction_method, ExtractionMethod::Static);
        assert!(attribution.has_warnings());

        let codes = attribution.warning_codes();
        assert!(codes.contains(&"CANONICAL_URL_MISMATCH"));
        assert!(codes.contains(&"MISSING_HTTP_CACHE_VALIDATORS"));
        assert!(codes.contains(&"MISSING_AUTHOR"));

        let citation = attribution.markdown_citation();
        assert!(citation.contains("Source"));
        assert!(citation.contains("Content Hash (BLAKE3)"));
    }
}
