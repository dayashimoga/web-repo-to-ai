//! # Structured Document Extractor Crate (`web-repo-to-ai-structured-extractor`)
//!
//! Provides extraction, format detection, parsing, and Canonical Context Document Model
//! (`CdmDocument`) synthesis for:
//! - OpenAPI 2.0 (Swagger), 3.0.x, and 3.1.x specifications (JSON and YAML)
//! - JSON Schema definitions (Draft-04, Draft-07, Draft 2020-12)
//! - Generic single- and multi-document YAML configurations
//! - CSV/TSV tabular data (Subphase 10.3.2)
//! - Jupyter Notebooks (`.ipynb`) (Subphase 10.3.3)

pub mod detector;
pub mod error;
pub mod extractor;
pub mod jsonschema;
pub mod models;
pub mod openapi;
pub mod yaml;

pub use detector::detect_format;
pub use error::{Result, StructuredError};
pub use extractor::StructuredExtractor;
pub use jsonschema::parse_json_schema;
pub use models::{
    JsonSchemaDoc, JsonSchemaProperty, OpenApiOperation, OpenApiParameter, OpenApiPath,
    OpenApiProperty, OpenApiRequestBody, OpenApiResponse, OpenApiSchema, OpenApiSpec, OpenApiTag,
    StructuredDocumentKind, StructuredInput, StructuredLimits, StructuredOptions, YamlDocument,
};
pub use openapi::parse_openapi;
pub use yaml::parse_yaml_stream;
