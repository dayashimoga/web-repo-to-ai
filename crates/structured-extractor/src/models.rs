//! # Domain Models for Structured Document Extraction
//!
//! Strongly-typed representations of OpenAPI specifications, JSON Schema definitions,
//! YAML configurations, extraction options, and resource limits.

use crate::error::{Result, StructuredError};
use serde::{Deserialize, Serialize};

/// Classification of supported structured document formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredDocumentKind {
    /// OpenAPI 2.0 (Swagger), 3.0.x, or 3.1.x API specification in JSON or YAML.
    OpenApi,
    /// JSON Schema specification (Draft-04, Draft-07, Draft 2020-12, etc.).
    JsonSchema,
    /// Generic single- or multi-document YAML (Kubernetes, CI/CD, Compose).
    GenericYaml,
    /// Delimited tabular text (CSV, TSV).
    Csv,
    /// Jupyter Notebook interactive JSON format (`.ipynb`).
    JupyterNotebook,
}

impl StructuredDocumentKind {
    /// Returns a human-readable display label for the format.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::OpenApi => "OpenAPI Specification",
            Self::JsonSchema => "JSON Schema",
            Self::GenericYaml => "YAML Configuration",
            Self::Csv => "CSV/TSV Tabular Data",
            Self::JupyterNotebook => "Jupyter Notebook",
        }
    }

    /// Returns a lowercase snake_case identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::OpenApi => "openapi",
            Self::JsonSchema => "json_schema",
            Self::GenericYaml => "yaml",
            Self::Csv => "csv",
            Self::JupyterNotebook => "jupyter_notebook",
        }
    }
}

/// Input container representing a structured document for extraction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredInput {
    /// Raw bytes of the document.
    pub bytes: Vec<u8>,
    /// Optional filename hinting format detection (e.g. `openapi.yaml`, `schema.json`).
    pub filename: Option<String>,
    /// Optional source URL or address for attribution.
    pub source_url: Option<String>,
}

impl StructuredInput {
    /// Constructs a new input from raw byte vector.
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
            filename: None,
            source_url: None,
        }
    }

    /// Constructs a new input from a string slice.
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(content: &str) -> Self {
        Self::from_bytes(content.as_bytes().to_vec())
    }

    /// Sets the source filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = Some(filename.into());
        self
    }

    /// Sets the source URL.
    #[must_use]
    pub fn with_source_url(mut self, source_url: impl Into<String>) -> Self {
        self.source_url = Some(source_url.into());
        self
    }

    /// Returns the content as a UTF-8 string slice if valid UTF-8.
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.bytes)
            .map_err(|e| StructuredError::LimitExceeded(format!("Input is not valid UTF-8: {e}")))
    }
}

/// Resource and DoS protection limits enforced during extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredLimits {
    /// Maximum allowed input file size in bytes (default: 20 MiB).
    pub max_input_bytes: usize,
    /// Maximum allowed object nesting depth to prevent stack exhaustion (default: 64).
    pub max_nesting_depth: usize,
    /// Maximum number of endpoint operations to extract (default: 2,000).
    pub max_endpoints: usize,
    /// Maximum number of component schemas to extract (default: 1,000).
    pub max_schemas: usize,
    /// Maximum number of generated CDM sections (default: 2,000).
    pub max_sections: usize,
}

impl Default for StructuredLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 20 * 1024 * 1024, // 20 MiB
            max_nesting_depth: 64,
            max_endpoints: 2_000,
            max_schemas: 1_000,
            max_sections: 2_000,
        }
    }
}

impl StructuredLimits {
    /// Validates input byte length against the maximum size limit.
    pub fn validate_input(&self, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() {
            return Err(StructuredError::LimitExceeded("Input is empty".to_string()));
        }
        if bytes.len() > self.max_input_bytes {
            return Err(StructuredError::LimitExceeded(format!(
                "Input size {} bytes exceeds maximum allowed limit of {} bytes",
                bytes.len(),
                self.max_input_bytes
            )));
        }
        Ok(())
    }

    /// Sets custom maximum input bytes.
    #[must_use]
    pub const fn with_max_input_bytes(mut self, bytes: usize) -> Self {
        self.max_input_bytes = bytes;
        self
    }

    /// Sets custom maximum nesting depth.
    #[must_use]
    pub const fn with_max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    /// Sets custom maximum endpoints.
    #[must_use]
    pub const fn with_max_endpoints(mut self, endpoints: usize) -> Self {
        self.max_endpoints = endpoints;
        self
    }
}

/// Configuration options controlling structured document transformation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredOptions {
    /// Whether to group operations by tag (true) or by URL path (false).
    pub group_by_tag: bool,
    /// Whether to render parameter tables for operations.
    pub include_parameters: bool,
    /// Whether to render request body schemas and examples.
    pub include_request_body: bool,
    /// Whether to render response status codes and schema tables.
    pub include_responses: bool,
    /// Whether to render reusable component/definition data models.
    pub include_schemas: bool,
    /// Whether to include raw JSON/YAML overview snippets.
    pub include_code_snippets: bool,
    /// Safety and resource bounds.
    pub limits: StructuredLimits,
}

impl Default for StructuredOptions {
    fn default() -> Self {
        Self {
            group_by_tag: true,
            include_parameters: true,
            include_request_body: true,
            include_responses: true,
            include_schemas: true,
            include_code_snippets: true,
            limits: StructuredLimits::default(),
        }
    }
}

impl StructuredOptions {
    /// Sets grouping mode: true for tag grouping, false for path grouping.
    #[must_use]
    pub const fn with_group_by_tag(mut self, group: bool) -> Self {
        self.group_by_tag = group;
        self
    }

    /// Sets whether to include component schema models.
    #[must_use]
    pub const fn with_include_schemas(mut self, include: bool) -> Self {
        self.include_schemas = include;
        self
    }

    /// Sets resource limits.
    #[must_use]
    pub fn with_limits(mut self, limits: StructuredLimits) -> Self {
        self.limits = limits;
        self
    }
}

// ============================================================================
// OPENAPI MODELS
// ============================================================================

/// Parsed representation of an OpenAPI or Swagger specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiSpec {
    /// Specification standard version (e.g. "3.1.0", "3.0.3", "2.0").
    pub spec_format_version: String,
    /// API title from info object.
    pub title: String,
    /// API version from info object.
    pub api_version: String,
    /// Optional overall description.
    pub description: Option<String>,
    /// Base server URLs.
    pub servers: Vec<String>,
    /// Path items and their operations.
    pub paths: Vec<OpenApiPath>,
    /// Reusable component data schemas.
    pub schemas: Vec<OpenApiSchema>,
    /// Declared tags with descriptions.
    pub tags: Vec<OpenApiTag>,
}

/// Represents an API endpoint path containing one or more HTTP operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiPath {
    /// URL path template (e.g. `/api/v1/jobs/{job_id}`).
    pub path: String,
    /// HTTP operations defined on this path.
    pub operations: Vec<OpenApiOperation>,
}

/// An individual HTTP method operation on a path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiOperation {
    /// HTTP Method (GET, POST, PUT, DELETE, PATCH, etc.).
    pub method: String,
    /// Optional operation identifier.
    pub operation_id: Option<String>,
    /// Brief one-line summary.
    pub summary: Option<String>,
    /// Detailed description of the operation.
    pub description: Option<String>,
    /// Associated organizational tags.
    pub tags: Vec<String>,
    /// Whether the operation is deprecated.
    pub deprecated: bool,
    /// Operation parameters (query, path, header, cookie).
    pub parameters: Vec<OpenApiParameter>,
    /// Optional request payload definition.
    pub request_body: Option<OpenApiRequestBody>,
    /// Responses returned by the endpoint.
    pub responses: Vec<OpenApiResponse>,
    /// Security requirement scheme names.
    pub security: Vec<String>,
}

/// Parameter definition for an API operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiParameter {
    /// Parameter name.
    pub name: String,
    /// Location of the parameter: "query", "path", "header", "cookie".
    pub location: String,
    /// Whether the parameter is required.
    pub required: bool,
    /// Parameter data type (string, integer, boolean, array, etc.).
    pub param_type: String,
    /// Parameter description.
    pub description: Option<String>,
    /// Whether the parameter is marked deprecated.
    pub deprecated: bool,
}

/// Request body payload definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiRequestBody {
    /// Description of the request body.
    pub description: Option<String>,
    /// Whether the request body is required.
    pub required: bool,
    /// Supported MIME content types (e.g. "application/json").
    pub content_types: Vec<String>,
    /// Summary of schema type or reference.
    pub schema_summary: Option<String>,
}

/// Response returned by an endpoint operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiResponse {
    /// HTTP status code or "default" (e.g. "200", "400", "404").
    pub status_code: String,
    /// Description of the response meaning.
    pub description: String,
    /// Supported MIME content types.
    pub content_types: Vec<String>,
    /// Summary of schema type or reference.
    pub schema_summary: Option<String>,
}

/// Reusable component data schema or model.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiSchema {
    /// Model / schema identifier name.
    pub name: String,
    /// Schema type (e.g. "object", "array", "string").
    pub schema_type: String,
    /// Description of the schema.
    pub description: Option<String>,
    /// Object property definitions if type is object.
    pub properties: Vec<OpenApiProperty>,
    /// List of required property names.
    pub required_fields: Vec<String>,
    /// Allowed enum values if scalar enum.
    pub enum_values: Vec<String>,
}

/// Property field within an object schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiProperty {
    /// Property name.
    pub name: String,
    /// Property type (string, integer, boolean, array, object).
    pub prop_type: String,
    /// Specific format (e.g. "date-time", "uuid", "int64").
    pub format: Option<String>,
    /// Whether this field is marked required in the parent schema.
    pub required: bool,
    /// Description of the field.
    pub description: Option<String>,
    /// Example value if provided.
    pub example: Option<String>,
}

/// Tag definition for grouping operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApiTag {
    /// Tag name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
}

// ============================================================================
// JSON SCHEMA MODELS
// ============================================================================

/// Parsed representation of a standalone JSON Schema definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonSchemaDoc {
    /// Schema dialect identifier (e.g. `http://json-schema.org/draft-07/schema#`).
    pub schema_version: Option<String>,
    /// Schema identifier URI (`$id` or `id`).
    pub id: Option<String>,
    /// Schema human-readable title.
    pub title: Option<String>,
    /// Schema description.
    pub description: Option<String>,
    /// Root data type ("object", "array", "string", etc.).
    pub root_type: String,
    /// Properties if root is object.
    pub properties: Vec<JsonSchemaProperty>,
    /// Required property names.
    pub required: Vec<String>,
    /// Allowed enum values if root is an enum.
    pub enum_values: Vec<String>,
    /// Sub-definitions or `$defs`.
    pub definitions: Vec<JsonSchemaDoc>,
}

/// Property field definition within a JSON Schema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JsonSchemaProperty {
    /// Property name.
    pub name: String,
    /// Data type.
    pub prop_type: String,
    /// Format identifier (e.g. "email", "uri", "date-time").
    pub format: Option<String>,
    /// Whether required by parent schema.
    pub required: bool,
    /// Description of the property.
    pub description: Option<String>,
    /// Default value if specified.
    pub default_val: Option<String>,
    /// Allowed enum values.
    pub enum_values: Vec<String>,
}

// ============================================================================
// GENERIC YAML MODELS
// ============================================================================

/// Parsed representation of a generic YAML document stream.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct YamlDocument {
    /// 1-based index within a multi-document YAML stream.
    pub document_index: usize,
    /// Total number of top-level keys or entries.
    pub top_level_count: usize,
    /// Top-level key names (if a mapping/dictionary).
    pub top_level_keys: Vec<String>,
    /// Rendered clean YAML representation.
    pub clean_yaml: String,
}
