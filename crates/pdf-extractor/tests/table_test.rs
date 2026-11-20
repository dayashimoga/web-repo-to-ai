//! Table detection and serialization test suite.

use web_repo_to_ai_pdf_extractor::models::{BoundingBox, PdfLine, PdfSpan};
use web_repo_to_ai_pdf_extractor::table::{
    PdfTable, PdfTableCell, PdfTableDetector, PdfTableRow, TableDetectorOptions,
};

fn create_sample_table() -> PdfTable {
    let header_cells = vec![
        PdfTableCell::new(
            "Quarter",
            BoundingBox::new(50.0, 700.0, 120.0, 712.0),
            0,
            0,
            true,
        ),
        PdfTableCell::new(
            "Revenue ($M)",
            BoundingBox::new(200.0, 700.0, 280.0, 712.0),
            0,
            1,
            true,
        ),
        PdfTableCell::new(
            "Growth",
            BoundingBox::new(350.0, 700.0, 410.0, 712.0),
            0,
            2,
            true,
        ),
    ];
    let row1_cells = vec![
        PdfTableCell::new(
            "Q1 2026",
            BoundingBox::new(50.0, 680.0, 120.0, 692.0),
            1,
            0,
            false,
        ),
        PdfTableCell::new(
            "145.2",
            BoundingBox::new(200.0, 680.0, 240.0, 692.0),
            1,
            1,
            false,
        ),
        PdfTableCell::new(
            "+12.4%",
            BoundingBox::new(350.0, 680.0, 400.0, 692.0),
            1,
            2,
            false,
        ),
    ];
    let row2_cells = vec![
        PdfTableCell::new(
            "Q2 2026",
            BoundingBox::new(50.0, 660.0, 120.0, 672.0),
            2,
            0,
            false,
        ),
        PdfTableCell::new(
            "168.0",
            BoundingBox::new(200.0, 660.0, 240.0, 672.0),
            2,
            1,
            false,
        ),
        PdfTableCell::new(
            "+15.7%",
            BoundingBox::new(350.0, 660.0, 400.0, 672.0),
            2,
            2,
            false,
        ),
    ];

    let row0 = PdfTableRow::new(
        header_cells,
        BoundingBox::new(50.0, 700.0, 410.0, 712.0),
        true,
    );
    let row1 = PdfTableRow::new(
        row1_cells,
        BoundingBox::new(50.0, 680.0, 410.0, 692.0),
        false,
    );
    let row2 = PdfTableRow::new(
        row2_cells,
        BoundingBox::new(50.0, 660.0, 410.0, 672.0),
        false,
    );

    PdfTable {
        table_id: "page-1-table-1".to_string(),
        page_number: 1,
        bbox: BoundingBox::new(50.0, 660.0, 410.0, 712.0),
        rows: vec![row0, row1, row2],
        column_count: 3,
        row_count: 3,
        caption: Some("Table 1: Quarterly Financial Performance".to_string()),
    }
}

#[test]
fn test_table_serialization_markdown() {
    let table = create_sample_table();
    let md = table.to_markdown();

    assert!(md.contains("**Table 1: Quarterly Financial Performance**"));
    assert!(md.contains("| Quarter | Revenue ($M) | Growth |"));
    assert!(md.contains("|---|---|---|"));
    assert!(md.contains("| Q1 2026 | 145.2 | +12.4% |"));
    assert!(md.contains("| Q2 2026 | 168.0 | +15.7% |"));
}

#[test]
fn test_table_serialization_html() {
    let table = create_sample_table();
    let html = table.to_html();

    assert!(html.contains("<table>"));
    assert!(html.contains("<caption>Table 1: Quarterly Financial Performance</caption>"));
    assert!(html.contains("<thead>"));
    assert!(html.contains("<th>Quarter</th>"));
    assert!(html.contains("<th>Revenue ($M)</th>"));
    assert!(html.contains("<tbody>"));
    assert!(html.contains("<td>Q1 2026</td>"));
    assert!(html.contains("<td>145.2</td>"));
    assert!(html.contains("</table>"));
}

#[test]
fn test_table_serialization_csv() {
    let table = create_sample_table();
    let csv = table.to_csv();

    let lines: Vec<&str> = csv.lines().collect();
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "Quarter,Revenue ($M),Growth");
    assert_eq!(lines[1], "Q1 2026,145.2,+12.4%");
    assert_eq!(lines[2], "Q2 2026,168.0,+15.7%");
}

#[test]
fn test_table_detection_from_spatial_lines() {
    let detector = PdfTableDetector::new();

    // Create preceding paragraph
    let para1 = PdfLine {
        text: "The company reported the following quarterly numbers:".to_string(),
        spans: vec![PdfSpan {
            text: "The company reported the following quarterly numbers:".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(50.0, 750.0, 380.0, 761.0),
        }],
        bbox: BoundingBox::new(50.0, 750.0, 380.0, 761.0),
        font_size: 11.0,
    };

    // Caption
    let caption = PdfLine {
        text: "Table 1: Quarterly Highlights".to_string(),
        spans: vec![PdfSpan {
            text: "Table 1: Quarterly Highlights".to_string(),
            font_name: None,
            font_size: 12.0,
            bbox: BoundingBox::new(50.0, 725.0, 220.0, 737.0),
        }],
        bbox: BoundingBox::new(50.0, 725.0, 220.0, 737.0),
        font_size: 12.0,
    };

    // Table rows (3 segments per row across 3 baselines)
    // Row 0 at y=700
    let r0_c0 = PdfLine {
        text: "Quarter".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(50.0, 700.0, 110.0, 711.0),
        font_size: 11.0,
    };
    let r0_c1 = PdfLine {
        text: "Revenue".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(200.0, 700.0, 260.0, 711.0),
        font_size: 11.0,
    };
    let r0_c2 = PdfLine {
        text: "Margin".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(350.0, 700.0, 400.0, 711.0),
        font_size: 11.0,
    };

    // Row 1 at y=680
    let r1_c0 = PdfLine {
        text: "Q1".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(50.0, 680.0, 80.0, 691.0),
        font_size: 11.0,
    };
    let r1_c1 = PdfLine {
        text: "$10M".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(200.0, 680.0, 235.0, 691.0),
        font_size: 11.0,
    };
    let r1_c2 = PdfLine {
        text: "25%".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(350.0, 680.0, 380.0, 691.0),
        font_size: 11.0,
    };

    // Row 2 at y=660
    let r2_c0 = PdfLine {
        text: "Q2".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(50.0, 660.0, 80.0, 671.0),
        font_size: 11.0,
    };
    let r2_c1 = PdfLine {
        text: "$12M".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(200.0, 660.0, 235.0, 671.0),
        font_size: 11.0,
    };
    let r2_c2 = PdfLine {
        text: "28%".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(350.0, 660.0, 380.0, 671.0),
        font_size: 11.0,
    };

    // Succeeding paragraph
    let para2 = PdfLine {
        text: "Overall growth continued into the subsequent fiscal year.".to_string(),
        spans: vec![PdfSpan {
            text: "Overall growth continued into the subsequent fiscal year.".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(50.0, 620.0, 390.0, 631.0),
        }],
        bbox: BoundingBox::new(50.0, 620.0, 390.0, 631.0),
        font_size: 11.0,
    };

    let all_lines = vec![
        para1.clone(),
        caption,
        r0_c0,
        r0_c1,
        r0_c2,
        r1_c0,
        r1_c1,
        r1_c2,
        r2_c0,
        r2_c1,
        r2_c2,
        para2.clone(),
    ];

    let (tables, non_table_lines) = detector.detect_tables(&all_lines, 1);

    assert_eq!(tables.len(), 1);
    let table = &tables[0];
    assert_eq!(table.column_count, 3);
    assert_eq!(table.row_count, 3);
    assert_eq!(
        table.caption.as_deref(),
        Some("Table 1: Quarterly Highlights")
    );
    assert_eq!(table.get_headers(), vec!["Quarter", "Revenue", "Margin"]);

    // Surrounding paragraphs remain in non_table_lines
    assert_eq!(non_table_lines.len(), 2);
    assert!(non_table_lines[0].text.contains("reported the following"));
    assert!(non_table_lines[1].text.contains("Overall growth continued"));
}

#[test]
fn test_sparse_table_detection() {
    let opts = TableDetectorOptions {
        min_columns: 2,
        min_rows: 2,
        ..Default::default()
    };
    let detector = PdfTableDetector::with_options(opts);

    // Row 0 has 2 columns
    let r0_c0 = PdfLine {
        text: "Item".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(50.0, 500.0, 100.0, 511.0),
        font_size: 10.0,
    };
    let r0_c1 = PdfLine {
        text: "Value".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(250.0, 500.0, 300.0, 511.0),
        font_size: 10.0,
    };

    // Row 1 has 2 columns
    let r1_c0 = PdfLine {
        text: "Alpha".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(50.0, 480.0, 100.0, 491.0),
        font_size: 10.0,
    };
    let r1_c1 = PdfLine {
        text: "100".to_string(),
        spans: vec![],
        bbox: BoundingBox::new(250.0, 480.0, 280.0, 491.0),
        font_size: 10.0,
    };

    let lines = vec![r0_c0, r0_c1, r1_c0, r1_c1];
    let (tables, non_table) = detector.detect_tables(&lines, 2);

    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].column_count, 2);
    assert_eq!(tables[0].row_count, 2);
    assert!(non_table.is_empty());
}
