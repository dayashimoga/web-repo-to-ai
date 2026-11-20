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
    /// Maximum allowed tabular rows to process (default: 10,000).
    pub max_tabular_rows: usize,
    /// Maximum allowed tabular columns to process (default: 500).
    pub max_tabular_columns: usize,
    /// Maximum allowed cell size in bytes (default: 64 KiB).
    pub max_cell_bytes: usize,
    /// Whether to truncate gracefully when max_tabular_rows is reached (default: true).
    pub truncate_on_row_limit: bool,
}

impl Default for StructuredLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: 20 * 1024 * 1024, // 20 MiB
            max_nesting_depth: 64,
            max_endpoints: 2_000,
            max_schemas: 1_000,
            max_sections: 2_000,
            max_tabular_rows: 10_000,
            max_tabular_columns: 500,
            max_cell_bytes: 64 * 1024, // 64 KiB
            truncate_on_row_limit: true,
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

    /// Sets custom maximum tabular rows.
    #[must_use]
    pub const fn with_max_tabular_rows(mut self, rows: usize) -> Self {
        self.max_tabular_rows = rows;
        self
    }

    /// Sets custom maximum tabular columns.
    #[must_use]
    pub const fn with_max_tabular_columns(mut self, cols: usize) -> Self {
        self.max_tabular_columns = cols;
        self
    }

    /// Sets custom maximum cell size in bytes.
    #[must_use]
    pub const fn with_max_cell_bytes(mut self, bytes: usize) -> Self {
        self.max_cell_bytes = bytes;
        self
    }

    /// Sets whether to truncate gracefully on row limit.
    #[must_use]
    pub const fn with_truncate_on_row_limit(mut self, truncate: bool) -> Self {
        self.truncate_on_row_limit = truncate;
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
    /// Whether to trim leading and trailing whitespace from tabular cells.
    pub tabular_trim_whitespace: bool,
    /// Whether to normalize common null representations (NA, null, -, etc.) to empty strings.
    pub tabular_normalize_nulls: bool,
    /// Whether to infer column data types for tabular data.
    pub tabular_infer_types: bool,
    /// Whether to include a summary schema section for tabular datasets.
    pub tabular_include_summary_section: bool,
    /// Whether to include the data table section.
    pub tabular_include_data_section: bool,
    /// Maximum number of preview rows to format in Markdown table (default: 2,000).
    pub tabular_max_preview_rows: usize,
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
            tabular_trim_whitespace: true,
            tabular_normalize_nulls: true,
            tabular_infer_types: true,
            tabular_include_summary_section: true,
            tabular_include_data_section: true,
            tabular_max_preview_rows: 2_000,
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

    /// Sets tabular whitespace trimming.
    #[must_use]
    pub const fn with_tabular_trim_whitespace(mut self, trim: bool) -> Self {
        self.tabular_trim_whitespace = trim;
        self
    }

    /// Sets tabular null normalization.
    #[must_use]
    pub const fn with_tabular_normalize_nulls(mut self, normalize: bool) -> Self {
        self.tabular_normalize_nulls = normalize;
        self
    }

    /// Sets tabular maximum preview rows.
    #[must_use]
    pub const fn with_tabular_max_preview_rows(mut self, max: usize) -> Self {
        self.tabular_max_preview_rows = max;
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

// ============================================================================
// TABULAR DATA MODELS (CSV/TSV)
// ============================================================================

/// Delimiter characters supported for tabular data parsing and sniffing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TabularDelimiter {
    /// Comma (`,`) - standard CSV.
    Comma,
    /// Tab (`\t`) - standard TSV.
    Tab,
    /// Semicolon (`;`) - European CSV.
    Semicolon,
    /// Pipe (`|`) - ASCII/Markdown pipe delimited.
    Pipe,
    /// Custom single-character delimiter.
    Custom(char),
}

impl TabularDelimiter {
    /// Returns the character representation of the delimiter.
    #[must_use]
    pub const fn as_char(self) -> char {
        match self {
            Self::Comma => ',',
            Self::Tab => '\t',
            Self::Semicolon => ';',
            Self::Pipe => '|',
            Self::Custom(c) => c,
        }
    }

    /// Returns a human-readable display label.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Comma => "Comma (,)",
            Self::Tab => "Tab (\\t)",
            Self::Semicolon => "Semicolon (;)",
            Self::Pipe => "Pipe (|)",
            Self::Custom(_) => "Custom",
        }
    }
}

/// Inferred data type of a tabular column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColumnDataType {
    /// Integer numerical values.
    Integer,
    /// Floating point decimal numbers.
    Float,
    /// Boolean flags (`true`/`false`, `yes`/`no`).
    Boolean,
    /// ISO 8601 or common calendar dates/timestamps.
    DateTime,
    /// Generic textual string fallback.
    String,
}

impl ColumnDataType {
    /// Returns a lowercase identifier string.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::DateTime => "datetime",
            Self::String => "string",
        }
    }
}

/// Statistical profile and schema definition for a tabular column.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnStats {
    /// 0-indexed column position.
    pub index: usize,
    /// Column header or assigned name.
    pub name: String,
    /// Inferred data type.
    pub data_type: ColumnDataType,
    /// Total row count in dataset.
    pub total_count: usize,
    /// Count of populated, non-empty cells.
    pub non_empty_count: usize,
    /// Count of empty or null cells.
    pub null_count: usize,
    /// Approximate count of unique distinct values.
    pub unique_count: usize,
    /// Up to 5 representative sample non-null values.
    pub sample_values: Vec<String>,
    /// Minimum value (alphanumerically or numerically).
    pub min_value: Option<String>,
    /// Maximum value (alphanumerically or numerically).
    pub max_value: Option<String>,
}

/// Fully parsed, normalized tabular dataset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TabularTable {
    /// Optional dataset name or source filename.
    pub name: Option<String>,
    /// Detected or configured delimiter.
    pub delimiter: TabularDelimiter,
    /// Whether the first row was treated as headers.
    pub has_headers: bool,
    /// Header names in column order.
    pub headers: Vec<String>,
    /// Statistical schema profile per column.
    pub columns: Vec<ColumnStats>,
    /// Extracted tabular rows (each row contains strings in column order).
    pub rows: Vec<Vec<String>>,
    /// Total discovered row count in raw input.
    pub total_rows: usize,
    /// Extracted row count within configured limits.
    pub extracted_rows: usize,
    /// Whether data was truncated due to row limit exhaustion.
    pub truncated: bool,
}

impl TabularTable {
    /// Computes data density (ratio of non-empty cells to total cells).
    #[must_use]
    pub fn data_density(&self) -> f64 {
        if self.rows.is_empty() || self.columns.is_empty() {
            return 0.0;
        }
        let total_cells = self.rows.len() * self.columns.len();
        let non_empty_cells: usize = self
            .rows
            .iter()
            .map(|r| r.iter().filter(|c| !c.is_empty()).count())
            .sum();
        (non_empty_cells as f64) / (total_cells as f64)
    }

    /// Serializes extracted data rows to a GitHub Flavored Markdown (GFM) pipe table.
    #[must_use]
    pub fn to_markdown(&self, max_rows: Option<usize>) -> String {
        if self.columns.is_empty() {
            return String::new();
        }

        let mut md = String::new();

        // Header row
        md.push('|');
        for h in &self.headers {
            md.push(' ');
            md.push_str(&sanitize_markdown_cell(h));
            md.push_str(" |");
        }
        md.push('\n');

        // Separator row
        md.push('|');
        for _ in &self.headers {
            md.push_str(" --- |");
        }
        md.push('\n');

        // Data rows
        let limit = max_rows.unwrap_or(self.rows.len());
        let render_count = self.rows.len().min(limit);

        for row in &self.rows[..render_count] {
            md.push('|');
            for col_idx in 0..self.headers.len() {
                let cell_val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                md.push(' ');
                md.push_str(&sanitize_markdown_cell(cell_val));
                md.push_str(" |");
            }
            md.push('\n');
        }

        if self.rows.len() > render_count || self.truncated {
            let total = if self.truncated {
                self.total_rows
            } else {
                self.rows.len()
            };
            let omitted = total.saturating_sub(render_count);
            md.push_str(&format!(
                "\n*... and {omitted} more rows omitted (total: {total})*\n"
            ));
        }

        md
    }

    /// Serializes tabular data to semantic HTML5 `<table>`.
    #[must_use]
    pub fn to_html(&self, max_rows: Option<usize>) -> String {
        if self.columns.is_empty() {
            return String::new();
        }

        let mut html = String::new();
        html.push_str("<table>\n");

        if self.has_headers {
            html.push_str("  <thead>\n    <tr>\n");
            for h in &self.headers {
                html.push_str(&format!("      <th>{}</th>\n", escape_html(h)));
            }
            html.push_str("    </tr>\n  </thead>\n");
        }

        html.push_str("  <tbody>\n");
        let limit = max_rows.unwrap_or(self.rows.len());
        let render_count = self.rows.len().min(limit);

        for row in &self.rows[..render_count] {
            html.push_str("    <tr>\n");
            for col_idx in 0..self.headers.len() {
                let cell_val = row.get(col_idx).map(|s| s.as_str()).unwrap_or("");
                html.push_str(&format!("      <td>{}</td>\n", escape_html(cell_val)));
            }
            html.push_str("    </tr>\n");
        }
        html.push_str("  </tbody>\n</table>\n");

        html
    }

    /// Serializes tabular data to RFC 4180 compliant CSV.
    #[must_use]
    pub fn to_csv(&self) -> String {
        let mut csv = String::new();
        let delim = self.delimiter.as_char();

        if self.has_headers {
            for (idx, h) in self.headers.iter().enumerate() {
                if idx > 0 {
                    csv.push(delim);
                }
                csv.push_str(&escape_delimited(h, delim));
            }
            csv.push('\n');
        }

        for row in &self.rows {
            for (idx, cell) in row.iter().enumerate() {
                if idx > 0 {
                    csv.push(delim);
                }
                csv.push_str(&escape_delimited(cell, delim));
            }
            csv.push('\n');
        }

        csv
    }

    /// Serializes tabular data to a JSON array of record objects `[{"col": "val"}, ...]`.
    #[must_use]
    pub fn to_json_records(&self, max_rows: Option<usize>) -> serde_json::Value {
        let limit = max_rows.unwrap_or(self.rows.len());
        let render_count = self.rows.len().min(limit);

        let mut array = Vec::with_capacity(render_count);
        for row in &self.rows[..render_count] {
            let mut obj = serde_json::Map::new();
            for (idx, header) in self.headers.iter().enumerate() {
                let val = row.get(idx).cloned().unwrap_or_default();
                obj.insert(header.clone(), serde_json::Value::String(val));
            }
            array.push(serde_json::Value::Object(obj));
        }

        serde_json::Value::Array(array)
    }

    /// Generates a rich Markdown summary describing dataset structure and column statistics.
    #[must_use]
    pub fn to_summary_markdown(&self) -> String {
        let mut md = String::new();
        let title = self.name.as_deref().unwrap_or("Tabular Dataset");
        md.push_str(&format!("# {title} Overview\n\n"));

        md.push_str(&format!(
            "- **Delimiter:** {}\n\
             - **Total Discovered Rows:** {}\n\
             - **Extracted Rows:** {}\n\
             - **Total Columns:** {}\n\
             - **Data Density:** {:.1}%\n\
             - **Headers Present:** {}\n\
             - **Truncated:** {}\n\n",
            self.delimiter.display_name(),
            self.total_rows,
            self.extracted_rows,
            self.columns.len(),
            self.data_density() * 100.0,
            self.has_headers,
            self.truncated,
        ));

        md.push_str("### Column Schema & Statistics\n\n");
        md.push_str("| Index | Column Name | Type | Non-Empty | Nulls | Unique | Samples |\n");
        md.push_str("| --- | --- | --- | --- | --- | --- | --- |\n");

        for col in &self.columns {
            let samples = if col.sample_values.is_empty() {
                "-".to_string()
            } else {
                col.sample_values
                    .iter()
                    .map(|s| sanitize_markdown_cell(s))
                    .collect::<Vec<_>>()
                    .join(", ")
            };

            md.push_str(&format!(
                "| {} | {} | `{}` | {} | {} | {} | {} |\n",
                col.index + 1,
                sanitize_markdown_cell(&col.name),
                col.data_type.as_str(),
                col.non_empty_count,
                col.null_count,
                col.unique_count,
                samples,
            ));
        }

        md
    }
}

/// Sanitizes a cell for Markdown table formatting.
fn sanitize_markdown_cell(text: &str) -> String {
    text.replace('\n', " ")
        .replace('\r', "")
        .replace('|', "\\|")
        .trim()
        .to_string()
}

/// Escapes HTML text.
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Escapes delimited CSV/TSV values per RFC 4180 rules.
fn escape_delimited(val: &str, delimiter: char) -> String {
    if val.contains(delimiter) || val.contains('"') || val.contains('\n') || val.contains('\r') {
        let escaped = val.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        val.to_string()
    }
}
