//! # Schema and Document Diffing Utilities
//!
//! Introspects differences between two CDM document instances or revisions,
//! identifying schema version shifts, added/removed/modified sections,
//! hierarchy transformations, and completeness changes.

use crate::model::CdmDocument;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Structural and content divergence detected between two sections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionDiff {
    /// Section identifier.
    pub section_id: String,
    /// Whether section body text changed.
    pub body_changed: bool,
    /// Whether cryptographic content hash changed.
    pub hash_changed: bool,
    /// Whether source line range shifted.
    pub line_range_changed: bool,
    /// Whether byte range shifted.
    pub byte_range_changed: bool,
    /// Whether title or heading changed.
    pub title_changed: bool,
}

impl SectionDiff {
    /// Returns true if any content or metadata difference was detected.
    #[must_use]
    pub const fn has_changes(&self) -> bool {
        self.body_changed
            || self.hash_changed
            || self.line_range_changed
            || self.byte_range_changed
            || self.title_changed
    }
}

/// Comprehensive diff report between two CDM document instances.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDiff {
    /// Schema specification version change `(from_version, to_version)`, if versions differ.
    pub schema_version_change: Option<(String, String)>,
    /// Section IDs added in the newer document.
    pub sections_added: Vec<String>,
    /// Section IDs removed from the original document.
    pub sections_removed: Vec<String>,
    /// Detailed modifications across common sections.
    pub sections_modified: Vec<SectionDiff>,
    /// Whether top-level metadata (title, license, languages, etc.) was altered.
    pub metadata_changed: bool,
    /// Whether structural hierarchy outline or tree was modified.
    pub hierarchy_changed: bool,
    /// Whether quantitative completeness figures shifted.
    pub completeness_changed: bool,
    /// Whether status of extraction changed.
    pub status_changed: bool,
}

impl SchemaDiff {
    /// Computes the diff from original document `a` to updated document `b`.
    #[must_use]
    pub fn between(a: &CdmDocument, b: &CdmDocument) -> Self {
        let mut diff = Self::default();

        // 1. Version check
        if a.schema_version != b.schema_version {
            diff.schema_version_change = Some((a.schema_version.clone(), b.schema_version.clone()));
        }

        // 2. Sections diffing
        let a_map: HashMap<&str, _> = a
            .sections
            .iter()
            .map(|s| (s.section_id.as_str(), s))
            .collect();
        let b_map: HashMap<&str, _> = b
            .sections
            .iter()
            .map(|s| (s.section_id.as_str(), s))
            .collect();

        let a_keys: HashSet<&str> = a_map.keys().copied().collect();
        let b_keys: HashSet<&str> = b_map.keys().copied().collect();

        for &id in b_keys.difference(&a_keys) {
            diff.sections_added.push(id.to_string());
        }
        diff.sections_added.sort();

        for &id in a_keys.difference(&b_keys) {
            diff.sections_removed.push(id.to_string());
        }
        diff.sections_removed.sort();

        for &id in a_keys.intersection(&b_keys) {
            let sec_a = a_map[id];
            let sec_b = b_map[id];

            let sec_diff = SectionDiff {
                section_id: id.to_string(),
                body_changed: sec_a.body != sec_b.body,
                hash_changed: sec_a.content_hash != sec_b.content_hash,
                line_range_changed: sec_a.line_range != sec_b.line_range,
                byte_range_changed: sec_a.byte_range != sec_b.byte_range,
                title_changed: sec_a.title != sec_b.title,
            };

            if sec_diff.has_changes() {
                diff.sections_modified.push(sec_diff);
            }
        }
        diff.sections_modified
            .sort_by(|x, y| x.section_id.cmp(&y.section_id));

        // 3. Metadata check
        diff.metadata_changed = a.metadata != b.metadata;

        // 4. Hierarchy check
        diff.hierarchy_changed = a.hierarchy != b.hierarchy;

        // 5. Completeness check
        diff.completeness_changed = a.completeness != b.completeness;

        // 6. Status check
        diff.status_changed = a.extraction_status != b.extraction_status;

        diff
    }

    /// Returns true if both documents are structurally and semantically identical.
    #[must_use]
    pub fn is_identical(&self) -> bool {
        self.schema_version_change.is_none()
            && self.sections_added.is_empty()
            && self.sections_removed.is_empty()
            && self.sections_modified.is_empty()
            && !self.metadata_changed
            && !self.hierarchy_changed
            && !self.completeness_changed
            && !self.status_changed
    }

    /// Returns true if any section was added, removed, or modified.
    #[must_use]
    pub fn has_content_changes(&self) -> bool {
        !self.sections_added.is_empty()
            || !self.sections_removed.is_empty()
            || !self.sections_modified.is_empty()
    }

    /// Formats a human-readable summary of the detected diffs.
    #[must_use]
    pub fn summary(&self) -> String {
        if self.is_identical() {
            return "Documents are identical (0 diffs)".to_string();
        }

        let mut out = String::new();
        if let Some((ref v_a, ref v_b)) = self.schema_version_change {
            out.push_str(&format!("Schema version changed: {v_a} -> {v_b}\n"));
        }
        if !self.sections_added.is_empty() {
            out.push_str(&format!(
                "Sections added ({}): {:?}\n",
                self.sections_added.len(),
                self.sections_added
            ));
        }
        if !self.sections_removed.is_empty() {
            out.push_str(&format!(
                "Sections removed ({}): {:?}\n",
                self.sections_removed.len(),
                self.sections_removed
            ));
        }
        if !self.sections_modified.is_empty() {
            out.push_str(&format!(
                "Sections modified ({}):\n",
                self.sections_modified.len()
            ));
            for s in &self.sections_modified {
                out.push_str(&format!(
                    "  - [{}] body:{}, hash:{}, lines:{}, bytes:{}\n",
                    s.section_id,
                    s.body_changed,
                    s.hash_changed,
                    s.line_range_changed,
                    s.byte_range_changed
                ));
            }
        }
        if self.hierarchy_changed {
            out.push_str("Hierarchy tree changed\n");
        }
        if self.metadata_changed {
            out.push_str("Document metadata changed\n");
        }
        if self.completeness_changed {
            out.push_str("Completeness report changed\n");
        }
        if self.status_changed {
            out.push_str("Extraction status changed\n");
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{CdmDocumentBuilder, CdmSectionBuilder};
    use crate::completeness::CompletenessReport;
    use crate::model::{SourceIdentity, SourceType};
    use crate::section::{CdmSection, SectionKind};

    fn make_test_doc() -> CdmDocument {
        let identity = SourceIdentity::new("https://example.com", "https://example.com");
        let sec = CdmSectionBuilder::new(
            "sec-1",
            "https://example.com#1",
            SectionKind::Prose,
            "Initial body text\n",
        )
        .build();

        CdmDocumentBuilder::new(identity, SourceType::WebpageStatic, "https://example.com")
            .title("Title 1")
            .add_section(sec)
            .completeness(CompletenessReport::single_item_success())
            .build()
    }

    #[test]
    fn test_identical_documents_produce_empty_diff() {
        let doc1 = make_test_doc();
        let doc2 = make_test_doc();
        let diff = SchemaDiff::between(&doc1, &doc2);

        assert!(diff.is_identical());
        assert!(!diff.has_content_changes());
        assert!(diff.summary().contains("identical"));
    }

    #[test]
    fn test_section_modifications_detected() {
        let doc1 = make_test_doc();
        let mut doc2 = make_test_doc();

        // Modify section body
        doc2.sections[0].body = "Updated body text\n".to_string();
        doc2.sections[0].content_hash = CdmSection::compute_hash(&doc2.sections[0].body);

        let diff = SchemaDiff::between(&doc1, &doc2);
        assert!(!diff.is_identical());
        assert!(diff.has_content_changes());
        assert_eq!(diff.sections_modified.len(), 1);
        assert!(diff.sections_modified[0].body_changed);
        assert!(diff.sections_modified[0].hash_changed);
    }

    #[test]
    fn test_section_added_and_removed_detected() {
        let doc1 = make_test_doc();
        let mut doc2 = make_test_doc();

        let sec2 = CdmSectionBuilder::new(
            "sec-2",
            "https://example.com#2",
            SectionKind::Prose,
            "Second section body\n",
        )
        .build();
        doc2.sections.push(sec2);
        doc2.sections.remove(0); // Remove sec-1

        let diff = SchemaDiff::between(&doc1, &doc2);
        assert_eq!(diff.sections_added, vec!["sec-2"]);
        assert_eq!(diff.sections_removed, vec!["sec-1"]);
    }
}
