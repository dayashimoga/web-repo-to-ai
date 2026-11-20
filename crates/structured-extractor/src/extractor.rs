//! # Central Structured Document Extractor Engine
//!
//! Orchestrates format detection, spec parsing, section synthesis, metadata injection,
//! and Canonical Context Document Model (`CdmDocument`) assembly.

use std::collections::BTreeMap;

use crate::detector::detect_format;
use crate::error::{Result, StructuredError};
use crate::jsonschema::parse_json_schema;
use crate::models::{
    JsonSchemaDoc, JupyterNotebook, NotebookCellKind, NotebookOutputType, OpenApiOperation,
    OpenApiPath, OpenApiSpec, StructuredDocumentKind, StructuredInput, StructuredOptions,
    TabularTable, YamlDocument,
};
use crate::notebook::parse_notebook;
use crate::openapi::parse_openapi;
use crate::tabular::parse_tabular;
use crate::yaml::parse_yaml_stream;

use web_repo_to_ai_cdm::{
    CdmDocument, CdmDocumentBuilder, CdmSectionBuilder, CdmValidator, CompletenessReport,
    DocumentMetadata, ExtractionStatus, HierarchyKind, HierarchyNode, SectionKind, SourceIdentity,
    SourceType,
};

/// High-level structured document extractor synthesizing `CdmDocument` instances.
#[derive(Debug, Default, Clone)]
pub struct StructuredExtractor;

impl StructuredExtractor {
    /// Constructs a new extractor instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Extracts structured content from an input into a validated `CdmDocument`.
    pub fn extract_to_cdm(
        &self,
        input: &StructuredInput,
        options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        options.limits.validate_input(&input.bytes)?;
        let text = input.as_str()?;
        let kind = detect_format(input)?;

        let doc = match kind {
            StructuredDocumentKind::OpenApi => {
                let spec = parse_openapi(text)?;
                self.convert_openapi_to_cdm(&spec, input, options)?
            }
            StructuredDocumentKind::JsonSchema => {
                let schema = parse_json_schema(text)?;
                self.convert_jsonschema_to_cdm(&schema, input, options)?
            }
            StructuredDocumentKind::GenericYaml => {
                let docs = parse_yaml_stream(text)?;
                self.convert_yaml_to_cdm(&docs, input, options)?
            }
            StructuredDocumentKind::Csv => {
                let table_name = input.filename.clone();
                let table = parse_tabular(text, None, table_name, options)?;
                self.convert_tabular_to_cdm(&table, input, options)?
            }
            StructuredDocumentKind::JupyterNotebook => {
                let notebook = parse_notebook(text, options, &options.limits)?;
                self.convert_notebook_to_cdm(&notebook, input, options)?
            }
        };

        let validator = CdmValidator::new();
        let report = validator.validate(&doc);
        if !report.is_valid() {
            let err_msgs: Vec<String> = report.errors().map(|e| e.to_string()).collect();
            return Err(StructuredError::LimitExceeded(format!(
                "CDM validation failed: {}",
                err_msgs.join("; ")
            )));
        }

        Ok(doc)
    }

    // ========================================================================
    // OPENAPI TO CDM CONVERSION
    // ========================================================================

    fn convert_openapi_to_cdm(
        &self,
        spec: &OpenApiSpec,
        input: &StructuredInput,
        options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        let (source_identity, source_url) = build_source_identity(input, "openapi.yaml");

        let total_endpoints: usize = spec.paths.iter().map(|p| p.operations.len()).sum();

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert("structured_format".to_string(), "openapi".to_string());
        custom_meta.insert(
            "spec_format_version".to_string(),
            spec.spec_format_version.clone(),
        );
        custom_meta.insert("api_version".to_string(), spec.api_version.clone());
        custom_meta.insert("endpoint_count".to_string(), total_endpoints.to_string());
        custom_meta.insert("schema_count".to_string(), spec.schemas.len().to_string());

        if !spec.servers.is_empty() {
            custom_meta.insert("api_servers".to_string(), spec.servers.join(", "));
        }

        let metadata = DocumentMetadata {
            title: Some(spec.title.clone()),
            description: spec.description.clone(),
            authors: Vec::new(),
            license: None,
            primary_language: Some("en".to_string()),
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec![
                "openapi".to_string(),
                "api".to_string(),
                "specification".to_string(),
            ],
            extra: custom_meta,
        };

        let mut builder = CdmDocumentBuilder::new(
            source_identity,
            SourceType::StructuredDocument,
            source_url.clone(),
        )
        .content_type("application/yaml")
        .metadata(metadata);

        let mut root_node = HierarchyNode::root(&spec.title);
        let mut section_counter = 0;

        // 1. Overview Section
        section_counter += 1;
        let overview_sec_id = format!("sec_{section_counter:03}_overview");
        let mut overview_md = format!(
            "# {}\n\n**API Version:** `{}`  \n**Specification Standard:** `{}`  \n\n",
            spec.title, spec.api_version, spec.spec_format_version
        );

        if let Some(ref desc) = spec.description {
            overview_md.push_str(desc);
            overview_md.push_str("\n\n");
        }

        if !spec.servers.is_empty() {
            overview_md.push_str("### Base Servers\n\n");
            for s in &spec.servers {
                overview_md.push_str(&format!("- `{s}`\n"));
            }
            overview_md.push('\n');
        }

        let overview_lines = overview_md.lines().count().max(1) as u32;
        let overview_bytes = overview_md.len() as u64;

        let overview_sec = CdmSectionBuilder::new(
            overview_sec_id.clone(),
            source_url.clone(),
            SectionKind::Prose,
            overview_md,
        )
        .title(format!("{} - Overview", spec.title))
        .original_path("overview.md")
        .line_range(1, overview_lines)
        .byte_range(0, overview_bytes)
        .attribute("section_type", "overview");

        builder = builder.add_section(overview_sec.build());

        root_node.add_child(HierarchyNode {
            id: overview_sec_id.clone(),
            name: "Overview".to_string(),
            path: "overview".to_string(),
            kind: HierarchyKind::Section,
            section_id: Some(overview_sec_id),
            children: Vec::new(),
            byte_size: Some(overview_bytes),
            line_count: Some(overview_lines),
        });

        // 2. Operations / Endpoints
        let mut operations_by_tag: BTreeMap<String, Vec<(&OpenApiPath, &OpenApiOperation)>> =
            BTreeMap::new();
        for path in &spec.paths {
            for op in &path.operations {
                let tag = if options.group_by_tag && !op.tags.is_empty() {
                    op.tags[0].clone()
                } else {
                    "Endpoints".to_string()
                };
                operations_by_tag.entry(tag).or_default().push((path, op));
            }
        }

        for (tag_name, ops) in operations_by_tag {
            let mut tag_node = HierarchyNode {
                id: format!("tag_{tag_name}"),
                name: tag_name.clone(),
                path: format!("tags/{tag_name}"),
                kind: HierarchyKind::Directory,
                section_id: None,
                children: Vec::new(),
                byte_size: None,
                line_count: None,
            };

            for (path, op) in ops {
                section_counter += 1;
                let sec_id = format!("sec_{section_counter:03}_op");
                let op_title = format!("{} {}", op.method, path.path);

                let mut op_md = format!("## {}\n\n", op_title);
                if let Some(ref op_id) = op.operation_id {
                    op_md.push_str(&format!("**Operation ID:** `{op_id}`\n\n"));
                }
                if let Some(ref sum) = op.summary {
                    op_md.push_str(&format!("**Summary:** {sum}\n\n"));
                }
                if let Some(ref desc) = op.description {
                    op_md.push_str(&format!("{desc}\n\n"));
                }
                if op.deprecated {
                    op_md.push_str(
                        "> [!WARNING]\n> **Deprecated:** This endpoint operation is deprecated.\n\n",
                    );
                }

                // Parameters Table
                if options.include_parameters && !op.parameters.is_empty() {
                    op_md.push_str("### Parameters\n\n");
                    op_md.push_str("| Parameter | Location | Type | Required | Description |\n");
                    op_md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
                    for p in &op.parameters {
                        let req_str = if p.required { "Yes" } else { "No" };
                        let desc_str = p.description.as_deref().unwrap_or("-").replace('|', "\\|");
                        op_md.push_str(&format!(
                            "| `{}` | `{}` | `{}` | {} | {} |\n",
                            p.name, p.location, p.param_type, req_str, desc_str
                        ));
                    }
                    op_md.push('\n');
                }

                // Request Body
                if options.include_request_body {
                    if let Some(ref rb) = op.request_body {
                        op_md.push_str("### Request Body\n\n");
                        let req_str = if rb.required {
                            " (Required)"
                        } else {
                            " (Optional)"
                        };
                        op_md.push_str(&format!("**Payload:**{}\n\n", req_str));
                        if let Some(ref desc) = rb.description {
                            op_md.push_str(&format!("{desc}\n\n"));
                        }
                        if let Some(ref schema_sum) = rb.schema_summary {
                            op_md.push_str(&format!("- **Schema:** `{schema_sum}`\n"));
                        }
                        if !rb.content_types.is_empty() {
                            op_md.push_str(&format!(
                                "- **Content-Types:** `{}`\n",
                                rb.content_types.join(", ")
                            ));
                        }
                        op_md.push('\n');
                    }
                }

                // Responses
                if options.include_responses && !op.responses.is_empty() {
                    op_md.push_str("### Responses\n\n");
                    op_md.push_str("| Status Code | Description | Content-Type | Schema |\n");
                    op_md.push_str("| :--- | :--- | :--- | :--- |\n");
                    for r in &op.responses {
                        let ct_str = if r.content_types.is_empty() {
                            "-".to_string()
                        } else {
                            r.content_types.join(", ")
                        };
                        let s_str = r.schema_summary.as_deref().unwrap_or("-");
                        let desc_str = r.description.replace('|', "\\|");
                        op_md.push_str(&format!(
                            "| `{}` | {} | `{}` | `{}` |\n",
                            r.status_code, desc_str, ct_str, s_str
                        ));
                    }
                    op_md.push('\n');
                }

                let op_lines = op_md.lines().count().max(1) as u32;
                let op_bytes = op_md.len() as u64;

                let sec = CdmSectionBuilder::new(
                    sec_id.clone(),
                    source_url.clone(),
                    SectionKind::Prose,
                    op_md,
                )
                .title(&op_title)
                .original_path(format!("paths/{}.md", sanitize_path(&path.path)))
                .line_range(1, op_lines)
                .byte_range(0, op_bytes)
                .attribute("method", op.method.clone())
                .attribute("path", path.path.clone())
                .attribute("tag", tag_name.clone());

                builder = builder.add_section(sec.build());

                tag_node.add_child(HierarchyNode {
                    id: sec_id.clone(),
                    name: op_title,
                    path: format!("paths/{sec_id}"),
                    kind: HierarchyKind::Section,
                    section_id: Some(sec_id),
                    children: Vec::new(),
                    byte_size: Some(op_bytes),
                    line_count: Some(op_lines),
                });
            }

            root_node.add_child(tag_node);
        }

        // 3. Component Schemas / Data Models
        if options.include_schemas && !spec.schemas.is_empty() {
            let mut schemas_parent = HierarchyNode {
                id: "sec_schemas_root".to_string(),
                name: "Data Models & Schemas".to_string(),
                path: "schemas".to_string(),
                kind: HierarchyKind::Directory,
                section_id: None,
                children: Vec::new(),
                byte_size: None,
                line_count: None,
            };

            for schema in &spec.schemas {
                section_counter += 1;
                let sec_id = format!("sec_{section_counter:03}_schema");
                let model_title = format!("Model: {}", schema.name);

                let mut schema_md = format!("## {}\n\n", model_title);
                schema_md.push_str(&format!("**Type:** `{}`\n\n", schema.schema_type));
                if let Some(ref desc) = schema.description {
                    schema_md.push_str(&format!("{desc}\n\n"));
                }

                if !schema.properties.is_empty() {
                    schema_md.push_str("### Properties\n\n");
                    schema_md.push_str("| Field | Type | Required | Format | Description |\n");
                    schema_md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
                    for p in &schema.properties {
                        let req_str = if p.required { "Yes" } else { "No" };
                        let fmt_str = p.format.as_deref().unwrap_or("-");
                        let desc_str = p.description.as_deref().unwrap_or("-").replace('|', "\\|");
                        schema_md.push_str(&format!(
                            "| `{}` | `{}` | {} | `{}` | {} |\n",
                            p.name, p.prop_type, req_str, fmt_str, desc_str
                        ));
                    }
                    schema_md.push('\n');
                }

                if !schema.enum_values.is_empty() {
                    schema_md.push_str("### Allowed Enum Values\n\n");
                    for e in &schema.enum_values {
                        schema_md.push_str(&format!("- `{e}`\n"));
                    }
                    schema_md.push('\n');
                }

                let s_lines = schema_md.lines().count().max(1) as u32;
                let s_bytes = schema_md.len() as u64;

                let sec = CdmSectionBuilder::new(
                    sec_id.clone(),
                    source_url.clone(),
                    SectionKind::Prose,
                    schema_md,
                )
                .title(&model_title)
                .original_path(format!("schemas/{}.md", schema.name))
                .line_range(1, s_lines)
                .byte_range(0, s_bytes)
                .attribute("schema_name", schema.name.clone());

                builder = builder.add_section(sec.build());

                schemas_parent.add_child(HierarchyNode {
                    id: sec_id.clone(),
                    name: schema.name.clone(),
                    path: format!("schemas/{sec_id}"),
                    kind: HierarchyKind::Section,
                    section_id: Some(sec_id),
                    children: Vec::new(),
                    byte_size: Some(s_bytes),
                    line_count: Some(s_lines),
                });
            }

            root_node.add_child(schemas_parent);
        }

        let total_items = section_counter as u64;
        let completeness = CompletenessReport::new(total_items, total_items, total_items, 0, 0, 0);

        builder = builder
            .hierarchy(root_node)
            .completeness(completeness)
            .extraction_status(ExtractionStatus::Completed);

        Ok(builder.build())
    }

    // ========================================================================
    // JSON SCHEMA TO CDM CONVERSION
    // ========================================================================

    fn convert_jsonschema_to_cdm(
        &self,
        schema: &JsonSchemaDoc,
        input: &StructuredInput,
        _options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        let (source_identity, source_url) = build_source_identity(input, "schema.json");

        let title = schema
            .title
            .as_deref()
            .unwrap_or("JSON Schema Definition")
            .to_string();

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert("structured_format".to_string(), "json_schema".to_string());
        custom_meta.insert("root_type".to_string(), schema.root_type.clone());
        custom_meta.insert(
            "property_count".to_string(),
            schema.properties.len().to_string(),
        );
        custom_meta.insert(
            "definition_count".to_string(),
            schema.definitions.len().to_string(),
        );

        if let Some(ref v) = schema.schema_version {
            custom_meta.insert("schema_dialect".to_string(), v.clone());
        }
        if let Some(ref id) = schema.id {
            custom_meta.insert("schema_id".to_string(), id.clone());
        }

        let metadata = DocumentMetadata {
            title: Some(title.clone()),
            description: schema.description.clone(),
            authors: Vec::new(),
            license: None,
            primary_language: Some("json".to_string()),
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec!["json_schema".to_string(), "schema".to_string()],
            extra: custom_meta,
        };

        let mut builder = CdmDocumentBuilder::new(
            source_identity,
            SourceType::StructuredDocument,
            source_url.clone(),
        )
        .content_type("application/json")
        .metadata(metadata);

        let mut root_node = HierarchyNode::root(&title);
        let mut sec_counter = 0;

        // 1. Root Schema Overview & Properties
        sec_counter += 1;
        let root_sec_id = format!("sec_{sec_counter:03}_root");
        let mut root_md = format!("# {}\n\n**Type:** `{}`\n\n", title, schema.root_type);

        if let Some(ref desc) = schema.description {
            root_md.push_str(desc);
            root_md.push_str("\n\n");
        }

        if !schema.properties.is_empty() {
            root_md.push_str("### Properties\n\n");
            root_md.push_str("| Property | Type | Required | Format | Default | Description |\n");
            root_md.push_str("| :--- | :--- | :--- | :--- | :--- | :--- |\n");
            for p in &schema.properties {
                let req_str = if p.required { "Yes" } else { "No" };
                let fmt_str = p.format.as_deref().unwrap_or("-");
                let def_str = p.default_val.as_deref().unwrap_or("-");
                let desc_str = p.description.as_deref().unwrap_or("-").replace('|', "\\|");
                root_md.push_str(&format!(
                    "| `{}` | `{}` | {} | `{}` | `{}` | {} |\n",
                    p.name, p.prop_type, req_str, fmt_str, def_str, desc_str
                ));
            }
            root_md.push('\n');
        }

        if !schema.enum_values.is_empty() {
            root_md.push_str("### Allowed Values\n\n");
            for e in &schema.enum_values {
                root_md.push_str(&format!("- `{e}`\n"));
            }
            root_md.push('\n');
        }

        let root_lines = root_md.lines().count().max(1) as u32;
        let root_bytes = root_md.len() as u64;

        let root_sec = CdmSectionBuilder::new(
            root_sec_id.clone(),
            source_url.clone(),
            SectionKind::Prose,
            root_md,
        )
        .title(&title)
        .original_path("schema.md")
        .line_range(1, root_lines)
        .byte_range(0, root_bytes)
        .attribute("section_type", "root_schema");

        builder = builder.add_section(root_sec.build());

        root_node.add_child(HierarchyNode {
            id: root_sec_id.clone(),
            name: title.clone(),
            path: "schema".to_string(),
            kind: HierarchyKind::Section,
            section_id: Some(root_sec_id),
            children: Vec::new(),
            byte_size: Some(root_bytes),
            line_count: Some(root_lines),
        });

        // 2. Sub-definitions ($defs)
        for sub_def in &schema.definitions {
            sec_counter += 1;
            let def_sec_id = format!("sec_{sec_counter:03}_def");
            let def_title = sub_def
                .title
                .clone()
                .unwrap_or_else(|| "Sub-definition".to_string());

            let mut def_md = format!(
                "## Definition: {}\n\n**Type:** `{}`\n\n",
                def_title, sub_def.root_type
            );
            if let Some(ref d) = sub_def.description {
                def_md.push_str(&format!("{d}\n\n"));
            }

            if !sub_def.properties.is_empty() {
                def_md.push_str("### Properties\n\n");
                def_md.push_str("| Property | Type | Required | Format | Description |\n");
                def_md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
                for p in &sub_def.properties {
                    let req_str = if p.required { "Yes" } else { "No" };
                    let fmt_str = p.format.as_deref().unwrap_or("-");
                    let desc_str = p.description.as_deref().unwrap_or("-").replace('|', "\\|");
                    def_md.push_str(&format!(
                        "| `{}` | `{}` | {} | `{}` | {} |\n",
                        p.name, p.prop_type, req_str, fmt_str, desc_str
                    ));
                }
                def_md.push('\n');
            }

            let def_lines = def_md.lines().count().max(1) as u32;
            let def_bytes = def_md.len() as u64;

            let def_sec = CdmSectionBuilder::new(
                def_sec_id.clone(),
                source_url.clone(),
                SectionKind::Prose,
                def_md,
            )
            .title(format!("Definition: {def_title}"))
            .original_path(format!("defs/{def_title}.md"))
            .line_range(1, def_lines)
            .byte_range(0, def_bytes)
            .attribute("definition", def_title.clone());

            builder = builder.add_section(def_sec.build());

            root_node.add_child(HierarchyNode {
                id: def_sec_id.clone(),
                name: def_title,
                path: format!("defs/{def_sec_id}"),
                kind: HierarchyKind::Section,
                section_id: Some(def_sec_id),
                children: Vec::new(),
                byte_size: Some(def_bytes),
                line_count: Some(def_lines),
            });
        }

        let total_items = sec_counter as u64;
        let completeness = CompletenessReport::new(total_items, total_items, total_items, 0, 0, 0);

        builder = builder
            .hierarchy(root_node)
            .completeness(completeness)
            .extraction_status(ExtractionStatus::Completed);

        Ok(builder.build())
    }

    // ========================================================================
    // GENERIC YAML TO CDM CONVERSION
    // ========================================================================

    fn convert_yaml_to_cdm(
        &self,
        docs: &[YamlDocument],
        input: &StructuredInput,
        _options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        let (source_identity, source_url) = build_source_identity(input, "config.yaml");

        let title = input
            .filename
            .as_deref()
            .unwrap_or("YAML Configuration Document")
            .to_string();

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert("structured_format".to_string(), "yaml".to_string());
        custom_meta.insert("document_count".to_string(), docs.len().to_string());

        let metadata = DocumentMetadata {
            title: Some(title.clone()),
            description: Some(format!(
                "YAML document containing {} document(s)",
                docs.len()
            )),
            authors: Vec::new(),
            license: None,
            primary_language: Some("yaml".to_string()),
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec!["yaml".to_string(), "configuration".to_string()],
            extra: custom_meta,
        };

        let mut builder = CdmDocumentBuilder::new(
            source_identity,
            SourceType::StructuredDocument,
            source_url.clone(),
        )
        .content_type("text/yaml")
        .metadata(metadata);

        let mut root_node = HierarchyNode::root(&title);

        for (i, doc) in docs.iter().enumerate() {
            let sec_id = format!("sec_{:03}_yaml_doc", i + 1);
            let doc_title = if docs.len() > 1 {
                format!("Document #{}", doc.document_index)
            } else {
                "Configuration Content".to_string()
            };

            let mut sec_md = format!("## {}\n\n", doc_title);
            if !doc.top_level_keys.is_empty() {
                sec_md.push_str(&format!(
                    "**Top-Level Keys ({}):** `{}`\n\n",
                    doc.top_level_count,
                    doc.top_level_keys.join("`, `")
                ));
            }
            sec_md.push_str("```yaml\n");
            sec_md.push_str(&doc.clean_yaml);
            sec_md.push_str("\n```\n");

            let doc_lines = sec_md.lines().count().max(1) as u32;
            let doc_bytes = sec_md.len() as u64;

            let sec = CdmSectionBuilder::new(
                sec_id.clone(),
                source_url.clone(),
                SectionKind::CodeSnippet,
                sec_md,
            )
            .title(&doc_title)
            .original_path(format!("doc_{}.yaml", doc.document_index))
            .line_range(1, doc_lines)
            .byte_range(0, doc_bytes)
            .attribute("yaml_index", doc.document_index.to_string());

            builder = builder.add_section(sec.build());

            root_node.add_child(HierarchyNode {
                id: sec_id.clone(),
                name: doc_title,
                path: format!("doc_{}", doc.document_index),
                kind: HierarchyKind::Section,
                section_id: Some(sec_id),
                children: Vec::new(),
                byte_size: Some(doc_bytes),
                line_count: Some(doc_lines),
            });
        }

        let total_items = docs.len() as u64;
        let completeness = CompletenessReport::new(total_items, total_items, total_items, 0, 0, 0);

        builder = builder
            .hierarchy(root_node)
            .completeness(completeness)
            .extraction_status(ExtractionStatus::Completed);

        Ok(builder.build())
    }

    /// Converts a parsed `TabularTable` into a rich, validated `CdmDocument`.
    pub fn convert_tabular_to_cdm(
        &self,
        table: &TabularTable,
        input: &StructuredInput,
        options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        let (source_identity, source_url) = build_source_identity(input, "dataset.csv");
        let title = table
            .name
            .clone()
            .or_else(|| input.filename.clone())
            .unwrap_or_else(|| "Tabular Dataset".to_string());

        let mut extra_meta = BTreeMap::new();
        extra_meta.insert("format".to_string(), "tabular".to_string());
        extra_meta.insert(
            "tabular_delimiter".to_string(),
            table.delimiter.display_name().to_string(),
        );
        extra_meta.insert(
            "tabular_delimiter_char".to_string(),
            table.delimiter.as_char().to_string(),
        );
        extra_meta.insert(
            "tabular_total_rows".to_string(),
            table.total_rows.to_string(),
        );
        extra_meta.insert(
            "tabular_extracted_rows".to_string(),
            table.extracted_rows.to_string(),
        );
        extra_meta.insert(
            "tabular_columns".to_string(),
            table.columns.len().to_string(),
        );
        extra_meta.insert(
            "tabular_has_header".to_string(),
            table.has_headers.to_string(),
        );
        extra_meta.insert("tabular_truncated".to_string(), table.truncated.to_string());
        extra_meta.insert(
            "tabular_data_density".to_string(),
            format!("{:.4}", table.data_density()),
        );

        let metadata = DocumentMetadata {
            title: Some(title.clone()),
            description: Some(format!(
                "Tabular dataset containing {} columns and {} rows",
                table.columns.len(),
                table.extracted_rows
            )),
            authors: Vec::new(),
            license: None,
            primary_language: Some(match table.delimiter {
                crate::models::TabularDelimiter::Tab => "tsv".to_string(),
                _ => "csv".to_string(),
            }),
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec!["csv".to_string(), "tabular".to_string()],
            extra: extra_meta,
        };

        let mut builder = CdmDocumentBuilder::new(
            source_identity,
            SourceType::StructuredDocument,
            source_url.clone(),
        )
        .content_type(match table.delimiter {
            crate::models::TabularDelimiter::Tab => "text/tab-separated-values",
            _ => "text/csv",
        })
        .metadata(metadata);

        let mut root_node = HierarchyNode::root(&title);
        let mut sec_counter = 0;

        // 1. Overview & Schema Summary Section
        if options.tabular_include_summary_section {
            sec_counter += 1;
            let summary_id = format!("sec_{:03}_summary", sec_counter);
            let summary_md = table.to_summary_markdown();
            let summary_lines = summary_md.lines().count().max(1) as u32;
            let summary_bytes = summary_md.len() as u64;

            let sec = CdmSectionBuilder::new(
                summary_id.clone(),
                source_url.clone(),
                SectionKind::Prose,
                summary_md,
            )
            .title(format!("{title} - Summary & Schema"))
            .original_path("summary.md")
            .line_range(1, summary_lines)
            .byte_range(0, summary_bytes)
            .attribute("content_type", "summary")
            .attribute("tabular_columns", table.columns.len().to_string())
            .attribute("tabular_extracted_rows", table.extracted_rows.to_string());

            builder = builder.add_section(sec.build());

            root_node.add_child(HierarchyNode {
                id: summary_id.clone(),
                name: "Summary & Schema".to_string(),
                path: "summary.md".to_string(),
                kind: HierarchyKind::Section,
                section_id: Some(summary_id),
                children: Vec::new(),
                byte_size: Some(summary_bytes),
                line_count: Some(summary_lines),
            });
        }

        // 2. Data Table Section
        if options.tabular_include_data_section {
            sec_counter += 1;
            let data_id = format!("sec_{:03}_data", sec_counter);
            let data_md = table.to_markdown(Some(options.tabular_max_preview_rows));
            let data_lines = data_md.lines().count().max(1) as u32;
            let data_bytes = data_md.len() as u64;

            let sec = CdmSectionBuilder::new(
                data_id.clone(),
                source_url.clone(),
                SectionKind::Table,
                data_md,
            )
            .title(format!("{title} - Data Table"))
            .original_path("data.md")
            .line_range(1, data_lines)
            .byte_range(0, data_bytes)
            .attribute("content_type", "table/markdown")
            .attribute("tabular_columns", table.columns.len().to_string())
            .attribute("tabular_extracted_rows", table.extracted_rows.to_string())
            .attribute("tabular_total_rows", table.total_rows.to_string());

            builder = builder.add_section(sec.build());

            root_node.add_child(HierarchyNode {
                id: data_id.clone(),
                name: "Data Table".to_string(),
                path: "data.md".to_string(),
                kind: HierarchyKind::Section,
                section_id: Some(data_id),
                children: Vec::new(),
                byte_size: Some(data_bytes),
                line_count: Some(data_lines),
            });
        }

        let total_items = table.total_rows as u64;
        let extracted_items = table.extracted_rows as u64;
        let excluded_items = total_items.saturating_sub(extracted_items);

        let completeness = CompletenessReport::new(
            total_items,
            extracted_items,
            extracted_items,
            excluded_items,
            0,
            0,
        );

        let status = if table.truncated {
            ExtractionStatus::Partial
        } else {
            ExtractionStatus::Completed
        };

        builder = builder
            .hierarchy(root_node)
            .completeness(completeness)
            .extraction_status(status);

        Ok(builder.build())
    }

    // ========================================================================
    // JUPYTER NOTEBOOK TO CDM CONVERSION
    // ========================================================================

    fn convert_notebook_to_cdm(
        &self,
        notebook: &JupyterNotebook,
        input: &StructuredInput,
        options: &StructuredOptions,
    ) -> Result<CdmDocument> {
        let (source_identity, source_url) = build_source_identity(input, "notebook.ipynb");

        // Infer title: filename hint, or first heading in notebook, or fallback
        let first_heading = notebook.cells.iter().find_map(|c| {
            if c.cell_type == NotebookCellKind::Markdown {
                extract_heading(&c.source).map(|(_, h)| h)
            } else {
                None
            }
        });

        let title = input
            .filename
            .as_ref()
            .map(|f| f.trim_end_matches(".ipynb").to_string())
            .or(first_heading)
            .unwrap_or_else(|| "Jupyter Notebook".to_string());

        let mut custom_meta = BTreeMap::new();
        custom_meta.insert(
            "structured_format".to_string(),
            "jupyter_notebook".to_string(),
        );
        custom_meta.insert(
            "nbformat".to_string(),
            format!(
                "{}.{}",
                notebook.metadata.nbformat, notebook.metadata.nbformat_minor
            ),
        );
        if let Some(ref kernel) = notebook.metadata.kernel_name {
            custom_meta.insert("kernel_name".to_string(), kernel.clone());
        }
        if let Some(ref kernel_disp) = notebook.metadata.kernel_display_name {
            custom_meta.insert("kernel_display_name".to_string(), kernel_disp.clone());
        }
        custom_meta.insert(
            "language".to_string(),
            notebook.primary_language().to_string(),
        );
        if let Some(ref ver) = notebook.metadata.language_version {
            custom_meta.insert("language_version".to_string(), ver.clone());
        }
        custom_meta.insert(
            "total_cells".to_string(),
            notebook.total_cells_discovered.to_string(),
        );
        custom_meta.insert(
            "code_cells".to_string(),
            notebook.code_cells_count().to_string(),
        );
        custom_meta.insert(
            "markdown_cells".to_string(),
            notebook.markdown_cells_count().to_string(),
        );
        custom_meta.insert(
            "raw_cells".to_string(),
            notebook.raw_cells_count().to_string(),
        );

        let metadata = DocumentMetadata {
            title: Some(title.clone()),
            description: Some(format!(
                "Jupyter Notebook with {} cells ({} code, {} markdown)",
                notebook.cells.len(),
                notebook.code_cells_count(),
                notebook.markdown_cells_count(),
            )),
            authors: Vec::new(),
            license: None,
            primary_language: Some(notebook.primary_language().to_string()),
            language_breakdown: BTreeMap::new(),
            published_at: None,
            modified_at: None,
            tags: vec![
                "jupyter".to_string(),
                "notebook".to_string(),
                notebook.primary_language().to_string(),
            ],
            extra: custom_meta,
        };

        let mut builder = CdmDocumentBuilder::new(
            source_identity,
            SourceType::StructuredDocument,
            source_url.clone(),
        )
        .content_type("application/x-ipynb+json")
        .metadata(metadata);

        let mut sec_counter = 0;
        let mut stack: Vec<(usize, HierarchyNode)> = vec![(0, HierarchyNode::root(&title))];

        // 1. Overview & Summary Section
        sec_counter += 1;
        let summary_id = format!("sec_{:03}_summary", sec_counter);
        let summary_md = notebook.to_summary_markdown();
        let summary_lines = summary_md.lines().count().max(1) as u32;
        let summary_bytes = summary_md.len() as u64;

        let summary_sec = CdmSectionBuilder::new(
            summary_id.clone(),
            source_url.clone(),
            SectionKind::Prose,
            summary_md,
        )
        .title(format!("{title} - Summary & Overview"))
        .original_path("summary.md")
        .line_range(1, summary_lines)
        .byte_range(0, summary_bytes)
        .attribute("content_type", "summary")
        .attribute(
            "notebook_total_cells",
            notebook.total_cells_discovered.to_string(),
        )
        .attribute(
            "notebook_code_cells",
            notebook.code_cells_count().to_string(),
        )
        .attribute(
            "notebook_markdown_cells",
            notebook.markdown_cells_count().to_string(),
        );

        builder = builder.add_section(summary_sec.build());

        stack[0].1.add_child(HierarchyNode {
            id: summary_id.clone(),
            name: "Notebook Summary & Overview".to_string(),
            path: "summary.md".to_string(),
            kind: HierarchyKind::Section,
            section_id: Some(summary_id),
            children: Vec::new(),
            byte_size: Some(summary_bytes),
            line_count: Some(summary_lines),
        });

        // 2. Cell Sections
        for cell in &notebook.cells {
            sec_counter += 1;
            let cell_num = cell.index + 1;
            let cell_sec_id = format!("sec_{:03}_cell_{}", sec_counter, cell.index);
            let cell_path = format!(
                "cell_{cell_num}.{}",
                match cell.cell_type {
                    NotebookCellKind::Markdown => "md",
                    NotebookCellKind::Code => match notebook.primary_language() {
                        "python" => "py",
                        "r" => "r",
                        "julia" => "jl",
                        _ => "txt",
                    },
                    NotebookCellKind::Raw => "raw",
                }
            );

            match cell.cell_type {
                NotebookCellKind::Markdown => {
                    let heading_opt = extract_heading(&cell.source);
                    let cell_title = heading_opt
                        .as_ref()
                        .map(|(_, h)| h.clone())
                        .unwrap_or_else(|| format!("{title} - Cell {cell_num} [Markdown]"));
                    let body = if cell.source.is_empty() {
                        "*(Empty Markdown Cell)*".to_string()
                    } else {
                        cell.source.clone()
                    };
                    let lines = body.lines().count().max(1) as u32;
                    let bytes = body.len() as u64;

                    let sec = CdmSectionBuilder::new(
                        cell_sec_id.clone(),
                        source_url.clone(),
                        SectionKind::Prose,
                        body,
                    )
                    .title(cell_title.clone())
                    .original_path(cell_path.clone())
                    .line_range(1, lines)
                    .byte_range(0, bytes)
                    .attribute("cell_type", "markdown")
                    .attribute("cell_index", cell.index.to_string());

                    builder = builder.add_section(sec.build());

                    let node = HierarchyNode {
                        id: cell_sec_id.clone(),
                        name: cell_title,
                        path: cell_path,
                        kind: if heading_opt.is_some() {
                            HierarchyKind::Heading
                        } else {
                            HierarchyKind::Section
                        },
                        section_id: Some(cell_sec_id),
                        children: Vec::new(),
                        byte_size: Some(bytes),
                        line_count: Some(lines),
                    };

                    if let Some((level, _)) = heading_opt {
                        while stack.len() > 1 && stack.last().unwrap().0 >= level {
                            let (_, child) = stack.pop().unwrap();
                            stack.last_mut().unwrap().1.add_child(child);
                        }
                        stack.push((level, node));
                    } else {
                        stack.last_mut().unwrap().1.add_child(node);
                    }
                }
                NotebookCellKind::Code => {
                    let exec_tag = cell
                        .execution_count
                        .map(|c| format!(" [{c}]"))
                        .unwrap_or_default();
                    let cell_title = format!("{title} - Cell {cell_num} [Code{exec_tag}]");
                    let body = if cell.source.is_empty() {
                        "# (Empty Code Cell)\n".to_string()
                    } else {
                        cell.source.clone()
                    };
                    let lines = body.lines().count().max(1) as u32;
                    let bytes = body.len() as u64;

                    let mut sec_builder = CdmSectionBuilder::new(
                        cell_sec_id.clone(),
                        source_url.clone(),
                        SectionKind::CodeSnippet,
                        body,
                    )
                    .title(cell_title.clone())
                    .original_path(cell_path.clone())
                    .language(notebook.primary_language())
                    .line_range(1, lines)
                    .byte_range(0, bytes)
                    .attribute("cell_type", "code")
                    .attribute("cell_index", cell.index.to_string());

                    if let Some(exec) = cell.execution_count {
                        sec_builder = sec_builder.attribute("execution_count", exec.to_string());
                    }

                    builder = builder.add_section(sec_builder.build());

                    let mut code_node = HierarchyNode {
                        id: cell_sec_id.clone(),
                        name: cell_title,
                        path: cell_path,
                        kind: HierarchyKind::Section,
                        section_id: Some(cell_sec_id),
                        children: Vec::new(),
                        byte_size: Some(bytes),
                        line_count: Some(lines),
                    };

                    // Output section if outputs present and enabled
                    if options.notebook_include_outputs && !cell.outputs.is_empty() {
                        let mut out_md = String::new();
                        for output in &cell.outputs {
                            let text = output.to_plaintext();
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                let lines: Vec<&str> = trimmed.lines().collect();
                                let max_lines = options.notebook_max_output_lines;
                                let displayed = if lines.len() > max_lines {
                                    let mut subset = lines[..max_lines].join("\n");
                                    subset.push_str(&format!(
                                        "\n... ({} lines truncated)",
                                        lines.len() - max_lines
                                    ));
                                    subset
                                } else {
                                    trimmed.to_string()
                                };

                                let prefix = match output.output_type {
                                    NotebookOutputType::Stream => {
                                        let name =
                                            output.stream_name.as_deref().unwrap_or("output");
                                        format!("**Output ({name}):**\n")
                                    }
                                    NotebookOutputType::ExecuteResult => {
                                        let count = output
                                            .execution_count
                                            .map(|c| format!(" [{c}]"))
                                            .unwrap_or_default();
                                        format!("**Result{count}:**\n")
                                    }
                                    NotebookOutputType::DisplayData => "**Display:**\n".to_string(),
                                    NotebookOutputType::Error => {
                                        "**Error Traceback:**\n".to_string()
                                    }
                                };
                                out_md.push_str(&prefix);
                                out_md.push_str("```text\n");
                                out_md.push_str(&displayed);
                                out_md.push_str("\n```\n\n");
                            }
                        }

                        if !out_md.trim().is_empty() {
                            sec_counter += 1;
                            let out_sec_id =
                                format!("sec_{:03}_cell_{}_out", sec_counter, cell.index);
                            let out_path = format!("cell_{cell_num}_output.txt");
                            let out_title = format!("{title} - Cell {cell_num} Output");
                            let out_lines = out_md.lines().count().max(1) as u32;
                            let out_bytes = out_md.len() as u64;

                            let out_sec = CdmSectionBuilder::new(
                                out_sec_id.clone(),
                                source_url.clone(),
                                SectionKind::Prose,
                                out_md,
                            )
                            .title(out_title.clone())
                            .original_path(out_path.clone())
                            .line_range(1, out_lines)
                            .byte_range(0, out_bytes)
                            .attribute("cell_type", "output")
                            .attribute("cell_index", cell.index.to_string());

                            builder = builder.add_section(out_sec.build());

                            let out_node = HierarchyNode {
                                id: out_sec_id.clone(),
                                name: out_title,
                                path: out_path,
                                kind: HierarchyKind::Section,
                                section_id: Some(out_sec_id),
                                children: Vec::new(),
                                byte_size: Some(out_bytes),
                                line_count: Some(out_lines),
                            };

                            code_node.add_child(out_node);
                        }
                    }

                    stack.last_mut().unwrap().1.add_child(code_node);
                }
                NotebookCellKind::Raw => {
                    if options.notebook_include_raw_cells {
                        let cell_title = format!("{title} - Cell {cell_num} [Raw]");
                        let body = if cell.source.is_empty() {
                            "*(Empty Raw Cell)*".to_string()
                        } else {
                            cell.source.clone()
                        };
                        let lines = body.lines().count().max(1) as u32;
                        let bytes = body.len() as u64;

                        let sec = CdmSectionBuilder::new(
                            cell_sec_id.clone(),
                            source_url.clone(),
                            SectionKind::Prose,
                            body,
                        )
                        .title(cell_title.clone())
                        .original_path(cell_path.clone())
                        .line_range(1, lines)
                        .byte_range(0, bytes)
                        .attribute("cell_type", "raw")
                        .attribute("cell_index", cell.index.to_string());

                        builder = builder.add_section(sec.build());

                        let raw_node = HierarchyNode {
                            id: cell_sec_id.clone(),
                            name: cell_title,
                            path: cell_path,
                            kind: HierarchyKind::Section,
                            section_id: Some(cell_sec_id),
                            children: Vec::new(),
                            byte_size: Some(bytes),
                            line_count: Some(lines),
                        };

                        stack.last_mut().unwrap().1.add_child(raw_node);
                    }
                }
            }
        }

        // Unwind any remaining headings on the stack
        while stack.len() > 1 {
            let (_, child) = stack.pop().unwrap();
            stack.last_mut().unwrap().1.add_child(child);
        }
        let root_node = stack.pop().unwrap().1;

        let total_items = notebook.total_cells_discovered as u64;
        let extracted_items = notebook.cells.len() as u64;
        let excluded_items = total_items.saturating_sub(extracted_items);

        let completeness = CompletenessReport::new(
            total_items,
            extracted_items,
            extracted_items,
            excluded_items,
            0,
            0,
        );

        let status = if notebook.was_truncated {
            ExtractionStatus::Partial
        } else {
            ExtractionStatus::Completed
        };

        builder = builder
            .hierarchy(root_node)
            .completeness(completeness)
            .extraction_status(status);

        Ok(builder.build())
    }
}

/// Helper building `SourceIdentity` and canonical URL from `StructuredInput`.
fn build_source_identity(
    input: &StructuredInput,
    default_filename: &str,
) -> (SourceIdentity, String) {
    let source_url = input
        .source_url
        .clone()
        .or_else(|| input.filename.as_ref().map(|f| format!("file://{f}")))
        .unwrap_or_else(|| format!("file://{default_filename}"));

    let mut source_identity = SourceIdentity::new(source_url.clone(), source_url.clone());
    if let Ok(parsed) = url::Url::parse(&source_url) {
        source_identity.domain = parsed.domain().map(ToString::to_string);
    }

    (source_identity, source_url)
}

/// Extracts heading level (1..=6) and title from Markdown content if present.
fn extract_heading(source: &str) -> Option<(usize, String)> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let hashes = trimmed.chars().take_while(|&c| c == '#').count();
            if (1..=6).contains(&hashes) {
                let rest = trimmed[hashes..].trim();
                if !rest.is_empty() {
                    return Some((hashes, rest.to_string()));
                }
            }
        }
    }
    None
}

/// Sanitizes a URL path for using as a relative filename.
fn sanitize_path(path: &str) -> String {
    path.trim_start_matches('/')
        .replace('/', "_")
        .replace(['{', '}'], "")
}
