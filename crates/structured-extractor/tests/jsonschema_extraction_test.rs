//! # JSON Schema Extraction Tests
//!
//! Validates parsing of JSON Schema Draft-07 / Draft 2020-12 specifications from JSON and YAML,
//! property table synthesis, required field validation, enum values, and sub-definitions (`$defs`).

use web_repo_to_ai_cdm::SourceType;
use web_repo_to_ai_structured_extractor::{
    detect_format, parse_json_schema, StructuredDocumentKind, StructuredExtractor, StructuredInput,
    StructuredOptions,
};

const SAMPLE_JSON_SCHEMA: &str = r#"{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$id": "https://example.com/schemas/user-profile.json",
  "title": "UserProfile",
  "description": "User account and profile configuration record",
  "type": "object",
  "required": ["id", "email", "username"],
  "properties": {
    "id": {
      "type": "string",
      "format": "uuid",
      "description": "Globally unique identifier"
    },
    "email": {
      "type": "string",
      "format": "email",
      "description": "Primary communication address"
    },
    "username": {
      "type": "string",
      "description": "Unique handle"
    },
    "role": {
      "type": "string",
      "default": "viewer",
      "enum": ["admin", "editor", "viewer"]
    },
    "age": {
      "type": "integer",
      "description": "Age in completed years"
    }
  },
  "$defs": {
    "Address": {
      "type": "object",
      "title": "AddressInfo",
      "description": "Physical postal address",
      "required": ["street", "city", "country"],
      "properties": {
        "street": {
          "type": "string"
        },
        "city": {
          "type": "string"
        },
        "country": {
          "type": "string"
        },
        "postal_code": {
          "type": "string"
        }
      }
    }
  }
}"#;

const SAMPLE_JSON_SCHEMA_YAML: &str = r#"
$schema: "https://json-schema.org/draft/2020-12/schema"
title: DeploymentConfiguration
description: Kubernetes deployment replica configuration
type: object
required:
  - app_name
  - replicas
properties:
  app_name:
    type: string
    description: Target application name
  replicas:
    type: integer
    default: 3
    description: Number of desired pods
  tier:
    type: string
    enum: [frontend, backend, worker]
"#;

#[test]
fn test_json_schema_detection_and_parsing() {
    let input = StructuredInput::from_str(SAMPLE_JSON_SCHEMA).with_filename("user-profile.json");
    let kind = detect_format(&input).expect("detect format");
    assert_eq!(kind, StructuredDocumentKind::JsonSchema);

    let schema = parse_json_schema(SAMPLE_JSON_SCHEMA).expect("parse JSON schema");
    assert_eq!(schema.title.as_deref(), Some("UserProfile"));
    assert_eq!(schema.root_type, "object");
    assert_eq!(schema.properties.len(), 5);
    assert_eq!(schema.required.len(), 3);
    assert_eq!(schema.definitions.len(), 1);

    let role_prop = schema
        .properties
        .iter()
        .find(|p| p.name == "role")
        .expect("role prop");
    assert_eq!(role_prop.default_val.as_deref(), Some("viewer"));
    assert_eq!(role_prop.enum_values, vec!["admin", "editor", "viewer"]);
}

#[test]
fn test_yaml_encoded_json_schema_parsing() {
    let input =
        StructuredInput::from_str(SAMPLE_JSON_SCHEMA_YAML).with_filename("config.schema.yaml");
    let kind = detect_format(&input).expect("detect format");
    assert_eq!(kind, StructuredDocumentKind::JsonSchema);

    let schema = parse_json_schema(SAMPLE_JSON_SCHEMA_YAML).expect("parse YAML schema");
    assert_eq!(schema.title.as_deref(), Some("DeploymentConfiguration"));
    assert_eq!(schema.properties.len(), 3);
}

#[test]
fn test_json_schema_cdm_extraction_and_sections() {
    let extractor = StructuredExtractor::new();
    let input = StructuredInput::from_str(SAMPLE_JSON_SCHEMA)
        .with_filename("user-profile.json")
        .with_source_url("https://example.com/schemas/user-profile.json");
    let options = StructuredOptions::default();

    let cdm = extractor
        .extract_to_cdm(&input, &options)
        .expect("extract to CDM");

    assert_eq!(cdm.source_type, SourceType::StructuredDocument);
    assert_eq!(cdm.metadata.title.as_deref(), Some("UserProfile"));
    assert_eq!(cdm.metadata.extra["structured_format"], "json_schema");
    assert_eq!(cdm.metadata.extra["root_type"], "object");
    assert_eq!(cdm.metadata.extra["property_count"], "5");
    assert_eq!(cdm.metadata.extra["definition_count"], "1");

    // Sections: Root Schema (1) + Address definition (1) = 2 sections
    assert_eq!(cdm.sections.len(), 2);

    let root_sec = &cdm.sections[0];
    assert!(root_sec
        .title
        .as_deref()
        .unwrap_or("")
        .contains("UserProfile"));
    assert!(root_sec
        .body
        .contains("| `id` | `string` | Yes | `uuid` | `-` |"));
    assert!(root_sec
        .body
        .contains("| `email` | `string` | Yes | `email` | `-` |"));
    assert!(root_sec
        .body
        .contains("| `role` | `string` | No | `-` | `viewer` |"));

    let def_sec = &cdm.sections[1];
    assert!(def_sec
        .title
        .as_deref()
        .unwrap_or("")
        .contains("AddressInfo"));
    assert!(def_sec.body.contains("| `street` | `string` | Yes | `-` |"));
    assert!(def_sec.body.contains("| `city` | `string` | Yes | `-` |"));

    assert_eq!(cdm.hierarchy.name, "UserProfile");
    assert_eq!(cdm.hierarchy.children.len(), 2);
}
