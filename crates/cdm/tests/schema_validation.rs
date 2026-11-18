//! # CDM Schema Integration & Invariant Tests
//!
//! Integration test suite validating the Canonical Context Document Model (CDM v1.0)
//! schema rules, cryptographic hash verification, tree hierarchy algorithms,
//! and completeness invariants.

use web_repo_to_ai_cdm::{
    ByteRange, CdmDocumentBuilder, CdmSectionBuilder, Citation, CompletenessReport,
    ExclusionReason, ExtractionStatus, ExtractionWarning, HierarchyKind, HierarchyNode, LineRange,
    RelationshipKind, SectionKind, SectionTokenMetrics, SourceIdentity, SourceType,
    WarningSeverity,
};
use web_repo_to_ai_core::DocumentId;

#[test]
fn test_schema_version_consistency() {
    assert_eq!(web_repo_to_ai_cdm::SCHEMA_VERSION, "1.0.0");
    assert_eq!(web_repo_to_ai_cdm::schema_version(), "1.0.0");

    let doc = CdmDocumentBuilder::new(
        SourceIdentity::new("https://example.com", "https://example.com"),
        SourceType::WebpageStatic,
        "https://example.com",
    )
    .build();

    assert_eq!(doc.schema_version, "1.0.0");
}

#[test]
fn test_section_cryptographic_hash_tamper_detection() {
    let original_body = "def calculate_metric(x, y):\n    return x * y + 42\n";
    let mut section = CdmSectionBuilder::new(
        "sec-calc",
        "https://example.com/calc.py",
        SectionKind::CodeFile,
        original_body,
    )
    .language("python")
    .build();

    assert!(section.verify_hash());
    assert_eq!(
        section.content_hash,
        blake3::hash(original_body.as_bytes()).to_hex().to_string()
    );

    // Tamper with section body
    section.body = "def calculate_metric(x, y):\n    return x * y + 9999\n".to_string();
    assert!(
        !section.verify_hash(),
        "Tampered section body must fail hash verification"
    );
}

#[test]
fn test_completeness_invariant_rigorous_matrix() {
    // 1. Fully successful repository
    let c1 = CompletenessReport::new(100, 85, 85, 10, 0, 5);
    assert!(c1.formula_valid());
    assert!(c1.is_complete());
    assert!((c1.extraction_ratio() - 0.85).abs() < 1e-6);

    // 2. Repository with failed files
    let c2 = CompletenessReport::new(100, 85, 80, 10, 5, 5);
    assert!(c2.formula_valid());
    assert!(
        !c2.is_complete(),
        "failed_count > 0 must fail is_complete()"
    );
    assert!((c2.extraction_ratio() - 0.80).abs() < 1e-6);

    // 3. Repository with unextracted eligible files (gap between eligible and extracted)
    let c3 = CompletenessReport::new(100, 85, 75, 15, 0, 10);
    assert!(c3.formula_valid());
    assert!(
        !c3.is_complete(),
        "extracted_count != eligible_count must fail is_complete()"
    );

    // 4. Broken formula (math doesn't add up to discovered)
    let c4 = CompletenessReport::new(100, 85, 85, 5, 0, 5);
    assert!(!c4.formula_valid(), "85 + 5 + 0 + 5 = 95 != 100");
    assert!(!c4.is_complete());

    // 5. Zero discovered items edge case
    let c5 = CompletenessReport::new(0, 0, 0, 0, 0, 0);
    assert!(c5.formula_valid());
    assert!(c5.is_complete());
    assert_eq!(c5.extraction_ratio(), 0.0);
}

#[test]
fn test_deep_hierarchy_traversal_and_queries() {
    let mut root = HierarchyNode::root("project-monorepo");

    // crates directory
    let mut crates_dir = HierarchyNode::directory("dir-crates", "crates", "crates");

    // crates/core
    let mut core_dir = HierarchyNode::directory("dir-core", "core", "crates/core");
    core_dir.add_child(HierarchyNode::file(
        "file-core-lib",
        "lib.rs",
        "crates/core/src/lib.rs",
        Some("sec-core-lib".to_string()),
        Some(4096),
        Some(150),
    ));
    crates_dir.add_child(core_dir);

    // crates/cdm
    let mut cdm_dir = HierarchyNode::directory("dir-cdm", "cdm", "crates/cdm");
    cdm_dir.add_child(HierarchyNode::file(
        "file-cdm-lib",
        "lib.rs",
        "crates/cdm/src/lib.rs",
        Some("sec-cdm-lib".to_string()),
        Some(8192),
        Some(300),
    ));
    crates_dir.add_child(cdm_dir);

    // README at root
    root.add_child(crates_dir);
    root.add_child(HierarchyNode::file(
        "file-root-readme",
        "README.md",
        "README.md",
        Some("sec-readme".to_string()),
        Some(2048),
        Some(80),
    ));

    assert_eq!(root.total_nodes(), 7);
    assert_eq!(root.total_files(), 3);

    // Find node by path
    let found = root.find_node_by_path("crates/cdm/src/lib.rs");
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "lib.rs");
    assert_eq!(found.unwrap().kind, HierarchyKind::File);
    assert_eq!(found.unwrap().byte_size, Some(8192));

    // Flattening
    let flattened = root.flatten();
    assert_eq!(flattened.len(), 7);
    let paths: Vec<&str> = flattened.iter().map(|n| n.path.as_str()).collect();
    assert!(paths.contains(&"crates/core/src/lib.rs"));
    assert!(paths.contains(&"crates/cdm/src/lib.rs"));
    assert!(paths.contains(&"README.md"));
}

#[test]
fn test_complex_multi_section_document_metrics() {
    let identity = SourceIdentity {
        raw_input: "https://api.example.com/v1".to_string(),
        canonical_url: "https://api.example.com/v1".to_string(),
        domain: Some("api.example.com".to_string()),
        repo_owner: None,
        repo_name: None,
        git_ref: None,
        subpath: None,
    };

    let sec1 = CdmSectionBuilder::new(
        "sec-1",
        "https://api.example.com/v1#intro",
        SectionKind::Prose,
        "Introduction to the REST API specifications and endpoints.\n",
    )
    .title("Introduction")
    .build();

    let sec2 = CdmSectionBuilder::new(
        "sec-2",
        "https://api.example.com/v1#code",
        SectionKind::CodeSnippet,
        "curl -X GET https://api.example.com/v1/health\n",
    )
    .language("bash")
    .build();

    let sec3 = CdmSectionBuilder::new(
        "sec-3",
        "https://api.example.com/v1#table",
        SectionKind::Table,
        "| Endpoint | Method | Status |\n|:---|:---:|---:|\n| /health | GET | 200 |\n",
    )
    .title("Endpoints Table")
    .build();

    let sec4 = CdmSectionBuilder::new(
        "sec-4",
        "https://api.example.com/v1#warning",
        SectionKind::Admonition,
        "Deprecated: v1 API will sunset on 2027-01-01.\n",
    )
    .attribute("admonition_type", "warning")
    .build();

    let doc = CdmDocumentBuilder::new(
        identity,
        SourceType::WebpageDynamic,
        "https://api.example.com/v1",
    )
    .document_id(DocumentId::new())
    .add_section(sec1)
    .add_section(sec2)
    .add_section(sec3)
    .add_section(sec4)
    .add_warning(ExtractionWarning::new(
        "API_DEPRECATED",
        "lifecycle",
        "Source API is marked deprecated",
        WarningSeverity::Warning,
    ))
    .add_citation(Citation::new(
        "cit-1",
        "https://api.example.com/v1",
        Some("API Documentation".to_string()),
        None,
    ))
    .build();

    assert_eq!(doc.sections.len(), 4);
    assert_eq!(doc.warnings.len(), 1);
    assert_eq!(doc.warnings[0].severity, WarningSeverity::Warning);
    assert_eq!(doc.citations.len(), 1);
    assert!(doc.verify_section_hashes());
    assert!(doc.total_char_count() > 0);
    assert!(doc.total_word_count() > 0);
    assert!(doc.total_estimated_tokens() > 0);
}

#[test]
fn test_line_and_byte_range_arithmetic() {
    let lr = LineRange::new(10, 25);
    assert_eq!(lr.start_line, 10);
    assert_eq!(lr.end_line, 25);
    assert_eq!(lr.line_count(), 16);

    // Inverted range guard
    let inverted_lr = LineRange::new(50, 20);
    assert_eq!(inverted_lr.line_count(), 0);

    let br = ByteRange::new(100, 250);
    assert_eq!(br.start_byte, 100);
    assert_eq!(br.end_byte, 250);
    assert_eq!(br.byte_length(), 150);

    // Inverted byte range guard
    let inverted_br = ByteRange::new(300, 100);
    assert_eq!(inverted_br.byte_length(), 0);
}

#[test]
fn test_section_token_metrics_edge_cases() {
    let empty_metrics = SectionTokenMetrics::compute_baseline("");
    assert_eq!(empty_metrics.char_count, 0);
    assert_eq!(empty_metrics.word_count, 0);
    assert_eq!(empty_metrics.line_count, 0);
    assert_eq!(empty_metrics.estimated_tokens, 0);

    let single_line = SectionTokenMetrics::compute_baseline("Hello world");
    assert_eq!(single_line.char_count, 11);
    assert_eq!(single_line.word_count, 2);
    assert_eq!(single_line.line_count, 1);
    assert_eq!(single_line.estimated_tokens, 3);
}

#[test]
fn test_all_enum_as_str_helpers() {
    assert_eq!(SourceType::WebpageStatic.as_str(), "webpage_static");
    assert_eq!(SourceType::WebpageDynamic.as_str(), "webpage_dynamic");
    assert_eq!(SourceType::GitHubPublic.as_str(), "github_public");
    assert_eq!(SourceType::GitHubPrivate.as_str(), "github_private");
    assert_eq!(SourceType::DocumentPdf.as_str(), "document_pdf");
    assert_eq!(SourceType::DocumentOcr.as_str(), "document_ocr");

    assert_eq!(SectionKind::Prose.as_str(), "prose");
    assert_eq!(SectionKind::CodeFile.as_str(), "code_file");
    assert_eq!(SectionKind::CodeSnippet.as_str(), "code_snippet");
    assert_eq!(SectionKind::Table.as_str(), "table");
    assert_eq!(SectionKind::Frontmatter.as_str(), "frontmatter");
    assert_eq!(SectionKind::Manifest.as_str(), "manifest");
    assert_eq!(SectionKind::Navigation.as_str(), "navigation");
    assert_eq!(SectionKind::Admonition.as_str(), "admonition");

    assert_eq!(HierarchyKind::Root.as_str(), "root");
    assert_eq!(HierarchyKind::Directory.as_str(), "directory");
    assert_eq!(HierarchyKind::File.as_str(), "file");
    assert_eq!(HierarchyKind::Section.as_str(), "section");
    assert_eq!(HierarchyKind::Heading.as_str(), "heading");

    assert_eq!(RelationshipKind::Imports.as_str(), "imports");
    assert_eq!(RelationshipKind::ImportedBy.as_str(), "imported_by");
    assert_eq!(RelationshipKind::Calls.as_str(), "calls");
    assert_eq!(RelationshipKind::Extends.as_str(), "extends");
    assert_eq!(RelationshipKind::LinksTo.as_str(), "links_to");
    assert_eq!(RelationshipKind::References.as_str(), "references");
    assert_eq!(RelationshipKind::ParentOf.as_str(), "parent_of");
    assert_eq!(RelationshipKind::ChildOf.as_str(), "child_of");
    assert_eq!(RelationshipKind::DefinesSymbol.as_str(), "defines_symbol");
    assert_eq!(RelationshipKind::DuplicateOf.as_str(), "duplicate_of");

    assert_eq!(ExclusionReason::RulePattern.as_str(), "rule_pattern");
    assert_eq!(ExclusionReason::SizeLimit.as_str(), "size_limit");
    assert_eq!(ExclusionReason::BinaryFile.as_str(), "binary_file");
    assert_eq!(
        ExclusionReason::ExcludedExtension.as_str(),
        "excluded_extension"
    );
    assert_eq!(ExclusionReason::Duplicate.as_str(), "duplicate");
    assert_eq!(ExclusionReason::UserConfig.as_str(), "user_config");
    assert_eq!(
        ExclusionReason::SecurityRestricted.as_str(),
        "security_restricted"
    );
    assert_eq!(
        ExclusionReason::UnsupportedType.as_str(),
        "unsupported_type"
    );

    assert_eq!(ExtractionStatus::Completed.as_str(), "completed");
    assert_eq!(ExtractionStatus::Partial.as_str(), "partial");
    assert_eq!(ExtractionStatus::Failed.as_str(), "failed");
    assert_eq!(ExtractionStatus::Unsupported.as_str(), "unsupported");

    assert_eq!(WarningSeverity::Critical.as_str(), "critical");
    assert_eq!(WarningSeverity::Warning.as_str(), "warning");
    assert_eq!(WarningSeverity::Info.as_str(), "info");
}
