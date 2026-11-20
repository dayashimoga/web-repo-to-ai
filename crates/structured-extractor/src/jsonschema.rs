//! # JSON Schema Parser and Converter
//!
//! Parses standalone JSON Schema specifications (Draft-04, Draft-07, Draft 2020-12, etc.)
//! from JSON or YAML, extracting root metadata, property tables, constraints, and sub-definitions.

use crate::error::{Result, StructuredError};
use crate::models::{JsonSchemaDoc, JsonSchemaProperty};
use serde_json::Value;

/// Parses a standalone JSON Schema definition from a JSON or YAML string.
pub fn parse_json_schema(text: &str) -> Result<JsonSchemaDoc> {
    let root: Value = if let Ok(val) = serde_json::from_str(text) {
        val
    } else {
        let yaml_val: serde_yaml_ng::Value = serde_yaml_ng::from_str(text).map_err(|e| {
            StructuredError::InvalidJsonSchema(format!("Failed to parse YAML: {e}"))
        })?;
        serde_json::to_value(yaml_val).map_err(|e| {
            StructuredError::InvalidJsonSchema(format!("Failed to convert YAML to JSON: {e}"))
        })?
    };

    let obj = root.as_object().ok_or_else(|| {
        StructuredError::InvalidJsonSchema("Root JSON Schema must be an object".to_string())
    })?;

    Ok(parse_schema_object(obj))
}

/// Recursively parses an object dictionary into `JsonSchemaDoc`.
fn parse_schema_object(obj: &serde_json::Map<String, Value>) -> JsonSchemaDoc {
    let schema_version = obj
        .get("$schema")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let id = obj
        .get("$id")
        .or_else(|| obj.get("id"))
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let title = obj
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let description = obj
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    let root_type = obj
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("object")
        .to_string();

    let required: Vec<String> = obj
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
            if let Some(p_dict) = prop_val.as_object() {
                let p_type = summarize_property_type(prop_val);
                let format = p_dict
                    .get("format")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let p_req = required.contains(prop_name);
                let p_desc = p_dict
                    .get("description")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);
                let default_val = p_dict.get("default").map(|d| match d {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                });

                let mut enum_values = Vec::new();
                if let Some(arr) = p_dict.get("enum").and_then(Value::as_array) {
                    for v in arr {
                        enum_values.push(match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        });
                    }
                }

                properties.push(JsonSchemaProperty {
                    name: prop_name.clone(),
                    prop_type: p_type,
                    format,
                    required: p_req,
                    description: p_desc,
                    default_val,
                    enum_values,
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

    let mut definitions = Vec::new();
    let defs_map = obj
        .get("$defs")
        .or_else(|| obj.get("definitions"))
        .and_then(Value::as_object);

    if let Some(defs) = defs_map {
        for (def_name, def_val) in defs {
            if let Some(d_dict) = def_val.as_object() {
                let mut sub_doc = parse_schema_object(d_dict);
                if sub_doc.title.is_none() {
                    sub_doc.title = Some(def_name.clone());
                }
                definitions.push(sub_doc);
            }
        }
    }

    // Sort properties alphabetically
    properties.sort_by(|a, b| a.name.cmp(&b.name));

    JsonSchemaDoc {
        schema_version,
        id,
        title,
        description,
        root_type,
        properties,
        required,
        enum_values,
        definitions,
    }
}

/// Summarizes the property type for a JSON Schema attribute.
fn summarize_property_type(val: &Value) -> String {
    if let Some(ref_str) = val.get("$ref").and_then(Value::as_str) {
        return ref_str
            .split('/')
            .next_back()
            .unwrap_or(ref_str)
            .to_string();
    }

    let type_str = val.get("type").and_then(Value::as_str).unwrap_or("any");

    if type_str == "array" {
        if let Some(items) = val.get("items") {
            let item_summary = summarize_property_type(items);
            return format!("Array<{item_summary}>");
        }
        return "Array".to_string();
    }

    type_str.to_string()
}
