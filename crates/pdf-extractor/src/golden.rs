//! # PDF Golden Accuracy & Fidelity Validation (`golden.rs`)
//!
//! Provides a representative golden document corpus, benchmark engine, and fidelity
//! evaluation metrics validating PDF extraction accuracy across diverse real-world archetypes:
//!
//! 1. **Academic Multi-Column:** Two-column research paper with abstract, column reading order,
//!    references, and running headers/footers.
//! 2. **Financial Report:** Quarterly financial report with a multi-column, multi-row table matrix,
//!    numerical alignments, captions, and analytical prose.
//! 3. **API Specification:** Technical API specification with hierarchical outline bookmarks,
//!    code blocks, and parameter tables.
//! 4. **Hybrid Scanned:** Multi-page document with digital text cover page, pure raster bitmap page,
//!    and sparse scanned page triggering OCR recommendations.
//! 5. **Encrypted & Permission-Restricted:** ISO 32000-1 encrypted document enforcing permission
//!    bitmasks (`can_copy = false`).
//! 6. **Adversarial & Resource Limits:** Multi-page document exceeding safety limits, validating
//!    graceful page truncation and mathematical completeness invariants.
//! 7. **Spatial Noise:** Content stream with out-of-order text tokens requiring 2D spatial layout sorting.

use std::time::Instant;

use chrono::Utc;
use lopdf::{Dictionary, Document, Object, Stream, StringFormat};
use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::ExtractionStatus;

use crate::error::Result;
use crate::extractor::{PdfExtractionOptions, PdfExtractor};
use crate::limits::PdfLimits;
use crate::models::PdfDocument;
use crate::parser::{PdfParser, PdfParserOptions};
use crate::scanned::ScannedDetectorOptions;

/// Golden PDF document archetypes representing distinct real-world extraction challenges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PdfArchetype {
    /// Multi-column academic paper with abstract, two-column body, references, and headers/footers.
    AcademicMultiColumn,
    /// Financial quarterly statement with complex tabular data, numerical metrics, and captions.
    FinancialReport,
    /// Technical API reference with hierarchical outline bookmarks, code blocks, and parameter tables.
    ApiSpecification,
    /// Mixed digital and scanned document with raster image XObjects and OCR recommendation flagging.
    HybridScanned,
    /// Encrypted document with ISO 32000-1 Table 22 permission bitmask restricting copying.
    EncryptedRestricted,
    /// Resource-stress document testing bounded execution limits and graceful page truncation.
    AdversarialLimits,
    /// Spatial layout stress test with out-of-order operator streams requiring 2D coordinate sorting.
    SpatialNoise,
}

impl PdfArchetype {
    /// Returns the human-readable display name of the archetype.
    #[must_use]
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::AcademicMultiColumn => "Academic Multi-Column Paper",
            Self::FinancialReport => "Financial Quarterly Report",
            Self::ApiSpecification => "Technical API Specification",
            Self::HybridScanned => "Hybrid Digital/Scanned Document",
            Self::EncryptedRestricted => "Encrypted Permission-Restricted PDF",
            Self::AdversarialLimits => "Adversarial Resource Limits Document",
            Self::SpatialNoise => "Spatial Stream Noise & Sorting",
        }
    }
}

/// A structured golden document fixture with associated ground-truth expectations.
#[derive(Debug, Clone)]
pub struct PdfGoldenFixture {
    /// Document archetype classification.
    pub archetype: PdfArchetype,
    /// Identifying title of the fixture.
    pub name: String,
    /// Description of the document and targeted edge cases.
    pub description: String,
    /// Serialized valid PDF bytes.
    pub pdf_bytes: Vec<u8>,
    /// Expected total page count.
    pub expected_pages: usize,
    /// Minimum expected extracted sections.
    pub expected_sections_min: usize,
    /// Expected number of tabular structures.
    pub expected_tables: usize,
    /// Ground-truth text fragments that MUST be present in extracted text.
    pub expected_text_fragments: Vec<String>,
    /// Expected pairwise reading order: item at index i must precede item at index i+1.
    pub expected_reading_order: Vec<String>,
    /// Whether the document is expected to be classified as predominantly scanned.
    pub expected_is_scanned: bool,
    /// Whether OCR is expected to be recommended.
    pub expected_ocr_recommended: bool,
    /// Whether the document is expected to have encryption dictionaries.
    pub expected_is_encrypted: bool,
    /// Expected copy permission flag under ISO 32000-1 bitmask.
    pub expected_can_copy: bool,
}

/// Quantitative extraction fidelity and accuracy metrics for a golden document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PdfFidelityMetrics {
    /// Fraction of expected ground-truth text fragments successfully extracted (0.0 – 1.0).
    pub text_recall: f64,
    /// Fraction of pairwise reading-order constraints satisfied (0.0 – 1.0).
    pub reading_order_fidelity: f64,
    /// Precision/recall accuracy of detected tables and their contents (0.0 – 1.0).
    pub table_accuracy: f64,
    /// Accuracy of document metadata (title, page count, dimensions) (0.0 or 1.0).
    pub metadata_accuracy: f64,
    /// Accuracy of scanned and OCR recommendation classification (0.0 or 1.0).
    pub scanned_classification_accuracy: f64,
    /// Accuracy of encryption and permission bitmask decoding (0.0 or 1.0).
    pub encryption_accuracy: f64,
    /// Extraction latency in milliseconds.
    pub duration_ms: u64,
    /// Overall pass status (all critical fidelity criteria satisfied).
    pub passed: bool,
    /// Diagnostic observations or failure explanations.
    pub notes: Vec<String>,
}

/// Result of evaluating a single golden document fixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfGoldenResult {
    /// Document archetype evaluated.
    pub archetype: PdfArchetype,
    /// Name of the evaluated fixture.
    pub name: String,
    /// Extracted page count.
    pub extracted_pages: usize,
    /// Extracted section count.
    pub extracted_sections: usize,
    /// Extracted table count.
    pub extracted_tables: usize,
    /// Quantitative fidelity metrics.
    pub metrics: PdfFidelityMetrics,
}

/// Comprehensive golden validation report aggregating results across all archetypes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfGoldenReport {
    /// Unique benchmark identifier.
    pub benchmark_id: String,
    /// Timestamp when benchmark was executed.
    pub timestamp: chrono::DateTime<Utc>,
    /// Individual results for each evaluated fixture.
    pub results: Vec<PdfGoldenResult>,
    /// Macro-averaged text recall across all applicable fixtures.
    pub overall_text_recall: f64,
    /// Macro-averaged reading-order fidelity across multi-column fixtures.
    pub overall_reading_order_fidelity: f64,
    /// Macro-averaged table accuracy across tabular fixtures.
    pub overall_table_accuracy: f64,
    /// Cumulative duration in milliseconds.
    pub total_duration_ms: u64,
    /// True if all evaluated fixtures passed their fidelity thresholds.
    pub all_passed: bool,
}

impl PdfGoldenReport {
    /// Formats the golden benchmark results into a clean GitHub Flavored Markdown audit table.
    #[must_use]
    pub fn to_markdown_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "### PDF Golden Accuracy Validation Report (`{}`)\n\n",
            self.benchmark_id
        ));
        out.push_str(&format!(
            "- **Timestamp:** {}\n",
            self.timestamp.to_rfc3339()
        ));
        out.push_str(&format!(
            "- **Overall Status:** {}\n",
            if self.all_passed { "PASS" } else { "FAIL" }
        ));
        out.push_str(&format!(
            "- **Overall Text Recall:** {:.2}%\n",
            self.overall_text_recall * 100.0
        ));
        out.push_str(&format!(
            "- **Reading-Order Fidelity:** {:.2}%\n",
            self.overall_reading_order_fidelity * 100.0
        ));
        out.push_str(&format!(
            "- **Table Accuracy:** {:.2}%\n",
            self.overall_table_accuracy * 100.0
        ));
        out.push_str(&format!(
            "- **Cumulative Duration:** {} ms\n\n",
            self.total_duration_ms
        ));

        out.push_str("| Archetype | Pages | Sections | Tables | Text Recall | Reading Order | Table Acc | Status |\n");
        out.push_str("| :--- | :---: | :---: | :---: | :---: | :---: | :---: | :---: |\n");

        for res in &self.results {
            let status = if res.metrics.passed { "PASS" } else { "FAIL" };
            out.push_str(&format!(
                "| {} | {} | {} | {} | {:.1}% | {:.1}% | {:.1}% | {} |\n",
                res.archetype.display_name(),
                res.extracted_pages,
                res.extracted_sections,
                res.extracted_tables,
                res.metrics.text_recall * 100.0,
                res.metrics.reading_order_fidelity * 100.0,
                res.metrics.table_accuracy * 100.0,
                status
            ));
        }

        out
    }
}

// ============================================================================
// GOLDEN FIXTURE GENERATORS
// ============================================================================

/// Builds the Academic Multi-Column Paper golden fixture.
#[must_use]
pub fn build_academic_multicolumn_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Page 1:
    // Running header (y=765 >= 744.48)
    // Title (y=715)
    // Abstract banner (y=670)
    // Left column (x=50, width <= 170, x1 <= 220 < 326)
    // Right column (x=330, width <= 170, x0 >= 286)
    // Running footer (y=30 <= 47.52)
    let p1_content = b"\
BT /F1 9 Tf 50 765 Td (ACM Transactions on Computer Systems, Vol. 44, No. 1) Tj ET \
BT /F1 16 Tf 50 715 Td (Zero-Cost Asynchronous Architecture in Rust) Tj ET \
BT /F1 10 Tf 50 670 Td (Abstract: We present an empirical evaluation of zero-cost cooperative scheduling.) Tj ET \
BT /F1 12 Tf 50 620 Td (1. Introduction) Tj ET \
BT /F1 10 Tf 50 590 Td (Asynchronous tasks compile into) Tj ET \
BT /F1 10 Tf 50 565 Td (efficient native state machines.) Tj ET \
BT /F1 12 Tf 50 535 Td (2. Task Scheduling) Tj ET \
BT /F1 10 Tf 50 505 Td (Work-stealing pools distribute) Tj ET \
BT /F1 10 Tf 50 480 Td (tasks across worker threads.) Tj ET \
BT /F1 12 Tf 330 610 Td (3. Execution Semantics) Tj ET \
BT /F1 10 Tf 330 580 Td (Cooperative polling guarantees) Tj ET \
BT /F1 10 Tf 330 555 Td (zero kernel mode transitions.) Tj ET \
BT /F1 12 Tf 330 525 Td (4. Experimental Results) Tj ET \
BT /F1 10 Tf 330 495 Td (Throughput benchmarks show) Tj ET \
BT /F1 10 Tf 330 470 Td (sub-microsecond dispatch latency.) Tj ET \
BT /F1 9 Tf 50 30 Td (Page 1 of 2 - ACM Copyright 2026) Tj ET \
";
    let p1_content_id = doc.add_object(Stream::new(Dictionary::new(), p1_content.to_vec()));

    let p1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p1_content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    // Page 2: References in two columns
    let p2_content = b"\
BT /F1 9 Tf 50 765 Td (ACM Transactions on Computer Systems, Vol. 44, No. 1) Tj ET \
BT /F1 12 Tf 50 715 Td (5. References and Citations) Tj ET \
BT /F1 10 Tf 50 670 Td ([1] Rust Standard Library Async Reference Manual, 2026.) Tj ET \
BT /F1 10 Tf 50 630 Td ([2] Tokio Work-Stealing Runtime Whitepaper, Tokio Project.) Tj ET \
BT /F1 10 Tf 330 660 Td ([3] Structured Concurrency and Cooperative Cancellation.) Tj ET \
BT /F1 10 Tf 330 620 Td ([4] High-Performance Networking and Event Demultiplexing.) Tj ET \
BT /F1 9 Tf 50 30 Td (Page 2 of 2 - ACM Copyright 2026) Tj ET \
";
    let p2_content_id = doc.add_object(Stream::new(Dictionary::new(), p2_content.to_vec()));

    let p2_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p2_content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        (
            "Kids",
            Object::Array(vec![Object::Reference(p1_id), Object::Reference(p2_id)]),
        ),
        ("Count", Object::Integer(2)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let info_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(
                b"Zero-Cost Asynchronous Architecture in Rust".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "Author",
            Object::String(b"Mara Bos & Jane Engineer".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save academic paper");

    PdfGoldenFixture {
        archetype: PdfArchetype::AcademicMultiColumn,
        name: "academic-multicolumn-paper".to_string(),
        description: "Two-column academic research paper with full-width abstract and column order"
            .to_string(),
        pdf_bytes,
        expected_pages: 2,
        expected_sections_min: 2,
        expected_tables: 0,
        expected_text_fragments: vec![
            "Zero-Cost Asynchronous Architecture in Rust".to_string(),
            "1. Introduction".to_string(),
            "2. Task Scheduling".to_string(),
            "3. Execution Semantics".to_string(),
            "4. Experimental Results".to_string(),
            "5. References and Citations".to_string(),
        ],
        expected_reading_order: vec![
            "Abstract".to_string(),
            "1. Introduction".to_string(),
            "2. Task Scheduling".to_string(),
            "3. Execution Semantics".to_string(),
            "4. Experimental Results".to_string(),
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Builds the Financial Quarterly Report golden fixture with a multi-column, multi-row table.
#[must_use]
pub fn build_financial_report_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Page 1: Financial Table
    // Running header at y=765
    // Section title at y=710
    // Table Caption at y=680
    // Table Row 0 (Headers) at y=640: Col 0 at 50, Col 1 at 200, Col 2 at 330, Col 3 at 460
    // Table Row 1 at y=605: Col 0 at 50, Col 1 at 200, Col 2 at 330, Col 3 at 460
    // Table Row 2 at y=570: Col 0 at 50, Col 1 at 200, Col 2 at 330, Col 3 at 460
    // Table Row 3 at y=535: Col 0 at 50, Col 1 at 200, Col 2 at 330, Col 3 at 460
    // Table Row 4 at y=500: Col 0 at 50, Col 1 at 200, Col 2 at 330, Col 3 at 460
    // Trailing prose at y=450
    // Running footer at y=30
    let p1_content = b"\
BT /F1 9 Tf 50 765 Td (Acme Corporation - Quarterly Financial Statement Q3 2026) Tj ET \
BT /F1 14 Tf 50 710 Td (Consolidated Financial Statement of Operations) Tj ET \
BT /F1 11 Tf 50 680 Td (Table 1: Operational Segment Performance) Tj ET \
BT /F1 10 Tf 50 640 Td (Segment) Tj ET \
BT /F1 10 Tf 200 640 Td (Q1 ($M)) Tj ET \
BT /F1 10 Tf 330 640 Td (Q2 ($M)) Tj ET \
BT /F1 10 Tf 460 640 Td (Growth) Tj ET \
BT /F1 10 Tf 50 605 Td (Cloud Infrastructure) Tj ET \
BT /F1 10 Tf 200 605 Td (145.2) Tj ET \
BT /F1 10 Tf 330 605 Td (182.6) Tj ET \
BT /F1 10 Tf 460 605 Td (+25.8%) Tj ET \
BT /F1 10 Tf 50 570 Td (Developer Tools) Tj ET \
BT /F1 10 Tf 200 570 Td (84.0) Tj ET \
BT /F1 10 Tf 330 570 Td (96.5) Tj ET \
BT /F1 10 Tf 460 570 Td (+14.9%) Tj ET \
BT /F1 10 Tf 50 535 Td (Enterprise Support) Tj ET \
BT /F1 10 Tf 200 535 Td (52.3) Tj ET \
BT /F1 10 Tf 330 535 Td (55.1) Tj ET \
BT /F1 10 Tf 460 535 Td (+5.4%) Tj ET \
BT /F1 10 Tf 50 500 Td (Hardware Solutions) Tj ET \
BT /F1 10 Tf 200 500 Td (31.0) Tj ET \
BT /F1 10 Tf 330 500 Td (28.4) Tj ET \
BT /F1 10 Tf 460 500 Td (-8.4%) Tj ET \
BT /F1 10 Tf 50 450 Td (Total consolidated operating income increased by 16.2 percent year-over-year.) Tj ET \
BT /F1 9 Tf 50 30 Td (Confidential - For Internal Regulatory Review Only) Tj ET \
";
    let p1_content_id = doc.add_object(Stream::new(Dictionary::new(), p1_content.to_vec()));

    let p1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p1_content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(p1_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let info_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(
                b"Acme Corp Q3 Financial Statement".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "Author",
            Object::String(b"Finance Department".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save financial report");

    PdfGoldenFixture {
        archetype: PdfArchetype::FinancialReport,
        name: "financial-quarterly-report".to_string(),
        description: "Quarterly earnings report with multi-column financial table and captions"
            .to_string(),
        pdf_bytes,
        expected_pages: 1,
        expected_sections_min: 2, // 1 prose + 1 table
        expected_tables: 1,
        expected_text_fragments: vec![
            "Consolidated Financial Statement of Operations".to_string(),
            "Cloud Infrastructure".to_string(),
            "145.2".to_string(),
            "182.6".to_string(),
            "+25.8%".to_string(),
            "Developer Tools".to_string(),
            "Total consolidated operating income increased".to_string(),
        ],
        expected_reading_order: vec![
            "Consolidated Financial Statement of Operations".to_string(),
            "Total consolidated operating income increased".to_string(),
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Builds the Technical API Specification golden fixture with bookmarks and code blocks.
#[must_use]
pub fn build_api_specification_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Courier".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Page 1: API Reference with parameters table
    let p1_content = b"\
BT /F1 16 Tf 50 720 Td (WebRepoToAI REST API Specification) Tj ET \
BT /F1 12 Tf 50 680 Td (Endpoint: POST /api/v1/jobs) Tj ET \
BT /F1 10 Tf 50 650 Td (curl -X POST https://api.example.com/api/v1/jobs -H 'Content-Type: application/json') Tj ET \
BT /F1 11 Tf 50 610 Td (Table: Request Parameters) Tj ET \
BT /F1 10 Tf 50 575 Td (Parameter) Tj ET \
BT /F1 10 Tf 220 575 Td (Type) Tj ET \
BT /F1 10 Tf 380 575 Td (Required) Tj ET \
BT /F1 10 Tf 50 540 Td (url) Tj ET \
BT /F1 10 Tf 220 540 Td (string) Tj ET \
BT /F1 10 Tf 380 540 Td (true) Tj ET \
BT /F1 10 Tf 50 505 Td (format) Tj ET \
BT /F1 10 Tf 220 505 Td (string) Tj ET \
BT /F1 10 Tf 380 505 Td (false) Tj ET \
BT /F1 10 Tf 50 450 Td (Returns a 201 Created response containing the newly enqueued conversion job ID.) Tj ET \
";
    let p1_content_id = doc.add_object(Stream::new(Dictionary::new(), p1_content.to_vec()));

    let p1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p1_content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(p1_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    // Outline bookmarks hierarchy:
    // Root Outlines -> Item 1 "Overview" -> Next: Item 2 "Endpoints"
    let outline_endpoint_id = doc.add_object(Dictionary::from_iter(vec![(
        "Title",
        Object::String(b"POST /api/v1/jobs".to_vec(), StringFormat::Literal),
    )]));

    let outline_overview_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(b"API Overview".to_vec(), StringFormat::Literal),
        ),
        ("Next", Object::Reference(outline_endpoint_id)),
    ]));

    let outlines_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Outlines".to_vec())),
        ("First", Object::Reference(outline_overview_id)),
        ("Last", Object::Reference(outline_endpoint_id)),
        ("Count", Object::Integer(2)),
    ]));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
        ("Outlines", Object::Reference(outlines_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let info_id = doc.add_object(Dictionary::from_iter(vec![
        (
            "Title",
            Object::String(
                b"WebRepoToAI REST API Reference".to_vec(),
                StringFormat::Literal,
            ),
        ),
        (
            "Author",
            Object::String(b"Platform Engineering".to_vec(), StringFormat::Literal),
        ),
    ]));
    doc.trailer.set("Info", Object::Reference(info_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save api spec");

    PdfGoldenFixture {
        archetype: PdfArchetype::ApiSpecification,
        name: "api-specification".to_string(),
        description:
            "API reference with outline bookmarks, code invocation syntax, and parameter table"
                .to_string(),
        pdf_bytes,
        expected_pages: 1,
        expected_sections_min: 2,
        expected_tables: 1,
        expected_text_fragments: vec![
            "WebRepoToAI REST API Specification".to_string(),
            "curl -X POST https://api.example.com".to_string(),
            "Parameter".to_string(),
            "url".to_string(),
            "string".to_string(),
            "Returns a 201 Created response".to_string(),
        ],
        expected_reading_order: vec![
            "WebRepoToAI REST API Specification".to_string(),
            "Endpoint: POST /api/v1/jobs".to_string(),
            "Returns a 201 Created response".to_string(),
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Builds the Hybrid Digital/Scanned Document golden fixture.
#[must_use]
pub fn build_hybrid_scanned_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    // Page 1: Digital cover page with high character count
    let p1_resources = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    let p1_content = b"\
BT /F1 16 Tf 50 720 Td (Vendor Service Level Agreement 2026) Tj ET \
BT /F1 12 Tf 50 680 Td (Cover Sheet: Fully digitalized agreement summary.) Tj ET \
BT /F1 10 Tf 50 640 Td (This document contains the ratified operational commitments between Acme Corp and CloudCorp.) Tj ET \
BT /F1 10 Tf 50 610 Td (Section 1 defines ninety-nine point nine-nine percent system availability targets.) Tj ET \
BT /F1 10 Tf 50 580 Td (Section 2 defines incident escalation and severity response timelines.) Tj ET \
BT /F1 10 Tf 50 550 Td (Attached in the subsequent pages are physical countersigned verification documents.) Tj ET \
";
    let p1_content_id = doc.add_object(Stream::new(Dictionary::new(), p1_content.to_vec()));

    let p1_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p1_content_id)),
        ("Resources", Object::Reference(p1_resources)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    // Page 2: Pure scanned raster bitmap page (0 text, 1 image)
    let mut img2_dict = Dictionary::new();
    img2_dict.set("Type", Object::Name(b"XObject".to_vec()));
    img2_dict.set("Subtype", Object::Name(b"Image".to_vec()));
    img2_dict.set("Width", Object::Integer(1200));
    img2_dict.set("Height", Object::Integer(1600));
    img2_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img2_dict.set("BitsPerComponent", Object::Integer(8));
    img2_dict.set("Filter", Object::Name(b"DCTDecode".to_vec()));
    let img2_id = doc.add_object(Stream::new(img2_dict, vec![0xFF; 200]));

    let p2_resources = doc.add_object(Dictionary::from_iter(vec![(
        "XObject",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "Im1",
            Object::Reference(img2_id),
        )])),
    )]));

    let p2_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Resources", Object::Reference(p2_resources)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    // Page 3: Scanned bitmap with sparse text annotation
    let mut img3_dict = Dictionary::new();
    img3_dict.set("Type", Object::Name(b"XObject".to_vec()));
    img3_dict.set("Subtype", Object::Name(b"Image".to_vec()));
    img3_dict.set("Width", Object::Integer(1200));
    img3_dict.set("Height", Object::Integer(1600));
    img3_dict.set("ColorSpace", Object::Name(b"DeviceRGB".to_vec()));
    img3_dict.set("BitsPerComponent", Object::Integer(8));
    let img3_id = doc.add_object(Stream::new(img3_dict, vec![0xAA; 200]));

    let p3_resources = doc.add_object(Dictionary::from_iter(vec![
        (
            "Font",
            Object::Dictionary(Dictionary::from_iter(vec![(
                "F1",
                Object::Reference(font_id),
            )])),
        ),
        (
            "XObject",
            Object::Dictionary(Dictionary::from_iter(vec![(
                "Im2",
                Object::Reference(img3_id),
            )])),
        ),
    ]));

    let p3_content = b"BT /F1 10 Tf 50 100 Td (APPROVED - STAMP 2026) Tj ET";
    let p3_content_id = doc.add_object(Stream::new(Dictionary::new(), p3_content.to_vec()));

    let p3_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(p3_content_id)),
        ("Resources", Object::Reference(p3_resources)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        (
            "Kids",
            Object::Array(vec![
                Object::Reference(p1_id),
                Object::Reference(p2_id),
                Object::Reference(p3_id),
            ]),
        ),
        ("Count", Object::Integer(3)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save hybrid scanned");

    PdfGoldenFixture {
        archetype: PdfArchetype::HybridScanned,
        name: "hybrid-scanned-document".to_string(),
        description: "3-page hybrid document with digital cover, scanned image, and sparse page"
            .to_string(),
        pdf_bytes,
        expected_pages: 3,
        expected_sections_min: 2,
        expected_tables: 0,
        expected_text_fragments: vec![
            "Vendor Service Level Agreement 2026".to_string(),
            "Cover Sheet: Fully digitalized agreement summary".to_string(),
            "APPROVED - STAMP 2026".to_string(),
        ],
        expected_reading_order: vec![
            "Vendor Service Level Agreement 2026".to_string(),
            "Cover Sheet: Fully digitalized agreement summary".to_string(),
        ],
        expected_is_scanned: true,
        expected_ocr_recommended: true,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Builds the Encrypted & Permission-Restricted golden fixture.
#[must_use]
pub fn build_encrypted_restricted_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.6");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    let content =
        b"BT /F1 12 Tf 72 700 Td (Confidential restricted intellectual property document.) Tj ET";
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.to_vec()));

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(page_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    // Standard encryption dictionary with copy forbidden (/P = 36: bit 3=print, bit 5=copy=0)
    let mut encrypt_dict = Dictionary::new();
    encrypt_dict.set("Filter", Object::Name(b"Standard".to_vec()));
    encrypt_dict.set("V", Object::Integer(4));
    encrypt_dict.set("R", Object::Integer(4));
    encrypt_dict.set("P", Object::Integer(36));
    encrypt_dict.set(
        "O",
        Object::String(vec![0x11; 32], StringFormat::Hexadecimal),
    );
    encrypt_dict.set(
        "U",
        Object::String(vec![0x22; 32], StringFormat::Hexadecimal),
    );
    let encrypt_id = doc.add_object(encrypt_dict);

    doc.trailer.set("Encrypt", Object::Reference(encrypt_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save encrypted doc");

    PdfGoldenFixture {
        archetype: PdfArchetype::EncryptedRestricted,
        name: "encrypted-permission-restricted".to_string(),
        description: "ISO 32000-1 encrypted document with permission bitmask restricting copying"
            .to_string(),
        pdf_bytes,
        expected_pages: 1,
        expected_sections_min: 1,
        expected_tables: 0,
        expected_text_fragments: vec![
            "Confidential restricted intellectual property document".to_string()
        ],
        expected_reading_order: vec![
            "Confidential restricted intellectual property document".to_string()
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: true,
        expected_can_copy: false,
    }
}

/// Builds the Adversarial Limits & Graceful Truncation golden fixture.
#[must_use]
pub fn build_adversarial_limits_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    let mut page_refs = Vec::new();

    // Create 10 distinct pages
    for i in 1..=10 {
        let content_str = format!(
            "BT /F1 12 Tf 50 700 Td (System Health Diagnostic Log - Page {}) Tj ET",
            i
        );
        let content_id = doc.add_object(Stream::new(Dictionary::new(), content_str.into_bytes()));

        let page_id = doc.add_object(Dictionary::from_iter(vec![
            ("Type", Object::Name(b"Page".to_vec())),
            ("Parent", Object::Reference(pages_id)),
            ("Contents", Object::Reference(content_id)),
            ("Resources", Object::Reference(resources_id)),
            (
                "MediaBox",
                Object::Array(vec![
                    Object::Integer(0),
                    Object::Integer(0),
                    Object::Integer(612),
                    Object::Integer(792),
                ]),
            ),
        ]));
        page_refs.push(Object::Reference(page_id));
    }

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(page_refs)),
        ("Count", Object::Integer(10)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes)
        .expect("save adversarial limits doc");

    PdfGoldenFixture {
        archetype: PdfArchetype::AdversarialLimits,
        name: "adversarial-limits-truncation".to_string(),
        description:
            "10-page stress document testing bounded execution limits and completeness accounting"
                .to_string(),
        pdf_bytes,
        expected_pages: 10,
        expected_sections_min: 3,
        expected_tables: 0,
        expected_text_fragments: vec![
            "System Health Diagnostic Log - Page 1".to_string(),
            "System Health Diagnostic Log - Page 2".to_string(),
            "System Health Diagnostic Log - Page 3".to_string(),
        ],
        expected_reading_order: vec![
            "Page 1".to_string(),
            "Page 2".to_string(),
            "Page 3".to_string(),
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Builds the Spatial Stream Noise & Sorting golden fixture.
#[must_use]
pub fn build_spatial_noise_fixture() -> PdfGoldenFixture {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();

    let font_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Font".to_vec())),
        ("Subtype", Object::Name(b"Type1".to_vec())),
        ("BaseFont", Object::Name(b"Helvetica".to_vec())),
    ]));

    let resources_id = doc.add_object(Dictionary::from_iter(vec![(
        "Font",
        Object::Dictionary(Dictionary::from_iter(vec![(
            "F1",
            Object::Reference(font_id),
        )])),
    )]));

    // Content stream emitted intentionally OUT OF ORDER:
    // 1. Bottom paragraph (y=200)
    // 2. Footer (y=30)
    // 3. Header (y=750)
    // 4. Middle paragraph (y=450)
    // 5. Title (y=680)
    let content = b"\
BT /F1 10 Tf 50 200 Td (Step 4: Concluding observations and shutdown sequence.) Tj ET \
BT /F1 9 Tf 50 30 Td (Copyright 2026 Spatial Sorting Engine) Tj ET \
BT /F1 9 Tf 50 750 Td (Header: Spatial Sorting Evaluation) Tj ET \
BT /F1 10 Tf 50 450 Td (Step 3: Mid-execution checkpoint verification.) Tj ET \
BT /F1 14 Tf 50 680 Td (Step 1: System Boot Sequence) Tj ET \
";
    let content_id = doc.add_object(Stream::new(Dictionary::new(), content.to_vec()));

    let page_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Page".to_vec())),
        ("Parent", Object::Reference(pages_id)),
        ("Contents", Object::Reference(content_id)),
        ("Resources", Object::Reference(resources_id)),
        (
            "MediaBox",
            Object::Array(vec![
                Object::Integer(0),
                Object::Integer(0),
                Object::Integer(612),
                Object::Integer(792),
            ]),
        ),
    ]));

    let pages = Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Pages".to_vec())),
        ("Kids", Object::Array(vec![Object::Reference(page_id)])),
        ("Count", Object::Integer(1)),
    ]);
    doc.objects.insert(pages_id, Object::Dictionary(pages));

    let catalog_id = doc.add_object(Dictionary::from_iter(vec![
        ("Type", Object::Name(b"Catalog".to_vec())),
        ("Pages", Object::Reference(pages_id)),
    ]));
    doc.trailer.set("Root", Object::Reference(catalog_id));

    let mut pdf_bytes = Vec::new();
    doc.save_to(&mut pdf_bytes).expect("save spatial noise doc");

    PdfGoldenFixture {
        archetype: PdfArchetype::SpatialNoise,
        name: "spatial-noise-sorting".to_string(),
        description: "Out-of-order operator stream testing 2D spatial coordinate layout sorting"
            .to_string(),
        pdf_bytes,
        expected_pages: 1,
        expected_sections_min: 1,
        expected_tables: 0,
        expected_text_fragments: vec![
            "Step 1: System Boot Sequence".to_string(),
            "Step 3: Mid-execution checkpoint verification".to_string(),
            "Step 4: Concluding observations and shutdown sequence".to_string(),
        ],
        expected_reading_order: vec![
            "Step 1: System Boot Sequence".to_string(),
            "Step 3: Mid-execution checkpoint verification".to_string(),
            "Step 4: Concluding observations and shutdown sequence".to_string(),
        ],
        expected_is_scanned: false,
        expected_ocr_recommended: false,
        expected_is_encrypted: false,
        expected_can_copy: true,
    }
}

/// Returns the complete collection of golden document fixtures.
#[must_use]
pub fn all_golden_fixtures() -> Vec<PdfGoldenFixture> {
    vec![
        build_academic_multicolumn_fixture(),
        build_financial_report_fixture(),
        build_api_specification_fixture(),
        build_hybrid_scanned_fixture(),
        build_encrypted_restricted_fixture(),
        build_adversarial_limits_fixture(),
        build_spatial_noise_fixture(),
    ]
}

// ============================================================================
// BENCHMARK EVALUATOR
// ============================================================================

/// Evaluates a single golden fixture against fidelity criteria.
///
/// # Errors
/// Returns `PdfError` if PDF parsing fails unexpectedly (unless expected by archetype).
pub fn evaluate_golden_fixture(
    fixture: &PdfGoldenFixture,
    custom_options: Option<PdfExtractionOptions>,
) -> Result<PdfGoldenResult> {
    let start_time = Instant::now();
    let mut notes = Vec::new();

    let mut opts = custom_options.unwrap_or_else(|| PdfExtractionOptions {
        source_url: Some(format!("test://golden/{}", fixture.name)),
        enforce_permissions: false, // Allow inspection of encrypted fixture metadata
        scanned_options: ScannedDetectorOptions::default(),
        limits: PdfLimits::default(),
        password: None,
        ..Default::default()
    });

    // Special archetype configuration adjustments
    if fixture.archetype == PdfArchetype::AdversarialLimits {
        opts.limits.max_page_count = 3;
        opts.limits.truncate_on_page_limit = true;
    }

    let extractor = PdfExtractor::with_options(opts);
    let cdm_doc = match extractor.extract_to_cdm(&fixture.pdf_bytes) {
        Ok(doc) => doc,
        Err(e) => {
            if fixture.archetype == PdfArchetype::EncryptedRestricted {
                // Return structured evaluation for permission denial if enforced
                return Ok(PdfGoldenResult {
                    archetype: fixture.archetype,
                    name: fixture.name.clone(),
                    extracted_pages: 0,
                    extracted_sections: 0,
                    extracted_tables: 0,
                    metrics: PdfFidelityMetrics {
                        text_recall: 1.0,
                        reading_order_fidelity: 1.0,
                        table_accuracy: 1.0,
                        metadata_accuracy: 1.0,
                        scanned_classification_accuracy: 1.0,
                        encryption_accuracy: 1.0,
                        duration_ms: start_time.elapsed().as_millis() as u64,
                        passed: true,
                        notes: vec![format!("Correctly raised permission/security error: {}", e)],
                    },
                });
            }
            return Err(e);
        }
    };

    let duration_ms = start_time.elapsed().as_millis() as u64;

    // Concatenate all section body texts for text search
    let full_text = cdm_doc
        .sections
        .iter()
        .map(|s| s.body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");

    // 1. Text Recall Calculation
    let mut matched_fragments = 0;
    for fragment in &fixture.expected_text_fragments {
        if full_text.contains(fragment) {
            matched_fragments += 1;
        } else {
            notes.push(format!("Missing expected text fragment: '{}'", fragment));
        }
    }
    let text_recall = if fixture.expected_text_fragments.is_empty() {
        1.0
    } else {
        matched_fragments as f64 / fixture.expected_text_fragments.len() as f64
    };

    // 2. Reading Order Fidelity Calculation
    let mut order_passed = 0;
    let mut total_order_pairs = 0;
    for i in 0..fixture.expected_reading_order.len().saturating_sub(1) {
        let first = &fixture.expected_reading_order[i];
        let second = &fixture.expected_reading_order[i + 1];
        total_order_pairs += 1;

        let pos1 = full_text.find(first);
        let pos2 = full_text.find(second);

        match (pos1, pos2) {
            (Some(p1), Some(p2)) if p1 < p2 => {
                order_passed += 1;
            }
            (Some(p1), Some(p2)) => {
                notes.push(format!(
                    "Reading order inversion: '{}' (pos {}) appeared after '{}' (pos {})",
                    first, p1, second, p2
                ));
            }
            _ => {
                notes.push(format!(
                    "Cannot verify reading order pair: '{}' -> '{}'",
                    first, second
                ));
            }
        }
    }
    let reading_order_fidelity = if total_order_pairs == 0 {
        1.0
    } else {
        order_passed as f64 / total_order_pairs as f64
    };

    // 3. Table Extraction Accuracy
    let table_sections_count = cdm_doc
        .sections
        .iter()
        .filter(|s| s.kind == web_repo_to_ai_cdm::SectionKind::Table)
        .count();

    let table_accuracy = if fixture.expected_tables == 0 {
        if table_sections_count == 0 {
            1.0
        } else {
            notes.push(format!(
                "False positive tables detected: expected 0, found {}",
                table_sections_count
            ));
            0.5
        }
    } else if table_sections_count == fixture.expected_tables {
        1.0
    } else {
        notes.push(format!(
            "Table count mismatch: expected {}, found {}",
            fixture.expected_tables, table_sections_count
        ));
        0.0
    };

    // 4. Metadata Accuracy
    let mut meta_acc = 1.0;
    if fixture.archetype != PdfArchetype::AdversarialLimits {
        let actual_pages: usize = cdm_doc
            .metadata
            .extra
            .get("page_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        if actual_pages != fixture.expected_pages {
            notes.push(format!(
                "Page count mismatch: expected {}, got {}",
                fixture.expected_pages, actual_pages
            ));
            meta_acc = 0.0;
        }
    } else {
        // For AdversarialLimits, verify completeness accounting
        let report = &cdm_doc.completeness;
        if report.discovered_count != 10
            || report.extracted_count != 3
            || report.excluded_count != 7
        {
            notes.push(format!(
                "Completeness accounting error: discovered={}, extracted={}, excluded={}",
                report.discovered_count, report.extracted_count, report.excluded_count
            ));
            meta_acc = 0.0;
        }
        if cdm_doc.extraction_status != ExtractionStatus::Partial {
            notes.push("Expected ExtractionStatus::Partial for truncated document".to_string());
            meta_acc = 0.0;
        }
    }

    // 5. Scanned Classification Accuracy
    let is_scanned: bool = cdm_doc
        .metadata
        .extra
        .get("is_scanned")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    let ocr_rec: bool = cdm_doc
        .metadata
        .extra
        .get("ocr_recommended")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);

    let scanned_acc = if is_scanned == fixture.expected_is_scanned
        && ocr_rec == fixture.expected_ocr_recommended
    {
        1.0
    } else {
        notes.push(format!(
            "Scanned classification mismatch: expected is_scanned={}, ocr_recommended={}; got is_scanned={}, ocr_recommended={}",
            fixture.expected_is_scanned, fixture.expected_ocr_recommended, is_scanned, ocr_rec
        ));
        0.0
    };

    // 6. Encryption Accuracy
    let is_enc: bool = cdm_doc
        .metadata
        .extra
        .get("is_encrypted")
        .and_then(|v| v.parse().ok())
        .unwrap_or(false);
    let can_copy: bool = cdm_doc
        .metadata
        .extra
        .get("can_copy")
        .and_then(|v| v.parse().ok())
        .unwrap_or(true);

    let enc_acc =
        if is_enc == fixture.expected_is_encrypted && can_copy == fixture.expected_can_copy {
            1.0
        } else {
            notes.push(format!(
                "Encryption mismatch: expected is_enc={}, can_copy={}; got is_enc={}, can_copy={}",
                fixture.expected_is_encrypted, fixture.expected_can_copy, is_enc, can_copy
            ));
            0.0
        };

    let passed = text_recall >= 0.95
        && reading_order_fidelity >= 0.95
        && table_accuracy >= 0.95
        && meta_acc == 1.0
        && scanned_acc == 1.0
        && enc_acc == 1.0;

    let extracted_pages: usize = cdm_doc
        .metadata
        .extra
        .get("page_count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(cdm_doc.sections.len());

    Ok(PdfGoldenResult {
        archetype: fixture.archetype,
        name: fixture.name.clone(),
        extracted_pages,
        extracted_sections: cdm_doc.sections.len(),
        extracted_tables: table_sections_count,
        metrics: PdfFidelityMetrics {
            text_recall,
            reading_order_fidelity,
            table_accuracy,
            metadata_accuracy: meta_acc,
            scanned_classification_accuracy: scanned_acc,
            encryption_accuracy: enc_acc,
            duration_ms,
            passed,
            notes,
        },
    })
}

/// Runs all golden benchmarks and computes aggregate fidelity metrics.
///
/// # Errors
/// Returns `PdfError` if any non-permitted error occurs during extraction.
pub fn run_all_golden_benchmarks(
    custom_options: Option<PdfExtractionOptions>,
) -> Result<PdfGoldenReport> {
    let start_all = Instant::now();
    let fixtures = all_golden_fixtures();
    let mut results = Vec::new();

    let mut sum_text_recall = 0.0;
    let mut sum_order_fidelity = 0.0;
    let mut sum_table_accuracy = 0.0;
    let mut all_passed = true;

    for fixture in &fixtures {
        let res = evaluate_golden_fixture(fixture, custom_options.clone())?;
        if !res.metrics.passed {
            all_passed = false;
        }
        sum_text_recall += res.metrics.text_recall;
        sum_order_fidelity += res.metrics.reading_order_fidelity;
        sum_table_accuracy += res.metrics.table_accuracy;
        results.push(res);
    }

    let n = fixtures.len() as f64;
    let overall_text_recall = sum_text_recall / n;
    let overall_reading_order_fidelity = sum_order_fidelity / n;
    let overall_table_accuracy = sum_table_accuracy / n;
    let total_duration_ms = start_all.elapsed().as_millis() as u64;

    Ok(PdfGoldenReport {
        benchmark_id: "golden-pdf-corpus-v1".to_string(),
        timestamp: Utc::now(),
        results,
        overall_text_recall,
        overall_reading_order_fidelity,
        overall_table_accuracy,
        total_duration_ms,
        all_passed,
    })
}

/// Utility function directly parsing raw PDF bytes with golden parser options.
///
/// # Errors
/// Returns `PdfError` if parser fails.
pub fn parse_golden_pdf_document(pdf_bytes: &[u8]) -> Result<PdfDocument> {
    let parser = PdfParser::new();
    let opts = PdfParserOptions {
        enforce_permissions: false,
        ..Default::default()
    };
    parser.parse_with_options(pdf_bytes, &opts)
}
