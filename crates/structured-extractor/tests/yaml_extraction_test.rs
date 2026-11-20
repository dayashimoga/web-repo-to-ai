//! # YAML Configuration and Multi-Document Stream Extraction Tests
//!
//! Validates parsing of single-document and multi-document YAML streams (`---`),
//! top-level key indexing, code block preservation, and CDM synthesis.

use web_repo_to_ai_cdm::{SectionKind, SourceType};
use web_repo_to_ai_structured_extractor::{
    detect_format, parse_yaml_stream, StructuredDocumentKind, StructuredExtractor, StructuredInput,
    StructuredOptions,
};

const SAMPLE_MULTI_DOC_YAML: &str = r#"
---
apiVersion: v1
kind: Service
metadata:
  name: web-service
  labels:
    app: web
spec:
  ports:
    - port: 80
      targetPort: 8080
  selector:
    app: web
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: web-deployment
spec:
  replicas: 3
  template:
    spec:
      containers:
        - name: web
          image: nginx:1.25
"#;

#[test]
fn test_yaml_detection_and_multi_document_stream() {
    let input = StructuredInput::from_str(SAMPLE_MULTI_DOC_YAML).with_filename("k8s-manifest.yaml");
    let kind = detect_format(&input).expect("detect format");
    assert_eq!(kind, StructuredDocumentKind::GenericYaml);

    let docs = parse_yaml_stream(SAMPLE_MULTI_DOC_YAML).expect("parse YAML stream");
    assert_eq!(docs.len(), 2);

    assert_eq!(docs[0].document_index, 1);
    assert_eq!(docs[0].top_level_count, 4);
    assert!(docs[0].top_level_keys.contains(&"apiVersion".to_string()));
    assert!(docs[0].top_level_keys.contains(&"kind".to_string()));
    assert!(docs[0].clean_yaml.contains("web-service"));

    assert_eq!(docs[1].document_index, 2);
    assert_eq!(docs[1].top_level_count, 4);
    assert!(docs[1].top_level_keys.contains(&"apiVersion".to_string()));
    assert!(docs[1].top_level_keys.contains(&"kind".to_string()));
    assert!(docs[1].top_level_keys.contains(&"metadata".to_string()));
    assert!(docs[1].top_level_keys.contains(&"spec".to_string()));
    assert!(docs[1].clean_yaml.contains("web-deployment"));
}

#[test]
fn test_yaml_cdm_extraction_and_sections() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(SAMPLE_MULTI_DOC_YAML)
        .with_filename("k8s-service.yaml")
        .with_source_url("file:///deploy/k8s-service.yaml");
    let options = StructuredOptions::default();

    let cdm = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract to CDM");

    assert_eq!(cdm.source_type, SourceType::StructuredDocument);
    assert_eq!(cdm.metadata.title.as_deref(), Some("k8s-service.yaml"));
    assert_eq!(cdm.metadata.extra["structured_format"], "yaml");
    assert_eq!(cdm.metadata.extra["document_count"], "2");

    // 2 documents = 2 sections
    assert_eq!(cdm.sections.len(), 2);
    assert_eq!(cdm.sections[0].kind, SectionKind::CodeSnippet);
    assert!(cdm.sections[0].body.contains("```yaml"));
    assert!(cdm.sections[0].body.contains("kind: Service"));

    assert_eq!(cdm.sections[1].kind, SectionKind::CodeSnippet);
    assert!(cdm.sections[1].body.contains("kind: Deployment"));

    assert_eq!(cdm.hierarchy.children.len(), 2);
}
