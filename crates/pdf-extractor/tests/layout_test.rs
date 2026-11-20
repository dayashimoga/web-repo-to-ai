//! Layout analysis test suite.

use web_repo_to_ai_pdf_extractor::layout::{LayoutOptions, PdfLayoutEngine};
use web_repo_to_ai_pdf_extractor::models::{BoundingBox, PageDimensions, PdfSpan, TextBlockKind};

#[test]
fn test_line_clustering_and_spacing() {
    let engine = PdfLayoutEngine::default();
    let dims = PageDimensions {
        width: 612.0,
        height: 792.0,
        rotation: 0,
    };

    let spans = vec![
        PdfSpan {
            text: "Hello".to_string(),
            font_name: Some("Helvetica".to_string()),
            font_size: 11.0,
            bbox: BoundingBox::new(72.0, 700.0, 102.0, 711.0),
        },
        PdfSpan {
            text: "World".to_string(),
            font_name: Some("Helvetica".to_string()),
            font_size: 11.0,
            bbox: BoundingBox::new(106.0, 700.0, 136.0, 711.0),
        },
    ];

    let blocks = engine.build_page_blocks(spans, &dims);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text, "Hello World");
    assert_eq!(blocks[0].kind, TextBlockKind::Paragraph);
}

#[test]
fn test_semantic_block_classification() {
    let engine = PdfLayoutEngine::default();
    let dims = PageDimensions {
        width: 612.0,
        height: 792.0,
        rotation: 0,
    };

    let spans = vec![
        // Running header at top
        PdfSpan {
            text: "Document Header 2026".to_string(),
            font_name: None,
            font_size: 9.0,
            bbox: BoundingBox::new(72.0, 750.0, 200.0, 759.0),
        },
        // Heading
        PdfSpan {
            text: "1. Introduction".to_string(),
            font_name: None,
            font_size: 16.0,
            bbox: BoundingBox::new(72.0, 680.0, 220.0, 696.0),
        },
        // Paragraph line 1
        PdfSpan {
            text: "This is the first paragraph describing the system.".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(72.0, 640.0, 350.0, 651.0),
        },
        // Paragraph line 2 (small vertical gap)
        PdfSpan {
            text: "It continues on the next line smoothly.".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(72.0, 626.0, 300.0, 637.0),
        },
        // List item 1
        PdfSpan {
            text: "- First bullet point item".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(72.0, 590.0, 250.0, 601.0),
        },
        // List item 2
        PdfSpan {
            text: "- Second bullet point item".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(72.0, 570.0, 260.0, 581.0),
        },
        // Page footer at bottom
        PdfSpan {
            text: "Page 1 of 10".to_string(),
            font_name: None,
            font_size: 9.0,
            bbox: BoundingBox::new(280.0, 30.0, 340.0, 39.0),
        },
    ];

    let blocks = engine.build_page_blocks(spans, &dims);
    assert_eq!(blocks.len(), 6);

    assert_eq!(blocks[0].kind, TextBlockKind::PageHeader);
    assert_eq!(blocks[0].text, "Document Header 2026");

    assert_eq!(blocks[1].kind, TextBlockKind::Header);
    assert_eq!(blocks[1].text, "1. Introduction");

    assert_eq!(blocks[2].kind, TextBlockKind::Paragraph);
    assert!(blocks[2].text.contains("first paragraph"));
    assert!(blocks[2].text.contains("continues on the next line"));

    assert_eq!(blocks[3].kind, TextBlockKind::ListItem);
    assert_eq!(blocks[3].text, "- First bullet point item");

    assert_eq!(blocks[4].kind, TextBlockKind::ListItem);
    assert_eq!(blocks[4].text, "- Second bullet point item");

    assert_eq!(blocks[5].kind, TextBlockKind::PageFooter);
    assert_eq!(blocks[5].text, "Page 1 of 10");
}

#[test]
fn test_multi_column_ordering() {
    let options = LayoutOptions {
        detect_columns: true,
        ..Default::default()
    };
    let engine = PdfLayoutEngine::new(options);
    let dims = PageDimensions {
        width: 600.0,
        height: 800.0,
        rotation: 0,
    };

    // Construct 3 lines on left column and 3 lines on right column
    let spans = vec![
        // Right column line 1 (y = 600)
        PdfSpan {
            text: "Right Col Line 1".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(320.0, 600.0, 450.0, 611.0),
        },
        // Left column line 1 (y = 600)
        PdfSpan {
            text: "Left Col Line 1".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(50.0, 600.0, 180.0, 611.0),
        },
        // Right column line 2 (y = 550)
        PdfSpan {
            text: "Right Col Line 2".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(320.0, 550.0, 450.0, 561.0),
        },
        // Left column line 2 (y = 550)
        PdfSpan {
            text: "Left Col Line 2".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(50.0, 550.0, 180.0, 561.0),
        },
        // Right column line 3 (y = 500)
        PdfSpan {
            text: "Right Col Line 3".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(320.0, 500.0, 450.0, 511.0),
        },
        // Left column line 3 (y = 500)
        PdfSpan {
            text: "Left Col Line 3".to_string(),
            font_name: None,
            font_size: 11.0,
            bbox: BoundingBox::new(50.0, 500.0, 180.0, 511.0),
        },
    ];

    let blocks = engine.build_page_blocks(spans, &dims);
    assert_eq!(blocks.len(), 6);

    // Left column should come first in reading order
    assert_eq!(blocks[0].text, "Left Col Line 1");
    assert_eq!(blocks[1].text, "Left Col Line 2");
    assert_eq!(blocks[2].text, "Left Col Line 3");
    // Followed by right column
    assert_eq!(blocks[3].text, "Right Col Line 1");
    assert_eq!(blocks[4].text, "Right Col Line 2");
    assert_eq!(blocks[5].text, "Right Col Line 3");
}
