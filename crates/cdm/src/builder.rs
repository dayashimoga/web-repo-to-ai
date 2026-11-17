//! # Fluent Builders for CDM Documents and Sections
//!
//! Provides ergonomic, type-safe builder patterns for constructing valid
//! [`CdmDocument`] and [`CdmSection`] structures with automatic cryptographic
//! hash generation and default baseline metrics.

use crate::completeness::CompletenessReport;
use crate::hierarchy::HierarchyNode;
use crate::metadata::{Citation, DocumentMetadata, ExclusionRecord};
use crate::model::{
    CdmDocument, ExtractionStatus, ExtractionWarning, RevisionInfo, SourceIdentity, SourceType,
};
use crate::relationships::Relationship;
use crate::section::{ByteRange, CdmSection, LineRange, SectionKind, SectionTokenMetrics};
use chrono::{DateTime, Utc};
use std::collections::BTreeMap;
use web_repo_to_ai_core::DocumentId;

/// Fluent builder for constructing a [`CdmDocument`].
#[derive(Debug, Clone)]
pub struct CdmDocumentBuilder {
    schema_version: String,
    document_id: DocumentId,
    source_identity: SourceIdentity,
    source_type: SourceType,
    canonical_url: String,
    retrieved_at: DateTime<Utc>,
    revision: RevisionInfo,
    content_type: String,
    metadata: DocumentMetadata,
    hierarchy: HierarchyNode,
    sections: Vec<CdmSection>,
    relationships: Vec<Relationship>,
    citations: Vec<Citation>,
    extraction_status: ExtractionStatus,
    warnings: Vec<ExtractionWarning>,
    exclusions: Vec<ExclusionRecord>,
    completeness: Option<CompletenessReport>,
}

impl CdmDocumentBuilder {
    /// Initializes a new builder with required coordinate fields.
    #[must_use]
    pub fn new(
        source_identity: SourceIdentity,
        source_type: SourceType,
        canonical_url: impl Into<String>,
    ) -> Self {
        let canonical_str = canonical_url.into();
        Self {
            schema_version: crate::SCHEMA_VERSION.to_string(),
            document_id: DocumentId::new(),
            source_identity,
            source_type,
            canonical_url: canonical_str.clone(),
            retrieved_at: Utc::now(),
            revision: RevisionInfo::default(),
            content_type: "text/plain".to_string(),
            metadata: DocumentMetadata::default(),
            hierarchy: HierarchyNode::root(canonical_str),
            sections: Vec::new(),
            relationships: Vec::new(),
            citations: Vec::new(),
            extraction_status: ExtractionStatus::Completed,
            warnings: Vec::new(),
            exclusions: Vec::new(),
            completeness: None,
        }
    }

    /// Sets the specific document ID (defaults to a newly generated UUID v7).
    #[must_use]
    pub fn document_id(mut self, document_id: DocumentId) -> Self {
        self.document_id = document_id;
        self
    }

    /// Overrides the schema version string (defaults to `1.0.0`).
    #[must_use]
    pub fn schema_version(mut self, schema_version: impl Into<String>) -> Self {
        self.schema_version = schema_version.into();
        self
    }

    /// Sets the acquisition timestamp.
    #[must_use]
    pub const fn retrieved_at(mut self, retrieved_at: DateTime<Utc>) -> Self {
        self.retrieved_at = retrieved_at;
        self
    }

    /// Sets revision and cache validation headers.
    #[must_use]
    pub fn revision(mut self, revision: RevisionInfo) -> Self {
        self.revision = revision;
        self
    }

    /// Sets the source MIME content type.
    #[must_use]
    pub fn content_type(mut self, content_type: impl Into<String>) -> Self {
        self.content_type = content_type.into();
        self
    }

    /// Sets holistic descriptive metadata.
    #[must_use]
    pub fn metadata(mut self, metadata: DocumentMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Sets the document title within metadata.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.metadata.title = Some(title.into());
        self
    }

    /// Sets the root structural hierarchy node.
    #[must_use]
    pub fn hierarchy(mut self, hierarchy: HierarchyNode) -> Self {
        self.hierarchy = hierarchy;
        self
    }

    /// Adds a content section.
    #[must_use]
    pub fn add_section(mut self, section: CdmSection) -> Self {
        self.sections.push(section);
        self
    }

    /// Extends the document with multiple content sections.
    #[must_use]
    pub fn extend_sections(mut self, sections: impl IntoIterator<Item = CdmSection>) -> Self {
        self.sections.extend(sections);
        self
    }

    /// Adds a relationship.
    #[must_use]
    pub fn add_relationship(mut self, relationship: Relationship) -> Self {
        self.relationships.push(relationship);
        self
    }

    /// Adds a citation.
    #[must_use]
    pub fn add_citation(mut self, citation: Citation) -> Self {
        self.citations.push(citation);
        self
    }

    /// Adds an extraction warning.
    #[must_use]
    pub fn add_warning(mut self, warning: ExtractionWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Adds an exclusion record.
    #[must_use]
    pub fn add_exclusion(mut self, exclusion: ExclusionRecord) -> Self {
        self.exclusions.push(exclusion);
        self
    }

    /// Sets the extraction status.
    #[must_use]
    pub const fn extraction_status(mut self, extraction_status: ExtractionStatus) -> Self {
        self.extraction_status = extraction_status;
        self
    }

    /// Explicitly sets the completeness report.
    #[must_use]
    pub const fn completeness(mut self, completeness: CompletenessReport) -> Self {
        self.completeness = Some(completeness);
        self
    }

    /// Consumes the builder and returns a verified [`CdmDocument`].
    ///
    /// If no explicit completeness report was provided, automatically constructs
    /// one derived from the sections and exclusions counts.
    #[must_use]
    pub fn build(self) -> CdmDocument {
        let completeness = self.completeness.unwrap_or_else(|| {
            let extracted = self.sections.len() as u64;
            let excluded = self.exclusions.len() as u64;
            let discovered = extracted + excluded;
            CompletenessReport::new(discovered, extracted, extracted, excluded, 0, 0)
        });

        CdmDocument {
            schema_version: self.schema_version,
            document_id: self.document_id,
            source_identity: self.source_identity,
            source_type: self.source_type,
            canonical_url: self.canonical_url,
            retrieved_at: self.retrieved_at,
            revision: self.revision,
            content_type: self.content_type,
            metadata: self.metadata,
            hierarchy: self.hierarchy,
            sections: self.sections,
            relationships: self.relationships,
            citations: self.citations,
            extraction_status: self.extraction_status,
            warnings: self.warnings,
            exclusions: self.exclusions,
            completeness,
        }
    }
}

/// Fluent builder for constructing a [`CdmSection`].
#[derive(Debug, Clone)]
pub struct CdmSectionBuilder {
    section_id: String,
    parent_id: Option<String>,
    title: Option<String>,
    kind: SectionKind,
    original_path: Option<String>,
    source_url: String,
    line_range: Option<LineRange>,
    byte_range: Option<ByteRange>,
    language: Option<String>,
    content_hash: Option<String>,
    body: String,
    token_metrics: Option<SectionTokenMetrics>,
    attributes: BTreeMap<String, String>,
}

impl CdmSectionBuilder {
    /// Initializes a section builder with required identifiers and content.
    #[must_use]
    pub fn new(
        section_id: impl Into<String>,
        source_url: impl Into<String>,
        kind: SectionKind,
        body: impl Into<String>,
    ) -> Self {
        Self {
            section_id: section_id.into(),
            parent_id: None,
            title: None,
            kind,
            original_path: None,
            source_url: source_url.into(),
            line_range: None,
            byte_range: None,
            language: None,
            content_hash: None,
            body: body.into(),
            token_metrics: None,
            attributes: BTreeMap::new(),
        }
    }

    /// Sets the title or heading.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Sets the parent section identifier.
    #[must_use]
    pub fn parent_id(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    /// Sets the source relative path.
    #[must_use]
    pub fn original_path(mut self, original_path: impl Into<String>) -> Self {
        self.original_path = Some(original_path.into());
        self
    }

    /// Sets the source line range span.
    #[must_use]
    pub const fn line_range(mut self, start_line: u32, end_line: u32) -> Self {
        self.line_range = Some(LineRange::new(start_line, end_line));
        self
    }

    /// Sets the byte offset boundaries.
    #[must_use]
    pub const fn byte_range(mut self, start_byte: u64, end_byte: u64) -> Self {
        self.byte_range = Some(ByteRange::new(start_byte, end_byte));
        self
    }

    /// Sets the language identifier.
    #[must_use]
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Explicitly sets the content hash (otherwise computed automatically via BLAKE3 on build).
    #[must_use]
    pub fn content_hash(mut self, content_hash: impl Into<String>) -> Self {
        self.content_hash = Some(content_hash.into());
        self
    }

    /// Sets the quantitative token metrics (otherwise computed via baseline heuristic on build).
    #[must_use]
    pub fn token_metrics(mut self, token_metrics: SectionTokenMetrics) -> Self {
        self.token_metrics = Some(token_metrics);
        self
    }

    /// Inserts an attribute key-value pair.
    #[must_use]
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Consumes the builder and returns a fully initialized [`CdmSection`].
    #[must_use]
    pub fn build(self) -> CdmSection {
        let content_hash = self
            .content_hash
            .unwrap_or_else(|| CdmSection::compute_hash(&self.body));
        let token_metrics = self
            .token_metrics
            .or_else(|| Some(SectionTokenMetrics::compute_baseline(&self.body)));

        CdmSection {
            section_id: self.section_id,
            parent_id: self.parent_id,
            title: self.title,
            kind: self.kind,
            original_path: self.original_path,
            source_url: self.source_url,
            line_range: self.line_range,
            byte_range: self.byte_range,
            language: self.language,
            content_hash,
            body: self.body,
            token_metrics,
            attributes: self.attributes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_and_section_builders() {
        let identity = SourceIdentity::new("https://example.com/rust", "https://example.com/rust");
        let section = CdmSectionBuilder::new(
            "sec-01",
            "https://example.com/rust#code",
            SectionKind::CodeSnippet,
            "pub fn hello() -> &'static str { \"hello\" }",
        )
        .title("Greeting function")
        .language("rust")
        .line_range(1, 1)
        .build();

        assert!(section.verify_hash());
        assert_eq!(section.language.as_deref(), Some("rust"));

        let doc = CdmDocumentBuilder::new(
            identity,
            SourceType::WebpageStatic,
            "https://example.com/rust",
        )
        .content_type("text/html")
        .add_section(section)
        .build();

        assert_eq!(doc.schema_version, crate::SCHEMA_VERSION);
        assert_eq!(doc.sections.len(), 1);
        assert!(doc.completeness.is_complete());
        assert_eq!(doc.total_word_count(), 9);
        assert!(doc.total_estimated_tokens() > 0);
        assert!(doc.verify_section_hashes());
    }
}
