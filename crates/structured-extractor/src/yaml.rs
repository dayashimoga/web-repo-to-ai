//! # Generic YAML Document Stream Parser and Converter
//!
//! Parses single- and multi-document YAML configurations (Kubernetes manifests,
//! CI/CD workflows, Docker Compose files) into structured `YamlDocument` models.

use crate::error::{Result, StructuredError};
use crate::models::YamlDocument;
use serde::Deserialize;

/// Parses a YAML stream into one or more `YamlDocument` representations.
pub fn parse_yaml_stream(text: &str) -> Result<Vec<YamlDocument>> {
    let mut documents = Vec::new();

    // Use serde_yaml_ng deserializer to parse multi-document stream
    for (idx, document) in serde_yaml_ng::Deserializer::from_str(text).enumerate() {
        let value =
            serde_yaml_ng::Value::deserialize(document).map_err(StructuredError::YamlError)?;

        let (top_level_count, top_level_keys) = match &value {
            serde_yaml_ng::Value::Mapping(map) => {
                let keys: Vec<String> = map
                    .keys()
                    .map(|k| match k {
                        serde_yaml_ng::Value::String(s) => s.clone(),
                        other => format!("{other:?}"),
                    })
                    .collect();
                (map.len(), keys)
            }
            serde_yaml_ng::Value::Sequence(seq) => (seq.len(), Vec::new()),
            _ => (1, Vec::new()),
        };

        let clean_yaml = serde_yaml_ng::to_string(&value)
            .unwrap_or_else(|_| text.to_string())
            .trim()
            .to_string();

        documents.push(YamlDocument {
            document_index: idx + 1,
            top_level_count,
            top_level_keys,
            clean_yaml,
        });
    }

    if documents.is_empty() {
        return Err(StructuredError::UnrecognizedFormat(
            "No valid YAML documents found in input".to_string(),
        ));
    }

    Ok(documents)
}
