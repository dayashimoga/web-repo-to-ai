//! # Completeness Scorer & Quality Assessor
//!
//! Evaluates extracted webpage models across metadata, content volume, structural fidelity,
//! and provenance to compute multidimensional completeness scores, identify diagnostic warnings,
//! and determine CDM document conversion suitability.

use serde::{Deserialize, Serialize};

use super::model::{
    CompletenessLevel, ContentMetrics, DimensionScores, ExtractionQualityReport, ExtractionWarning,
    ExtractionWarningCategory,
};
use crate::article::ExtractedArticle;
use crate::browser::extractor::ExtractedWebpage;
use crate::docs::ExtractedDocumentation;
use crate::dom::CleanedDocument;
use crate::elements::{ExtractedElements, LinkType};
use crate::provenance::{SourceAttribution, WarningSeverity};

/// Configurable weights and evaluation parameters for the quality scorer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityScorerOptions {
    /// Relative weight of metadata completeness in overall score (default 0.20).
    pub metadata_weight: f64,
    /// Relative weight of content substance and density in overall score (default 0.35).
    pub content_weight: f64,
    /// Relative weight of structural fidelity (headings, code, tables) in overall score (default 0.25).
    pub structural_weight: f64,
    /// Relative weight of acquisition provenance in overall score (default 0.20).
    pub provenance_weight: f64,
    /// Target word count for full content score saturation (default 250 words).
    pub target_word_count: usize,
    /// Minimum content density threshold before raising low-density warnings (default 0.02).
    pub min_content_density: f64,
    /// Minimum overall completeness score threshold for CDM conversion acceptance (default 0.25).
    pub cdm_acceptance_threshold: f64,
}

impl Default for QualityScorerOptions {
    fn default() -> Self {
        Self {
            metadata_weight: 0.20,
            content_weight: 0.35,
            structural_weight: 0.25,
            provenance_weight: 0.20,
            target_word_count: 250,
            min_content_density: 0.02,
            cdm_acceptance_threshold: 0.25,
        }
    }
}

/// Evaluator that computes completeness scores, quantitative metrics, and extraction warnings.
#[derive(Debug, Clone, Default)]
pub struct QualityScorer {
    options: QualityScorerOptions,
}

impl QualityScorer {
    /// Constructs a new quality scorer with default weights.
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: QualityScorerOptions::default(),
        }
    }

    /// Constructs a quality scorer with custom options.
    #[must_use]
    pub fn with_options(options: QualityScorerOptions) -> Self {
        Self { options }
    }

    /// Evaluates a fully hydrated `ExtractedWebpage` and raw HTML source.
    #[must_use]
    pub fn assess_webpage(
        &self,
        webpage: &ExtractedWebpage,
        raw_html: &str,
    ) -> ExtractionQualityReport {
        self.assess_components(
            &webpage.cleaned_doc,
            webpage.article.as_ref(),
            webpage.documentation.as_ref(),
            &webpage.elements,
            &webpage.attribution,
            &webpage.provenance,
            raw_html,
        )
    }

    /// Evaluates individual extraction pipeline components.
    #[must_use]
    #[allow(clippy::too_many_arguments, clippy::too_many_lines)]
    pub fn assess_components(
        &self,
        cleaned_doc: &CleanedDocument,
        article: Option<&ExtractedArticle>,
        documentation: Option<&ExtractedDocumentation>,
        elements: &ExtractedElements,
        attribution: &SourceAttribution,
        provenance: &crate::browser::extractor::DynamicProvenance,
        raw_html: &str,
    ) -> ExtractionQualityReport {
        let mut warnings = Vec::new();

        // 1. Quantify text and document metrics
        let raw_html_bytes = if raw_html.is_empty() {
            cleaned_doc.raw_html_bytes
        } else {
            raw_html.len()
        };
        let cleaned_text = cleaned_doc.cleaned_text.trim();
        let cleaned_text_bytes = cleaned_text.len();
        let word_count = if cleaned_text.is_empty() {
            0
        } else {
            cleaned_text.split_whitespace().count()
        };
        let character_count = cleaned_text.chars().count();
        let reading_time_minutes = if word_count > 0 {
            word_count.div_ceil(200)
        } else {
            0
        };
        let paragraph_count = count_paragraphs(cleaned_text);
        let content_density = if raw_html_bytes > 0 {
            cleaned_text_bytes as f64 / raw_html_bytes as f64
        } else {
            0.0
        };
        let boilerplate_ratio = if raw_html_bytes > 0 {
            ((raw_html_bytes.saturating_sub(cleaned_text_bytes)) as f64 / raw_html_bytes as f64)
                .clamp(0.0, 1.0)
        } else {
            0.0
        };

        // 2. Heading analysis
        let (heading_count, headings_by_level, heading_hierarchy_valid, heading_warnings) =
            analyze_headings(&cleaned_doc.cleaned_html);
        warnings.extend(heading_warnings);

        // 3. Code blocks analysis
        let code_block_count = elements.code_blocks.len();
        let code_blocks_with_syntax = elements
            .code_blocks
            .iter()
            .filter(|cb| {
                cb.language.as_ref().is_some_and(|lang| {
                    let trimmed = lang.trim().to_ascii_lowercase();
                    !trimmed.is_empty() && trimmed != "text" && trimmed != "plaintext"
                })
            })
            .count();

        if code_block_count > 0 && code_blocks_with_syntax == 0 {
            warnings.push(ExtractionWarning::new(
                "CODE_SYNTAX_UNDETECTED",
                ExtractionWarningCategory::Code,
                WarningSeverity::Info,
                "Extracted code blocks do not have an explicitly identified programming language.",
                0.02,
            ));
        }

        // 4. Tables analysis
        let table_count = elements.tables.len();
        let tables_with_headers = elements
            .tables
            .iter()
            .filter(|t| t.has_headers && !t.headers.is_empty())
            .count();

        if table_count > 0 && tables_with_headers < table_count {
            warnings.push(ExtractionWarning::new(
                "TABLE_MISSING_HEADERS",
                ExtractionWarningCategory::Table,
                WarningSeverity::Info,
                "One or more extracted tables lack an explicit header row.",
                0.02,
            ));
        }

        // 5. Links analysis
        let link_count = elements.links.len();
        let internal_link_count = elements
            .links
            .iter()
            .filter(|l| matches!(l.link_type, LinkType::Internal))
            .count();
        let external_link_count = elements
            .links
            .iter()
            .filter(|l| matches!(l.link_type, LinkType::External))
            .count();

        // 6. Metadata scoring
        let (metadata_score, meta_warnings) = evaluate_metadata(cleaned_doc, attribution);
        warnings.extend(meta_warnings);

        // 7. Content scoring
        let (content_score, content_warnings) = evaluate_content(
            word_count,
            content_density,
            raw_html_bytes,
            article,
            documentation,
            self.options.target_word_count,
            self.options.min_content_density,
        );
        warnings.extend(content_warnings);

        // 8. Structural scoring
        let structural_score = evaluate_structure(
            heading_count,
            headings_by_level,
            heading_hierarchy_valid,
            code_block_count,
            code_blocks_with_syntax,
            table_count,
            tables_with_headers,
            link_count,
            documentation.is_some(),
        );

        // 9. Provenance scoring
        let (provenance_score, prov_warnings) = evaluate_provenance(attribution, provenance);
        warnings.extend(prov_warnings);

        // 10. Overall score synthesis & penalty application
        let raw_score = (self.options.metadata_weight * metadata_score)
            + (self.options.content_weight * content_score)
            + (self.options.structural_weight * structural_score)
            + (self.options.provenance_weight * provenance_score);

        let total_penalties: f64 = warnings.iter().map(|w| w.score_penalty).sum();
        let penalized = (raw_score - total_penalties).clamp(0.0, 1.0);
        let overall_score = if word_count == 0 {
            0.0
        } else {
            (penalized * 1000.0).round() / 1000.0
        };

        let has_critical = warnings
            .iter()
            .any(|w| w.severity == WarningSeverity::Critical);

        let completeness_level = if word_count == 0 {
            CompletenessLevel::Empty
        } else if overall_score >= 0.85 && !has_critical {
            CompletenessLevel::Complete
        } else if overall_score >= 0.70 && !has_critical {
            CompletenessLevel::High
        } else if overall_score >= 0.50 {
            CompletenessLevel::Moderate
        } else if overall_score >= 0.25 {
            CompletenessLevel::Partial
        } else if overall_score >= 0.05 {
            CompletenessLevel::Degraded
        } else {
            CompletenessLevel::Empty
        };

        let is_acceptable_for_cdm = overall_score >= self.options.cdm_acceptance_threshold
            && completeness_level != CompletenessLevel::Empty;

        let metrics = ContentMetrics {
            raw_html_bytes,
            cleaned_text_bytes,
            word_count,
            character_count,
            reading_time_minutes,
            paragraph_count,
            heading_count,
            headings_by_level,
            heading_hierarchy_valid,
            code_block_count,
            code_blocks_with_syntax,
            table_count,
            tables_with_headers,
            link_count,
            internal_link_count,
            external_link_count,
            content_density,
            boilerplate_ratio,
        };

        let dimension_scores = DimensionScores {
            metadata: (metadata_score * 1000.0).round() / 1000.0,
            content: (content_score * 1000.0).round() / 1000.0,
            structural: (structural_score * 1000.0).round() / 1000.0,
            provenance: (provenance_score * 1000.0).round() / 1000.0,
        };

        ExtractionQualityReport {
            overall_score,
            completeness_level,
            dimension_scores,
            metrics,
            warnings,
            is_acceptable_for_cdm,
        }
    }
}

/// Evaluates metadata richness and emits diagnostic warnings for missing fields.
fn evaluate_metadata(
    cleaned_doc: &CleanedDocument,
    attribution: &SourceAttribution,
) -> (f64, Vec<ExtractionWarning>) {
    let mut score: f64 = 0.0;
    let mut warnings = Vec::new();

    let meta = &attribution.metadata;
    let doc_meta = &cleaned_doc.metadata;

    // Title (0.25)
    let title = meta.title.as_ref().or(doc_meta.title.as_ref());
    if let Some(t) = title {
        if !t.trim().is_empty() {
            score += 0.25;
        } else {
            warnings.push(ExtractionWarning::new(
                "METADATA_MISSING_TITLE",
                ExtractionWarningCategory::Metadata,
                WarningSeverity::Warning,
                "Webpage title is empty or missing from <head> and OpenGraph.",
                0.08,
            ));
        }
    } else {
        warnings.push(ExtractionWarning::new(
            "METADATA_MISSING_TITLE",
            ExtractionWarningCategory::Metadata,
            WarningSeverity::Warning,
            "Webpage title is empty or missing from <head> and OpenGraph.",
            0.08,
        ));
    }

    // Description (0.20)
    let desc = meta.description.as_ref().or(doc_meta.description.as_ref());
    if let Some(d) = desc {
        if !d.trim().is_empty() {
            score += 0.20;
        } else {
            warnings.push(ExtractionWarning::new(
                "METADATA_MISSING_DESCRIPTION",
                ExtractionWarningCategory::Metadata,
                WarningSeverity::Info,
                "Meta description is missing from <head> and OpenGraph.",
                0.02,
            ));
        }
    } else {
        warnings.push(ExtractionWarning::new(
            "METADATA_MISSING_DESCRIPTION",
            ExtractionWarningCategory::Metadata,
            WarningSeverity::Info,
            "Meta description is missing from <head> and OpenGraph.",
            0.02,
        ));
    }

    // Author (0.15)
    let author = meta.author.as_ref().or(doc_meta.author.as_ref());
    if let Some(a) = author {
        if !a.trim().is_empty() {
            score += 0.15;
        } else {
            warnings.push(ExtractionWarning::new(
                "METADATA_MISSING_AUTHOR",
                ExtractionWarningCategory::Metadata,
                WarningSeverity::Info,
                "Document author or byline is not declared.",
                0.01,
            ));
        }
    } else {
        warnings.push(ExtractionWarning::new(
            "METADATA_MISSING_AUTHOR",
            ExtractionWarningCategory::Metadata,
            WarningSeverity::Info,
            "Document author or byline is not declared.",
            0.01,
        ));
    }

    // Dates (0.15)
    let has_date = meta.published_date.is_some()
        || meta.modified_date.is_some()
        || doc_meta.published_time.is_some();
    if has_date {
        score += 0.15;
    } else {
        warnings.push(ExtractionWarning::new(
            "METADATA_MISSING_DATE",
            ExtractionWarningCategory::Metadata,
            WarningSeverity::Info,
            "No publication or modification date could be discovered in metadata.",
            0.01,
        ));
    }

    // Canonical link (0.15)
    if meta.canonical_link.is_some() || doc_meta.canonical_url.is_some() {
        score += 0.15;
    } else {
        warnings.push(ExtractionWarning::new(
            "METADATA_MISSING_CANONICAL",
            ExtractionWarningCategory::Metadata,
            WarningSeverity::Info,
            "No canonical URL declaration (<link rel=\"canonical\">) discovered.",
            0.02,
        ));
    }

    // Language (0.10)
    let has_lang = meta.language.is_some() || doc_meta.language.is_some();
    if has_lang {
        score += 0.10;
    }

    (score.clamp(0.0, 1.0), warnings)
}

/// Evaluates content volume, word count, density, and article heuristics.
fn evaluate_content(
    word_count: usize,
    density: f64,
    raw_html_bytes: usize,
    article: Option<&ExtractedArticle>,
    documentation: Option<&ExtractedDocumentation>,
    target_words: usize,
    min_density: f64,
) -> (f64, Vec<ExtractionWarning>) {
    let mut score;
    let mut warnings = Vec::new();

    // 1. Word count scoring
    if word_count == 0 {
        score = 0.0;
        warnings.push(ExtractionWarning::new(
            "CONTENT_EMPTY",
            ExtractionWarningCategory::Content,
            WarningSeverity::Critical,
            "Extracted plain text is completely empty.",
            0.35,
        ));
    } else if word_count < 30 {
        score = (word_count as f64 / 30.0) * 0.35;
        warnings.push(ExtractionWarning::new(
            "CONTENT_VERY_SPARSE",
            ExtractionWarningCategory::Content,
            WarningSeverity::Warning,
            format!("Extracted content is very sparse ({word_count} words)."),
            0.10,
        ));
    } else if word_count < 100 {
        score = 0.35 + ((word_count - 30) as f64 / 70.0) * 0.35;
        warnings.push(ExtractionWarning::new(
            "CONTENT_SHORT",
            ExtractionWarningCategory::Content,
            WarningSeverity::Info,
            format!("Extracted content is relatively short ({word_count} words)."),
            0.02,
        ));
    } else if word_count < target_words {
        let span = (target_words - 100) as f64;
        score = 0.70 + ((word_count - 100) as f64 / span) * 0.20;
    } else {
        score =
            0.90 + 0.10 * (1.0 - (-((word_count - target_words) as f64) / 1000.0).exp()).min(1.0);
    }

    // 2. Content density checks
    if raw_html_bytes > 1500 && density < (min_density / 2.0) {
        warnings.push(ExtractionWarning::new(
            "CONTENT_EXTREMELY_LOW_DENSITY",
            ExtractionWarningCategory::Content,
            WarningSeverity::Warning,
            format!(
                "Text-to-HTML content density ({:.1}%) is critically low.",
                density * 100.0
            ),
            0.08,
        ));
    } else if raw_html_bytes > 800 && density < min_density {
        warnings.push(ExtractionWarning::new(
            "CONTENT_LOW_DENSITY",
            ExtractionWarningCategory::Content,
            WarningSeverity::Info,
            format!(
                "Text-to-HTML content density ({:.1}%) is below expected baseline.",
                density * 100.0
            ),
            0.03,
        ));
    }

    // 3. Article / Doc structure bonus
    if let Some(art) = article {
        if !art.is_empty() {
            score = (score * 1.10).min(1.0);
        }
    }
    if let Some(doc) = documentation {
        if !doc.is_empty() {
            score = (score * 1.10).min(1.0);
        }
    }

    (score.clamp(0.0, 1.0), warnings)
}

/// Evaluates heading structure, code blocks, tables, and hyperlinks.
#[allow(clippy::too_many_arguments)]
fn evaluate_structure(
    heading_count: usize,
    headings_by_level: [usize; 6],
    heading_hierarchy_valid: bool,
    code_block_count: usize,
    code_blocks_with_syntax: usize,
    table_count: usize,
    tables_with_headers: usize,
    link_count: usize,
    has_documentation: bool,
) -> f64 {
    // 1. Heading component (0.35)
    let heading_score = if heading_count == 0 {
        0.50
    } else {
        let h1_bonus = if headings_by_level[0] > 0 { 0.60 } else { 0.40 };
        let hierarchy_bonus = if heading_hierarchy_valid { 0.40 } else { 0.10 };
        h1_bonus + hierarchy_bonus
    };

    // 2. Code block component (0.25)
    let code_score = if code_block_count == 0 {
        1.00
    } else {
        0.50 + 0.50 * (code_blocks_with_syntax as f64 / code_block_count as f64)
    };

    // 3. Table component (0.25)
    let table_score = if table_count == 0 {
        1.00
    } else {
        0.60 + 0.40 * (tables_with_headers as f64 / table_count as f64)
    };

    // 4. Link & navigation component (0.15)
    let link_score = if link_count > 0 { 1.00 } else { 0.80 };

    let mut total =
        (0.35 * heading_score) + (0.25 * code_score) + (0.25 * table_score) + (0.15 * link_score);

    if has_documentation {
        total = (total * 1.05).min(1.0);
    }

    total.clamp(0.0, 1.0)
}

/// Evaluates cryptographic hashes, HTTP transport provenance, and runtime degradation.
fn evaluate_provenance(
    attribution: &SourceAttribution,
    provenance: &crate::browser::extractor::DynamicProvenance,
) -> (f64, Vec<ExtractionWarning>) {
    let mut score: f64 = 0.0;
    let mut warnings = Vec::new();

    // Cryptographic hashes (0.35)
    let has_sha256 = !attribution.content_hashes.raw_html_sha256.is_empty();
    let has_blake3 = !attribution.content_hashes.raw_html_blake3.is_empty();
    if has_sha256 && has_blake3 {
        score += 0.35;
    } else if has_sha256 || has_blake3 {
        score += 0.20;
    }

    // HTTP transport status & IP pinning (0.35)
    if let Some(http) = &attribution.http_provenance {
        if http.status_code >= 200 && http.status_code < 300 {
            score += 0.20;
        } else {
            score += 0.10;
        }

        if http.pinned_ip.is_some() {
            score += 0.10;
        }

        if http.etag.is_some() || http.last_modified.is_some() {
            score += 0.05;
        } else {
            warnings.push(ExtractionWarning::new(
                "MISSING_CACHE_VALIDATORS",
                ExtractionWarningCategory::Transport,
                WarningSeverity::Info,
                "HTTP response lacks both ETag and Last-Modified cache validators.",
                0.01,
            ));
        }
    } else if provenance.http_status >= 200 && provenance.http_status < 300 {
        score += 0.30;
    }

    // Canonical resolution (0.20)
    if attribution.canonical_resolution.is_high_confidence() {
        score += 0.20;
    } else {
        score += 0.10;
    }

    // Warnings inspection from attribution and provenance
    for w in &attribution.warnings {
        if w.code == "CANONICAL_URL_MISMATCH" {
            warnings.push(ExtractionWarning::new(
                "CANONICAL_URL_MISMATCH",
                ExtractionWarningCategory::Structure,
                WarningSeverity::Warning,
                &w.message,
                0.05,
            ));
        }
    }

    if !provenance.blocked_subresources.is_empty() {
        warnings.push(ExtractionWarning::new(
            "SUBRESOURCES_BLOCKED_SSRF",
            ExtractionWarningCategory::Security,
            WarningSeverity::Warning,
            format!(
                "{} subresource requests were intercepted and blocked by SSRF policy.",
                provenance.blocked_subresources.len()
            ),
            0.06,
        ));
    }

    if !provenance.page_errors.is_empty() {
        warnings.push(ExtractionWarning::new(
            "PAGE_JAVASCRIPT_ERRORS",
            ExtractionWarningCategory::Browser,
            WarningSeverity::Warning,
            format!(
                "{} unhandled JavaScript page errors were emitted during rendering.",
                provenance.page_errors.len()
            ),
            0.04,
        ));
    }

    if provenance
        .warnings
        .iter()
        .any(|w| w.contains("BROWSER_DEGRADED_TO_STATIC"))
    {
        warnings.push(ExtractionWarning::new(
            "BROWSER_DEGRADED_TO_STATIC",
            ExtractionWarningCategory::Browser,
            WarningSeverity::Warning,
            "Dynamic browser rendering failed; extraction degraded to static HTML.",
            0.08,
        ));
    }

    // Extra +0.10 base for successful extraction
    score = (score + 0.10).min(1.0);

    (score.clamp(0.0, 1.0), warnings)
}

/// Recursively scans parsed HTML DOM to count headings and verify monotonic hierarchy.
fn analyze_headings(html: &str) -> (usize, [usize; 6], bool, Vec<ExtractionWarning>) {
    let mut warnings = Vec::new();
    let mut counts = [0usize; 6];
    let mut sequence = Vec::new();

    if let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) {
        let parser = dom.parser();
        let mut flat_headings = Vec::new();

        fn traverse(handle: tl::NodeHandle, parser: &tl::Parser<'_>, flat: &mut Vec<(u8, String)>) {
            if let Some(tl::Node::Tag(tag)) = handle.get(parser) {
                let name = tag.name().as_utf8_str();
                let name_lower = name.to_ascii_lowercase();
                if let Some(level) = match name_lower.as_str() {
                    "h1" => Some(1u8),
                    "h2" => Some(2u8),
                    "h3" => Some(3u8),
                    "h4" => Some(4u8),
                    "h5" => Some(5u8),
                    "h6" => Some(6u8),
                    _ => None,
                } {
                    let text = tag.inner_text(parser);
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        flat.push((level, trimmed.to_string()));
                    }
                }

                for child in tag.children().top().iter() {
                    traverse(*child, parser, flat);
                }
            }
        }

        for root_child in dom.children() {
            traverse(*root_child, parser, &mut flat_headings);
        }

        for (lvl, _) in &flat_headings {
            counts[(*lvl - 1) as usize] += 1;
            sequence.push(*lvl);
        }
    }

    let total: usize = counts.iter().sum();
    let mut hierarchy_valid = true;

    if total > 0 {
        if counts[0] == 0 {
            warnings.push(ExtractionWarning::new(
                "HEADING_MISSING_H1",
                ExtractionWarningCategory::Structure,
                WarningSeverity::Info,
                "Document contains section headings but lacks a top-level <h1> heading.",
                0.03,
            ));
        }

        for window in sequence.windows(2) {
            let prev = window[0];
            let next = window[1];
            if next > prev + 1 {
                hierarchy_valid = false;
                warnings.push(
                    ExtractionWarning::new(
                        "HEADING_HIERARCHY_GAP",
                        ExtractionWarningCategory::Structure,
                        WarningSeverity::Info,
                        format!(
                            "Heading hierarchy skips a level: jumped from <h{prev}> to <h{next}>.",
                        ),
                        0.03,
                    )
                    .with_context(format!("h{prev} -> h{next}")),
                );
                break;
            }
        }
    }

    (total, counts, hierarchy_valid, warnings)
}

/// Counts non-empty paragraph blocks in plain text.
fn count_paragraphs(text: &str) -> usize {
    let double_nl_count = text
        .split("\n\n")
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .count();
    if double_nl_count > 0 {
        double_nl_count
    } else {
        text.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::browser::extractor::DynamicProvenance;
    use crate::browser::model::{BlockedSubresource, PageError};
    use crate::dom::metadata::DocumentMetadata;
    use crate::elements::model::{
        ExtractedCodeBlock, ExtractedElements, ExtractedLink, ExtractedTable,
    };
    use crate::provenance::{
        CapturedMetadata, ContentHashes, HttpProvenance, ProvenanceCitations, SourceAttribution,
    };
    use crate::ExtractionMethod;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use web_repo_to_ai_url_security::{UrlSecurityPolicy, ValidatedUrl};

    fn sample_cleaned_doc(title: &str, html: &str, text: &str) -> CleanedDocument {
        let meta = DocumentMetadata {
            title: Some(title.to_string()),
            description: Some("Sample test description for document.".to_string()),
            author: Some("Jane Engineer".to_string()),
            published_time: Some("2026-11-17T12:00:00Z".to_string()),
            canonical_url: Some("https://example.com/docs/test".to_string()),
            language: Some("en".to_string()),
            ..Default::default()
        };

        CleanedDocument {
            metadata: meta,
            raw_html_bytes: html.len(),
            cleaned_html: html.to_string(),
            cleaned_text: text.to_string(),
            removed_nodes_count: 5,
            has_semantic_main: true,
        }
    }

    fn sample_attribution(url: &str) -> SourceAttribution {
        let policy = UrlSecurityPolicy::default();
        let validated = ValidatedUrl::parse(url, &policy).unwrap();

        let meta = CapturedMetadata {
            title: Some("Sample Document".to_string()),
            description: Some("Sample test description for document.".to_string()),
            author: Some("Jane Engineer".to_string()),
            published_date: Some("2026-11-17T12:00:00Z".to_string()),
            canonical_link: Some(url.to_string()),
            language: Some("en".to_string()),
            ..Default::default()
        };

        let http = HttpProvenance {
            status_code: 200,
            status_text: "OK".to_string(),
            headers: HashMap::new(),
            content_type: Some("text/html".to_string()),
            content_length: Some(1024),
            etag: Some("\"abc123etag\"".to_string()),
            last_modified: None,
            cache_control: None,
            server: None,
            pinned_ip: Some(IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34))),
            redirect_history: Vec::new(),
        };

        let hashes = ContentHashes::compute(b"<html>sample</html>", "sample text");

        let citations = ProvenanceCitations {
            concise: format!("[Sample]({url})"),
            markdown_callout: format!("> Source: {url}"),
            plaintext: format!("Source: {url}"),
        };

        SourceAttribution {
            canonical_url: url.to_string(),
            initial_url: url.to_string(),
            retrieved_at: Utc::now(),
            load_duration_ms: 45,
            extraction_method: ExtractionMethod::Static,
            content_hashes: hashes,
            http_provenance: Some(http),
            metadata: meta,
            canonical_resolution: crate::dedup::model::CanonicalResolution {
                canonical_url: validated,
                source: crate::dedup::model::CanonicalSource::HtmlLinkRelCanonical,
                raw_candidate: Some(url.to_string()),
                is_cross_domain: false,
                confidence: 100,
                warnings: Vec::new(),
            },
            browser_diagnostics: None,
            citations,
            warnings: Vec::new(),
        }
    }

    fn sample_provenance(url: &str) -> DynamicProvenance {
        DynamicProvenance {
            method: ExtractionMethod::Static,
            http_status: 200,
            load_time_ms: 45,
            final_url: url.to_string(),
            redirects: Vec::new(),
            console_messages: Vec::new(),
            page_errors: Vec::new(),
            blocked_subresources: Vec::new(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn test_rich_article_completeness_scoring() {
        let html = r#"
            <main>
                <h1>Comprehensive Guide to Rust Async</h1>
                <p>Rust provides powerful asynchronous programming facilities via Tokio and futures.</p>
                <h2>Event Loops and Tasks</h2>
                <p>The event loop coordinates asynchronous tasks via cooperative multitasking.</p>
                <h3>Worker Threads</h3>
                <p>Worker threads process tasks from work-stealing queues with minimal lock contention.</p>
            </main>
        "#;
        let text = "Rust provides powerful asynchronous programming facilities via Tokio and futures. The event loop coordinates asynchronous tasks via cooperative multitasking. Worker threads process tasks from work-stealing queues with minimal lock contention. ".repeat(10);

        let cleaned = sample_cleaned_doc("Comprehensive Guide to Rust Async", html, &text);
        let attribution = sample_attribution("https://example.com/docs/test");
        let provenance = sample_provenance("https://example.com/docs/test");

        let mut elements = ExtractedElements::default();
        elements.code_blocks.push(ExtractedCodeBlock {
            code: "tokio::spawn(async move { ... });".to_string(),
            language: Some("rust".to_string()),
            is_inline: false,
            line_count: 1,
            has_line_numbers_stripped: false,
        });
        elements.links.push(ExtractedLink {
            url: "https://example.com/docs/tokio".to_string(),
            original_href: "/docs/tokio".to_string(),
            text: "Tokio Docs".to_string(),
            title: None,
            rel: Vec::new(),
            link_type: LinkType::Internal,
        });

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report.overall_score >= 0.85);
        assert_eq!(report.completeness_level, CompletenessLevel::Complete);
        assert!(report.is_acceptable_for_cdm);
        assert!(!report.has_critical_warnings());
        assert_eq!(report.metrics.heading_count, 3);
        assert!(report.metrics.heading_hierarchy_valid);
        assert_eq!(report.metrics.code_blocks_with_syntax, 1);
        assert!(report.summary().contains("Complete"));
    }

    #[test]
    fn test_empty_content_yields_critical_warning_and_empty_grade() {
        let html = "<html><body></body></html>";
        let cleaned = CleanedDocument {
            metadata: DocumentMetadata::default(),
            raw_html_bytes: html.len(),
            cleaned_html: html.to_string(),
            cleaned_text: String::new(),
            removed_nodes_count: 0,
            has_semantic_main: false,
        };

        let attribution = sample_attribution("https://example.com/empty");
        let provenance = sample_provenance("https://example.com/empty");
        let elements = ExtractedElements::default();

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert_eq!(report.completeness_level, CompletenessLevel::Empty);
        assert!(!report.is_acceptable_for_cdm);
        assert!(report.has_critical_warnings());
        assert!(report.warning_codes().contains(&"CONTENT_EMPTY"));
    }

    #[test]
    fn test_sparse_content_yields_partial_or_degraded_grade() {
        let html = "<main><p>Short notice.</p></main>";
        let cleaned = sample_cleaned_doc("Notice", html, "Short notice.");
        let attribution = sample_attribution("https://example.com/notice");
        let provenance = sample_provenance("https://example.com/notice");
        let elements = ExtractedElements::default();

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report.overall_score < 0.70);
        assert!(report.warning_codes().contains(&"CONTENT_VERY_SPARSE"));
    }

    #[test]
    fn test_missing_metadata_warnings_and_penalties() {
        let html =
            "<main><p>Valid content with missing metadata fields in document head.</p></main>";
        let mut cleaned =
            sample_cleaned_doc("", html, "Valid content with missing metadata fields.");
        cleaned.metadata = DocumentMetadata::default();

        let mut attribution = sample_attribution("https://example.com/nometa");
        attribution.metadata = CapturedMetadata::default();

        let provenance = sample_provenance("https://example.com/nometa");
        let elements = ExtractedElements::default();

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report.warning_codes().contains(&"METADATA_MISSING_TITLE"));
        assert!(report
            .warning_codes()
            .contains(&"METADATA_MISSING_DESCRIPTION"));
        assert!(report.warning_codes().contains(&"METADATA_MISSING_AUTHOR"));
        assert!(report.warning_codes().contains(&"METADATA_MISSING_DATE"));
        assert!(report.dimension_scores.metadata < 0.20);
    }

    #[test]
    fn test_heading_hierarchy_gap_detected() {
        // Skips h2 (jumps from h1 to h3)
        let html = r#"
            <div>
                <h1>Title</h1>
                <h3>Deep Subheading Without H2</h3>
            </div>
        "#;
        let text = "Title Deep Subheading Without H2";
        let cleaned = sample_cleaned_doc("Gap Test", html, text);
        let attribution = sample_attribution("https://example.com/gap");
        let provenance = sample_provenance("https://example.com/gap");
        let elements = ExtractedElements::default();

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(!report.metrics.heading_hierarchy_valid);
        assert!(report.warning_codes().contains(&"HEADING_HIERARCHY_GAP"));
    }

    #[test]
    fn test_code_syntax_and_table_header_detection() {
        let html = "<div>content</div>";
        let cleaned = sample_cleaned_doc("Elements Test", html, "Sample content");
        let attribution = sample_attribution("https://example.com/elements");
        let provenance = sample_provenance("https://example.com/elements");

        let mut elements = ExtractedElements::default();
        // Code block without syntax
        elements.code_blocks.push(ExtractedCodeBlock {
            code: "let x = 1;".to_string(),
            language: None,
            is_inline: false,
            line_count: 1,
            has_line_numbers_stripped: false,
        });
        // Table without headers
        elements.tables.push(ExtractedTable {
            caption: None,
            headers: Vec::new(),
            rows: vec![vec!["A".to_string(), "B".to_string()]],
            column_alignments: Vec::new(),
            has_headers: false,
            num_rows: 1,
            num_cols: 2,
        });

        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report.warning_codes().contains(&"CODE_SYNTAX_UNDETECTED"));
        assert!(report.warning_codes().contains(&"TABLE_MISSING_HEADERS"));
    }

    #[test]
    fn test_subresources_blocked_and_page_errors_penalties() {
        let html = "<main><p>Rendering with errors</p></main>";
        let cleaned = sample_cleaned_doc("Diagnostics", html, "Rendering with errors");
        let attribution = sample_attribution("https://example.com/diagnostics");
        let mut provenance = sample_provenance("https://example.com/diagnostics");

        provenance.blocked_subresources.push(BlockedSubresource {
            url: "http://169.254.169.254/latest/meta-data".to_string(),
            reason: "private_ip".to_string(),
        });
        provenance.page_errors.push(PageError {
            message: "Uncaught TypeError: Cannot read property of undefined".to_string(),
            stack: None,
        });

        let elements = ExtractedElements::default();
        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report
            .warning_codes()
            .contains(&"SUBRESOURCES_BLOCKED_SSRF"));
        assert!(report.warning_codes().contains(&"PAGE_JAVASCRIPT_ERRORS"));
    }

    #[test]
    fn test_browser_degraded_to_static_penalty() {
        let html = "<main><p>Degraded static fallback content</p></main>";
        let cleaned = sample_cleaned_doc("Degraded", html, "Degraded static fallback content");
        let attribution = sample_attribution("https://example.com/degraded");
        let mut provenance = sample_provenance("https://example.com/degraded");
        provenance
            .warnings
            .push("BROWSER_DEGRADED_TO_STATIC".to_string());

        let elements = ExtractedElements::default();
        let scorer = QualityScorer::new();
        let report = scorer.assess_components(
            &cleaned,
            None,
            None,
            &elements,
            &attribution,
            &provenance,
            html,
        );

        assert!(report
            .warning_codes()
            .contains(&"BROWSER_DEGRADED_TO_STATIC"));
    }
}
