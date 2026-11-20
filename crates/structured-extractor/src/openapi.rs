//! # OpenAPI and Swagger Specification Parser and Converter
//!
//! Parses OpenAPI 2.0 (Swagger), 3.0.x, and 3.1.x specifications from JSON or YAML,
//! extracting paths, operations, parameters, request bodies, responses, and component schemas.

use crate::error::{Result, StructuredError};
use crate::models::{
    OpenApiOperation, OpenApiParameter, OpenApiPath, OpenApiProperty, OpenApiRequestBody,
    OpenApiResponse, OpenApiSchema, OpenApiSpec, OpenApiTag,
};
use serde_json::Value;

/// Parses an OpenAPI specification from a JSON or YAML string.
pub fn parse_openapi(text: &str) -> Result<OpenApiSpec> {
    // Attempt fast JSON parse, fallback to YAML
    let root: Value = if let Ok(val) = serde_json::from_str(text) {
        val
    } else {
        let yaml_val: serde_yaml_ng::Value = serde_yaml_ng::from_str(text)
            .map_err(|e| StructuredError::InvalidOpenApi(format!("Failed to parse YAML: {e}")))?;
        // Convert YAML value to JSON Value
        serde_json::to_value(yaml_val).map_err(|e| {
            StructuredError::InvalidOpenApi(format!("Failed to convert YAML to JSON: {e}"))
        })?
    };

    let obj = root
        .as_object()
        .ok_or_else(|| StructuredError::InvalidOpenApi("Root must be an object".to_string()))?;

    // Determine specification format version
    let spec_format_version = if let Some(v) = obj.get("openapi").and_then(Value::as_str) {
        v.to_string()
    } else if let Some(v) = obj.get("swagger").and_then(Value::as_str) {
        format!("Swagger {v}")
    } else {
        return Err(StructuredError::InvalidOpenApi(
            "Missing 'openapi' or 'swagger' version declaration".to_string(),
        ));
    };

    // Extract Info
    let info = obj.get("info").and_then(Value::as_object);
    let title = info
        .and_then(|i| i.get("title"))
        .and_then(Value::as_str)
        .unwrap_or("API Specification")
        .to_string();

    let api_version = info
        .and_then(|i| i.get("version"))
        .and_then(Value::as_str)
        .unwrap_or("1.0.0")
        .to_string();

    let description = info
        .and_then(|i| i.get("description"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    // Extract Servers / Host
    let mut servers = Vec::new();
    if let Some(srv_arr) = obj.get("servers").and_then(Value::as_array) {
        for s in srv_arr {
            if let Some(u) = s.get("url").and_then(Value::as_str) {
                let desc = s.get("description").and_then(Value::as_str);
                if let Some(d) = desc {
                    servers.push(format!("{u} ({d})"));
                } else {
                    servers.push(u.to_string());
                }
            }
        }
    } else if let Some(host) = obj.get("host").and_then(Value::as_str) {
        let base_path = obj.get("basePath").and_then(Value::as_str).unwrap_or("/");
        let schemes = obj.get("schemes").and_then(Value::as_array);
        let scheme = schemes
            .and_then(|s| s.first())
            .and_then(Value::as_str)
            .unwrap_or("https");
        servers.push(format!("{scheme}://{host}{base_path}"));
    }

    // Extract Tags
    let mut tags = Vec::new();
    if let Some(tag_arr) = obj.get("tags").and_then(Value::as_array) {
        for t in tag_arr {
            if let Some(name) = t.get("name").and_then(Value::as_str) {
                let desc = t
                    .get("description")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                tags.push(OpenApiTag {
                    name: name.to_string(),
                    description: desc,
                });
            }
        }
    }

    // Extract Paths and Operations
    let mut paths = Vec::new();
    if let Some(paths_obj) = obj.get("paths").and_then(Value::as_object) {
        for (path_str, path_item) in paths_obj {
            if let Some(path_obj) = path_item.as_object() {
                let path_item_params = parse_parameters(path_obj.get("parameters"));
                let mut operations = Vec::new();

                for (method, op_val) in path_obj {
                    let method_upper = method.to_uppercase();
                    if !is_http_method(&method_upper) {
                        continue;
                    }

                    if let Some(op_obj) = op_val.as_object() {
                        let op = parse_operation(
                            &method_upper,
                            op_obj,
                            &path_item_params,
                            spec_format_version.starts_with("Swagger"),
                        );
                        operations.push(op);
                    }
                }

                if !operations.is_empty() {
                    paths.push(OpenApiPath {
                        path: path_str.clone(),
                        operations,
                    });
                }
            }
        }
    }

    // Sort paths alphabetically
    paths.sort_by(|a, b| a.path.cmp(&b.path));

    // Extract Schemas / Definitions
    let mut schemas = Vec::new();
    let schemas_obj = obj
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(Value::as_object)
        .or_else(|| obj.get("definitions").and_then(Value::as_object));

    if let Some(s_obj) = schemas_obj {
        for (name, schema_val) in s_obj {
            if let Some(s_dict) = schema_val.as_object() {
                schemas.push(parse_schema(name, s_dict));
            }
        }
    }

    // Sort schemas alphabetically
    schemas.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(OpenApiSpec {
        spec_format_version,
        title,
        api_version,
        description,
        servers,
        paths,
        schemas,
        tags,
    })
}

/// Checks if string is a standard HTTP method.
fn is_http_method(m: &str) -> bool {
    matches!(
        m,
        "GET" | "POST" | "PUT" | "DELETE" | "PATCH" | "HEAD" | "OPTIONS" | "TRACE"
    )
}

/// Parses an individual operation.
fn parse_operation(
    method: &str,
    op_obj: &serde_json::Map<String, Value>,
    path_params: &[OpenApiParameter],
    is_swagger: bool,
) -> OpenApiOperation {
    let operation_id = op_obj
        .get("operationId")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let summary = op_obj
        .get("summary")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let description = op_obj
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let deprecated = op_obj
        .get("deprecated")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let tags = op_obj
        .get("tags")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    // Parameters
    let mut parameters = path_params.to_vec();
    if let Some(op_params) = op_obj.get("parameters") {
        for p in parse_parameters(Some(op_params)) {
            // Override by name & location if present
            if let Some(existing) = parameters
                .iter_mut()
                .find(|ex| ex.name == p.name && ex.location == p.location)
            {
                *existing = p;
            } else {
                parameters.push(p);
            }
        }
    }

    // Request Body
    let mut request_body = None;
    if is_swagger {
        // Swagger 2.0 defines body payload in parameters with in: "body"
        if let Some(body_param) = parameters.iter().find(|p| p.location == "body") {
            request_body = Some(OpenApiRequestBody {
                description: body_param.description.clone(),
                required: body_param.required,
                content_types: vec!["application/json".to_string()],
                schema_summary: Some(body_param.param_type.clone()),
            });
        }
        // Remove body from parameters list
        parameters.retain(|p| p.location != "body");
    } else if let Some(rb_obj) = op_obj.get("requestBody").and_then(Value::as_object) {
        let rb_desc = rb_obj
            .get("description")
            .and_then(Value::as_str)
            .map(ToString::to_string);
        let rb_req = rb_obj
            .get("required")
            .and_then(Value::as_bool)
            .unwrap_or(false);

        let mut content_types = Vec::new();
        let mut schema_summary = None;

        if let Some(content_obj) = rb_obj.get("content").and_then(Value::as_object) {
            for (ct, media_val) in content_obj {
                content_types.push(ct.clone());
                if schema_summary.is_none() {
                    if let Some(schema_val) = media_val.get("schema") {
                        schema_summary = Some(summarize_schema_ref_or_type(schema_val));
                    }
                }
            }
        }

        request_body = Some(OpenApiRequestBody {
            description: rb_desc,
            required: rb_req,
            content_types,
            schema_summary,
        });
    }

    // Responses
    let mut responses = Vec::new();
    if let Some(res_obj) = op_obj.get("responses").and_then(Value::as_object) {
        for (status, res_val) in res_obj {
            if let Some(r_dict) = res_val.as_object() {
                let desc = r_dict
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("No description provided")
                    .to_string();

                let mut content_types = Vec::new();
                let mut schema_summary = None;

                if let Some(c_obj) = r_dict.get("content").and_then(Value::as_object) {
                    for (ct, media_val) in c_obj {
                        content_types.push(ct.clone());
                        if schema_summary.is_none() {
                            if let Some(s) = media_val.get("schema") {
                                schema_summary = Some(summarize_schema_ref_or_type(s));
                            }
                        }
                    }
                } else if let Some(schema_val) = r_dict.get("schema") {
                    // Swagger 2.0 schema directly in response object
                    content_types.push("application/json".to_string());
                    schema_summary = Some(summarize_schema_ref_or_type(schema_val));
                }

                responses.push(OpenApiResponse {
                    status_code: status.clone(),
                    description: desc,
                    content_types,
                    schema_summary,
                });
            }
        }
    }
    // Sort responses: 2xx first, then 4xx, 5xx, default
    responses.sort_by(|a, b| a.status_code.cmp(&b.status_code));

    // Security requirements
    let mut security = Vec::new();
    if let Some(sec_arr) = op_obj.get("security").and_then(Value::as_array) {
        for item in sec_arr {
            if let Some(m) = item.as_object() {
                for k in m.keys() {
                    if !security.contains(k) {
                        security.push(k.clone());
                    }
                }
            }
        }
    }

    OpenApiOperation {
        method: method.to_string(),
        operation_id,
        summary,
        description,
        tags,
        deprecated,
        parameters,
        request_body,
        responses,
        security,
    }
}

/// Parses parameters list.
fn parse_parameters(params_val: Option<&Value>) -> Vec<OpenApiParameter> {
    let mut result = Vec::new();
    let arr = match params_val.and_then(Value::as_array) {
        Some(a) => a,
        None => return result,
    };

    for item in arr {
        if let Some(obj) = item.as_object() {
            let name = obj
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            if name.is_empty() {
                continue;
            }

            let location = obj
                .get("in")
                .and_then(Value::as_str)
                .unwrap_or("query")
                .to_string();
            let required = obj
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(location == "path");
            let description = obj
                .get("description")
                .and_then(Value::as_str)
                .map(ToString::to_string);
            let deprecated = obj
                .get("deprecated")
                .and_then(Value::as_bool)
                .unwrap_or(false);

            let param_type = if let Some(schema) = obj.get("schema") {
                let base_type = summarize_schema_ref_or_type(schema);
                if let Some(fmt) = schema.get("format").and_then(Value::as_str) {
                    format!("{base_type} ({fmt})")
                } else {
                    base_type
                }
            } else if let Some(t) = obj.get("type").and_then(Value::as_str) {
                if let Some(fmt) = obj.get("format").and_then(Value::as_str) {
                    format!("{t} ({fmt})")
                } else {
                    t.to_string()
                }
            } else {
                "string".to_string()
            };

            result.push(OpenApiParameter {
                name,
                location,
                required,
                param_type,
                description,
                deprecated,
            });
        }
    }

    result
}

/// Summarizes a schema reference or type into a concise string.
fn summarize_schema_ref_or_type(schema: &Value) -> String {
    if let Some(ref_str) = schema.get("$ref").and_then(Value::as_str) {
        return ref_str
            .split('/')
            .next_back()
            .unwrap_or(ref_str)
            .to_string();
    }

    let type_str = schema
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("object");

    if type_str == "array" {
        if let Some(items) = schema.get("items") {
            let item_summary = summarize_schema_ref_or_type(items);
            return format!("Array<{item_summary}>");
        }
        return "Array".to_string();
    }

    type_str.to_string()
}

/// Parses a component schema or definition into `OpenApiSchema`.
fn parse_schema(name: &str, obj: &serde_json::Map<String, Value>) -> OpenApiSchema {
    let schema_type = obj
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("object")
        .to_string();
    let description = obj
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let required_fields: Vec<String> = obj
        .get("required")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    let mut properties = Vec::new();
    if let Some(prop_map) = obj.get("properties").and_then(Value::as_object) {
        for (prop_name, prop_val) in prop_map {
            if let Some(p_obj) = prop_val.as_object() {
                let p_type = summarize_schema_ref_or_type(prop_val);
                let p_fmt = p_obj
                    .get("format")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let p_desc = p_obj
                    .get("description")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let p_req = required_fields.contains(prop_name);
                let p_ex = p_obj.get("example").map(|v| match v {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                });

                properties.push(OpenApiProperty {
                    name: prop_name.clone(),
                    prop_type: p_type,
                    format: p_fmt,
                    required: p_req,
                    description: p_desc,
                    example: p_ex,
                });
            }
        }
    }

    let mut enum_values = Vec::new();
    if let Some(arr) = obj.get("enum").and_then(Value::as_array) {
        for v in arr {
            enum_values.push(match v {
                Value::String(s) => s.clone(),
                other => other.to_string(),
            });
        }
    }

    OpenApiSchema {
        name: name.to_string(),
        schema_type,
        description,
        properties,
        required_fields,
        enum_values,
    }
}
