//! # Schema Migration Pipeline and Version Evolution
//!
//! Sequential migration pipeline transforming legacy CDM document JSON representations
//! (`v0.8.0` prototype -> `v0.9.0` preview -> `v1.0.0` production) into fully compliant
//! modern [`CdmDocument`] structures without loss of source fidelity.

use crate::error::{CdmError, Result};
use crate::model::CdmDocument;
use crate::section::{CdmSection, SectionTokenMetrics};
use crate::version::SchemaVersion;
use serde_json::Value;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

/// Trait implemented by executable schema migration steps.
pub trait MigrationStep: std::fmt::Debug + Send + Sync {
    /// Originating schema version for this step.
    #[allow(clippy::wrong_self_convention)]
    fn from_version(&self) -> SchemaVersion;

    /// Destination schema version produced by this step.
    fn to_version(&self) -> SchemaVersion;

    /// Human-readable description of structural adaptations performed.
    fn description(&self) -> &'static str;

    /// Executes the migration transformation on a mutable JSON value.
    fn migrate(&self, value: Value) -> Result<Value>;
}

/// Migration step: `v0.8.0` (early prototype) -> `v0.9.0` (preview format).
///
/// Renames legacy `content_blocks` to `sections`, standardizes section IDs,
/// extracts `source_url`, and sets initial revision metadata.
#[derive(Debug, Default, Clone, Copy)]
pub struct Migration0_8To0_9;

impl MigrationStep for Migration0_8To0_9 {
    fn from_version(&self) -> SchemaVersion {
        SchemaVersion::v0_8_0()
    }

    fn to_version(&self) -> SchemaVersion {
        SchemaVersion::v0_9_0()
    }

    fn description(&self) -> &'static str {
        "Migrates CDM v0.8.0 content_blocks into normalized v0.9.0 sections and source identity"
    }

    fn migrate(&self, mut value: Value) -> Result<Value> {
        let obj = value
            .as_object_mut()
            .ok_or_else(|| CdmError::MigrationError {
                from: "0.8.0".to_string(),
                to: "0.9.0".to_string(),
                reason: "Root document must be a JSON object".to_string(),
            })?;

        // 1. Rename content_blocks -> sections if needed
        if let Some(blocks) = obj.remove("content_blocks") {
            obj.insert("sections".to_string(), blocks);
        }

        let canonical_url = obj
            .get("canonical_url")
            .and_then(Value::as_str)
            .or_else(|| obj.get("url").and_then(Value::as_str))
            .unwrap_or("")
            .to_string();

        // 2. Normalize sections
        if let Some(sections) = obj.get_mut("sections").and_then(Value::as_array_mut) {
            for (idx, sec_val) in sections.iter_mut().enumerate() {
                if let Some(sec) = sec_val.as_object_mut() {
                    // id / block_id -> section_id
                    if !sec.contains_key("section_id") {
                        let id = sec
                            .remove("block_id")
                            .or_else(|| sec.remove("id"))
                            .and_then(|v| v.as_str().map(ToString::to_string))
                            .unwrap_or_else(|| format!("sec-{idx}"));
                        sec.insert("section_id".to_string(), Value::String(id));
                    }

                    // text / content -> body
                    if !sec.contains_key("body") {
                        let body = sec
                            .remove("text")
                            .or_else(|| sec.remove("content"))
                            .and_then(|v| v.as_str().map(ToString::to_string))
                            .unwrap_or_default();
                        sec.insert("body".to_string(), Value::String(body));
                    }

                    // Ensure kind
                    if !sec.contains_key("kind") {
                        sec.insert("kind".to_string(), Value::String("prose".to_string()));
                    }

                    // Ensure source_url
                    if !sec.contains_key("source_url") {
                        let src = sec
                            .remove("url")
                            .and_then(|v| v.as_str().map(ToString::to_string))
                            .unwrap_or_else(|| canonical_url.clone());
                        sec.insert("source_url".to_string(), Value::String(src));
                    }

                    // Calculate content_hash if absent
                    if !sec.contains_key("content_hash") {
                        let body_str = sec.get("body").and_then(Value::as_str).unwrap_or("");
                        let hash = CdmSection::compute_hash(body_str);
                        sec.insert("content_hash".to_string(), Value::String(hash));
                    }
                }
            }
        }

        // 3. Normalize source_identity
        if !obj.contains_key("source_identity") {
            let mut id_obj = serde_json::Map::new();
            id_obj.insert(
                "raw_input".to_string(),
                Value::String(canonical_url.clone()),
            );
            id_obj.insert(
                "canonical_url".to_string(),
                Value::String(canonical_url.clone()),
            );
            id_obj.insert("domain".to_string(), Value::Null);
            id_obj.insert("repo_owner".to_string(), Value::Null);
            id_obj.insert("repo_name".to_string(), Value::Null);
            id_obj.insert("git_ref".to_string(), Value::Null);
            id_obj.insert("subpath".to_string(), Value::Null);
            obj.insert("source_identity".to_string(), Value::Object(id_obj));
        }

        // 4. Update schema_version
        obj.insert(
            "schema_version".to_string(),
            Value::String("0.9.0".to_string()),
        );

        Ok(value)
    }
}

/// Migration step: `v0.9.0` (preview format) -> `v1.0.0` (authoritative schema).
///
/// Populates completeness accounting reports, establishes hierarchy trees,
/// computes token metrics baselines, fills default exclusions/relationships arrays,
/// and sets schema version to `1.0.0`.
#[derive(Debug, Default, Clone, Copy)]
pub struct Migration0_9To1_0;

impl MigrationStep for Migration0_9To1_0 {
    fn from_version(&self) -> SchemaVersion {
        SchemaVersion::v0_9_0()
    }

    fn to_version(&self) -> SchemaVersion {
        SchemaVersion::v1_0_0()
    }

    fn description(&self) -> &'static str {
        "Migrates CDM v0.9.0 preview into production v1.0.0 with completeness invariant and hierarchy"
    }

    fn migrate(&self, mut value: Value) -> Result<Value> {
        let obj = value
            .as_object_mut()
            .ok_or_else(|| CdmError::MigrationError {
                from: "0.9.0".to_string(),
                to: "1.0.0".to_string(),
                reason: "Root document must be a JSON object".to_string(),
            })?;

        let canonical_url = obj
            .get("canonical_url")
            .and_then(Value::as_str)
            .unwrap_or("https://unknown.source/")
            .to_string();

        // 1. Ensure document_id (UUID v7 if missing or nil)
        if !obj.contains_key("document_id") {
            let id = Uuid::now_v7().to_string();
            obj.insert("document_id".to_string(), Value::String(id));
        }

        // 2. Sections normalization & token metrics derivation
        let mut section_count = 0;
        let mut section_ids: Vec<String> = Vec::new();

        if let Some(sections) = obj.get_mut("sections").and_then(Value::as_array_mut) {
            section_count = sections.len() as u64;
            for (idx, sec_val) in sections.iter_mut().enumerate() {
                if let Some(sec) = sec_val.as_object_mut() {
                    // Ensure section_id
                    let sec_id = sec
                        .get("section_id")
                        .and_then(Value::as_str)
                        .map(ToString::to_string)
                        .unwrap_or_else(|| format!("sec-{idx}"));
                    sec.insert("section_id".to_string(), Value::String(sec_id.clone()));
                    section_ids.push(sec_id);

                    // Ensure body
                    let body_str = sec
                        .get("body")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string();

                    // Ensure content_hash
                    if !sec.contains_key("content_hash")
                        || sec
                            .get("content_hash")
                            .and_then(Value::as_str)
                            .map_or(true, str::is_empty)
                    {
                        let hash = CdmSection::compute_hash(&body_str);
                        sec.insert("content_hash".to_string(), Value::String(hash));
                    }

                    // Derive baseline token_metrics if absent
                    if !sec.contains_key("token_metrics")
                        || sec.get("token_metrics") == Some(&Value::Null)
                    {
                        let baseline = SectionTokenMetrics::compute_baseline(&body_str);
                        let metrics_val =
                            serde_json::to_value(baseline).map_err(CdmError::Serialization)?;
                        sec.insert("token_metrics".to_string(), metrics_val);
                    }

                    // Ensure attributes map
                    if !sec.contains_key("attributes") {
                        sec.insert(
                            "attributes".to_string(),
                            Value::Object(serde_json::Map::new()),
                        );
                    }
                }
            }
        }

        // 3. Establish root hierarchy if missing
        if !obj.contains_key("hierarchy") || obj.get("hierarchy") == Some(&Value::Null) {
            let mut root = serde_json::Map::new();
            root.insert("id".to_string(), Value::String("root".to_string()));
            root.insert("name".to_string(), Value::String(canonical_url.clone()));
            root.insert("path".to_string(), Value::String(String::new()));
            root.insert("kind".to_string(), Value::String("root".to_string()));
            root.insert("section_id".to_string(), Value::Null);
            root.insert("byte_size".to_string(), Value::Null);
            root.insert("line_count".to_string(), Value::Null);

            // Add children referencing sections
            let mut children = Vec::new();
            for (idx, sec_id) in section_ids.iter().enumerate() {
                let mut child = serde_json::Map::new();
                child.insert("id".to_string(), Value::String(format!("node-{idx}")));
                child.insert("name".to_string(), Value::String(format!("section-{idx}")));
                child.insert("path".to_string(), Value::String(format!("sec-{idx}")));
                child.insert("kind".to_string(), Value::String("section".to_string()));
                child.insert("section_id".to_string(), Value::String(sec_id.clone()));
                child.insert("children".to_string(), Value::Array(Vec::new()));
                child.insert("byte_size".to_string(), Value::Null);
                child.insert("line_count".to_string(), Value::Null);
                children.push(Value::Object(child));
            }
            root.insert("children".to_string(), Value::Array(children));
            obj.insert("hierarchy".to_string(), Value::Object(root));
        }

        // 4. Populate CompletenessReport satisfying mathematical invariant
        if !obj.contains_key("completeness") || obj.get("completeness") == Some(&Value::Null) {
            let mut comp = serde_json::Map::new();
            comp.insert(
                "discovered_count".to_string(),
                Value::Number(section_count.into()),
            );
            comp.insert(
                "eligible_count".to_string(),
                Value::Number(section_count.into()),
            );
            comp.insert(
                "extracted_count".to_string(),
                Value::Number(section_count.into()),
            );
            comp.insert("excluded_count".to_string(), Value::Number(0.into()));
            comp.insert("failed_count".to_string(), Value::Number(0.into()));
            comp.insert("unsupported_count".to_string(), Value::Number(0.into()));
            obj.insert("completeness".to_string(), Value::Object(comp));
        }

        // 5. Ensure auxiliary arrays exist
        if !obj.contains_key("relationships") {
            obj.insert("relationships".to_string(), Value::Array(Vec::new()));
        }
        if !obj.contains_key("citations") {
            obj.insert("citations".to_string(), Value::Array(Vec::new()));
        }
        if !obj.contains_key("warnings") {
            obj.insert("warnings".to_string(), Value::Array(Vec::new()));
        }
        if !obj.contains_key("exclusions") {
            obj.insert("exclusions".to_string(), Value::Array(Vec::new()));
        }

        // 6. Ensure metadata exists
        if !obj.contains_key("metadata") {
            obj.insert(
                "metadata".to_string(),
                Value::Object(serde_json::Map::new()),
            );
        }

        // 7. Ensure revision info exists
        if !obj.contains_key("revision") {
            obj.insert(
                "revision".to_string(),
                Value::Object(serde_json::Map::new()),
            );
        }

        // 8. Ensure source coordinates and timestamps
        if !obj.contains_key("source_type") {
            obj.insert(
                "source_type".to_string(),
                Value::String("webpage_static".to_string()),
            );
        }
        if !obj.contains_key("content_type") {
            obj.insert(
                "content_type".to_string(),
                Value::String("text/plain".to_string()),
            );
        }
        if !obj.contains_key("retrieved_at") {
            obj.insert(
                "retrieved_at".to_string(),
                Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }
        if !obj.contains_key("canonical_url") {
            obj.insert(
                "canonical_url".to_string(),
                Value::String(canonical_url.clone()),
            );
        }
        if !obj.contains_key("source_identity") {
            let mut id_obj = serde_json::Map::new();
            id_obj.insert(
                "raw_input".to_string(),
                Value::String(canonical_url.clone()),
            );
            id_obj.insert(
                "canonical_url".to_string(),
                Value::String(canonical_url.clone()),
            );
            id_obj.insert("domain".to_string(), Value::Null);
            id_obj.insert("repo_owner".to_string(), Value::Null);
            id_obj.insert("repo_name".to_string(), Value::Null);
            id_obj.insert("git_ref".to_string(), Value::Null);
            id_obj.insert("subpath".to_string(), Value::Null);
            obj.insert("source_identity".to_string(), Value::Object(id_obj));
        }

        // 9. Ensure extraction_status is lowercase
        if let Some(status) = obj.get_mut("extraction_status") {
            if let Some(s) = status.as_str() {
                *status = Value::String(s.to_lowercase());
            }
        } else {
            obj.insert(
                "extraction_status".to_string(),
                Value::String("completed".to_string()),
            );
        }

        // 10. Update schema_version to authoritative 1.0.0
        obj.insert(
            "schema_version".to_string(),
            Value::String("1.0.0".to_string()),
        );

        Ok(value)
    }
}

/// Registry orchestrating sequential schema migration steps.
pub struct MigrationRegistry {
    steps: Vec<Box<dyn MigrationStep>>,
}

impl Default for MigrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl MigrationRegistry {
    /// Creates an empty migration registry populated with built-in standard steps.
    #[must_use]
    pub fn new() -> Self {
        let mut reg = Self { steps: Vec::new() };
        reg.register(Migration0_8To0_9);
        reg.register(Migration0_9To1_0);
        reg
    }

    /// Registers a new migration step.
    pub fn register<S: MigrationStep + 'static>(&mut self, step: S) {
        self.steps.push(Box::new(step));
    }

    /// Finds a sequential chain of migration steps connecting `from` to `to` using BFS.
    ///
    /// # Errors
    /// Returns [`CdmError::UnregisteredMigration`] if no continuous path exists.
    pub fn find_path(
        &self,
        from: &SchemaVersion,
        to: &SchemaVersion,
    ) -> Result<Vec<&dyn MigrationStep>> {
        if from == to {
            return Ok(Vec::new());
        }

        // Breadth-first search for the shortest migration chain
        let mut queue: VecDeque<(SchemaVersion, Vec<usize>)> = VecDeque::new();
        queue.push_back((from.clone(), Vec::new()));

        let mut visited: HashMap<SchemaVersion, usize> = HashMap::new();
        visited.insert(from.clone(), 0);

        while let Some((current_ver, path)) = queue.pop_front() {
            if current_ver == *to {
                return Ok(path.iter().map(|&idx| &*self.steps[idx]).collect());
            }

            for (idx, step) in self.steps.iter().enumerate() {
                if step.from_version() == current_ver {
                    let next_ver = step.to_version();
                    let next_cost = path.len() + 1;

                    if !visited.contains_key(&next_ver) || visited[&next_ver] > next_cost {
                        visited.insert(next_ver.clone(), next_cost);
                        let mut next_path = path.clone();
                        next_path.push(idx);
                        queue.push_back((next_ver, next_path));
                    }
                }
            }
        }

        Err(CdmError::UnregisteredMigration {
            from: from.to_string(),
            to: to.to_string(),
        })
    }

    /// Migrates a JSON [`serde_json::Value`] from its embedded schema version to `target_version`.
    ///
    /// If `target_version` is `None`, migrates to [`SchemaVersion::current()`] (`1.0.0`).
    ///
    /// # Errors
    /// Returns [`CdmError`] if version parsing, path discovery, or step migration fails.
    pub fn migrate_value(
        &self,
        mut value: Value,
        target_version: Option<&SchemaVersion>,
    ) -> Result<Value> {
        let current_target = SchemaVersion::current();
        let target = target_version.unwrap_or(&current_target);

        // Detect current schema version in payload
        let raw_version_str = value
            .get("schema_version")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                // Heuristic fallback for legacy unversioned payloads:
                if value.get("content_blocks").is_some() {
                    "0.8.0"
                } else {
                    "0.9.0"
                }
            });

        let from = SchemaVersion::parse(raw_version_str)?;

        if from == *target {
            return Ok(value);
        }

        if from > *target {
            return Err(CdmError::MigrationError {
                from: from.to_string(),
                to: target.to_string(),
                reason: "Downgrading schema versions is not supported".to_string(),
            });
        }

        let path = self.find_path(&from, target)?;
        for step in path {
            value = step.migrate(value)?;
        }

        Ok(value)
    }

    /// Migrates a JSON string representation to `target_version`.
    ///
    /// # Errors
    /// Returns [`CdmError`] if parsing or migration fails.
    pub fn migrate_json(
        &self,
        json_str: &str,
        target_version: Option<&SchemaVersion>,
    ) -> Result<Value> {
        let value: Value = serde_json::from_str(json_str).map_err(CdmError::Deserialization)?;
        self.migrate_value(value, target_version)
    }

    /// Migrates a JSON value and deserializes it directly into a [`CdmDocument`].
    ///
    /// # Errors
    /// Returns [`CdmError`] on migration or deserialization failure.
    pub fn migrate_and_deserialize(&self, value: Value) -> Result<CdmDocument> {
        let migrated_val = self.migrate_value(value, None)?;
        serde_json::from_value(migrated_val).map_err(CdmError::Deserialization)
    }
}

/// Convenience function migrating a JSON string to [`CdmDocument`] v1.0.0.
///
/// # Errors
/// Returns [`CdmError`] if migration or deserialization fails.
pub fn migrate_document_json(json_str: &str) -> Result<CdmDocument> {
    MigrationRegistry::default()
        .migrate_json(json_str, None)
        .and_then(|val| serde_json::from_value(val).map_err(CdmError::Deserialization))
}

/// Convenience function migrating a JSON [`serde_json::Value`] to [`CdmDocument`] v1.0.0.
///
/// # Errors
/// Returns [`CdmError`] if migration or deserialization fails.
pub fn migrate_document_value(value: Value) -> Result<CdmDocument> {
    MigrationRegistry::default().migrate_and_deserialize(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v0_8_to_v1_0_sequential_migration() {
        let legacy_json = serde_json::json!({
            "schema_version": "0.8.0",
            "url": "https://example.com/legacy",
            "canonical_url": "https://example.com/legacy",
            "content_blocks": [
                {
                    "block_id": "blk-1",
                    "text": "Hello legacy world\n",
                    "url": "https://example.com/legacy#1"
                }
            ]
        });

        let registry = MigrationRegistry::default();
        let migrated_doc = registry.migrate_and_deserialize(legacy_json).unwrap();

        assert_eq!(migrated_doc.schema_version, "1.0.0");
        assert_eq!(migrated_doc.sections.len(), 1);
        assert_eq!(migrated_doc.sections[0].section_id, "blk-1");
        assert_eq!(migrated_doc.sections[0].body, "Hello legacy world\n");
        assert!(!migrated_doc.sections[0].content_hash.is_empty());
        assert!(migrated_doc.completeness.formula_valid());
        assert_eq!(migrated_doc.completeness.discovered_count, 1);
        assert_eq!(migrated_doc.completeness.extracted_count, 1);

        // Verify that migrated document passes full CDM validator!
        let report = migrated_doc.validation_report();
        assert!(report.is_valid(), "Report issues: {:?}", report.issues);
    }

    #[test]
    fn test_v0_9_to_v1_0_direct_migration() {
        let v0_9_json = serde_json::json!({
            "schema_version": "0.9.0",
            "canonical_url": "https://example.com/preview",
            "source_type": "webpage_static",
            "retrieved_at": "2026-11-17T12:00:00Z",
            "content_type": "text/html",
            "source_identity": {
                "raw_input": "https://example.com/preview",
                "canonical_url": "https://example.com/preview",
                "domain": "example.com"
            },
            "sections": [
                {
                    "section_id": "sec-0",
                    "source_url": "https://example.com/preview/main",
                    "kind": "prose",
                    "body": "Preview document content\n"
                }
            ]
        });

        let registry = MigrationRegistry::default();
        let migrated = registry.migrate_and_deserialize(v0_9_json).unwrap();

        assert_eq!(migrated.schema_version, "1.0.0");
        assert_eq!(migrated.sections.len(), 1);
        assert_eq!(
            migrated.sections[0].content_hash,
            CdmSection::compute_hash("Preview document content\n")
        );
        assert!(migrated.sections[0].token_metrics.is_some());
        assert!(migrated.completeness.is_complete());
        assert_eq!(migrated.hierarchy.id, "root");
        assert_eq!(migrated.hierarchy.children.len(), 1);
        assert_eq!(
            migrated.hierarchy.children[0].section_id,
            Some("sec-0".to_string())
        );

        assert!(migrated.validate().is_ok());
    }

    #[test]
    fn test_unregistered_migration_error() {
        let registry = MigrationRegistry::default();
        let impossible_from = SchemaVersion::new(0, 5, 0);
        let target = SchemaVersion::new(1, 0, 0);

        let err = registry.find_path(&impossible_from, &target).unwrap_err();
        assert!(matches!(err, CdmError::UnregisteredMigration { .. }));
    }

    #[test]
    fn test_downgrade_migration_rejected() {
        let modern_json = serde_json::json!({
            "schema_version": "1.0.0",
            "canonical_url": "https://example.com/modern"
        });

        let registry = MigrationRegistry::default();
        let err = registry
            .migrate_value(modern_json, Some(&SchemaVersion::v0_9_0()))
            .unwrap_err();

        assert!(err
            .to_string()
            .contains("Downgrading schema versions is not supported"));
    }
}
