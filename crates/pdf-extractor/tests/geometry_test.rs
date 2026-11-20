//! Bounding box geometry test suite.

use web_repo_to_ai_pdf_extractor::models::BoundingBox;

#[test]
fn test_bounding_box_geometry() {
    let bb = BoundingBox::new(10.0, 20.0, 110.0, 220.0);
    assert_eq!(bb.x0, 10.0);
    assert_eq!(bb.y0, 20.0);
    assert_eq!(bb.x1, 110.0);
    assert_eq!(bb.y1, 220.0);
    assert_eq!(bb.width(), 100.0);
    assert_eq!(bb.height(), 200.0);
    assert_eq!(bb.area(), 20000.0);

    // Normalization when inverted
    let inverted = BoundingBox::new(110.0, 220.0, 10.0, 20.0);
    assert_eq!(inverted, bb);

    // Contains
    let inside = BoundingBox::new(20.0, 30.0, 80.0, 150.0);
    assert!(bb.contains(&inside));
    assert!(!inside.contains(&bb));

    // Intersects
    let overlapping = BoundingBox::new(50.0, 50.0, 150.0, 250.0);
    assert!(bb.intersects(&overlapping));
    assert!(overlapping.intersects(&bb));

    let disjoint = BoundingBox::new(200.0, 300.0, 250.0, 350.0);
    assert!(!bb.intersects(&disjoint));

    // Union
    let u = bb.union(&disjoint);
    assert_eq!(u.x0, 10.0);
    assert_eq!(u.y0, 20.0);
    assert_eq!(u.x1, 250.0);
    assert_eq!(u.y1, 350.0);
}
