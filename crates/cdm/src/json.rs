//! # Canonical JSON Serialization and Deserialization Engine
//!
//! Provides deterministic, bounded, and verified JSON encoding/decoding
//! for the Canonical Context Document Model (`CdmDocument`).

use crate::error::{CdmError, Result};
use crate::model::CdmDocument;
use std::io::{Read, Write};

/// Default maximum allowed JSON payload size (64 MiB) to guard against unbounded memory exhaustion.
pub const DEFAULT_MAX_JSON_BYTES: usize = 64 * 1024 * 1024;

/// Configuration options controlling CDM JSON serialization and deserialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JsonOptions {
    /// Formats JSON output with 2-space indentation and line breaks when serializing.
    pub pretty: bool,
    /// Maximum byte ceiling permitted during deserialization.
    pub max_bytes: usize,
    /// Strictly verifies section BLAKE3 cryptographic digests during deserialization.
    pub verify_hashes: bool,
    /// Strictly validates that the schema version string matches supported versions.
    pub validate_version: bool,
    /// Automatically detects and up-migrates legacy schemas (e.g. `v0.8.0`, `v0.9.0`) to current version.
    pub auto_migrate: bool,
}

impl Default for JsonOptions {
    fn default() -> Self {
        Self {
            pretty: false,
            max_bytes: DEFAULT_MAX_JSON_BYTES,
            verify_hashes: false,
            validate_version: true,
            auto_migrate: false,
        }
    }
}

impl JsonOptions {
    /// Constructs options with pretty-printing enabled.
    #[must_use]
    pub const fn pretty() -> Self {
        Self {
            pretty: true,
            max_bytes: DEFAULT_MAX_JSON_BYTES,
            verify_hashes: false,
            validate_version: true,
            auto_migrate: false,
        }
    }

    /// Constructs options with compact formatting enabled.
    #[must_use]
    pub const fn compact() -> Self {
        Self {
            pretty: false,
            max_bytes: DEFAULT_MAX_JSON_BYTES,
            verify_hashes: false,
            validate_version: true,
            auto_migrate: false,
        }
    }

    /// Overrides the maximum allowed byte ceiling for deserialization.
    #[must_use]
    pub const fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    /// Toggles strict section cryptographic hash verification.
    #[must_use]
    pub const fn with_verify_hashes(mut self, verify: bool) -> Self {
        self.verify_hashes = verify;
        self
    }

    /// Toggles strict schema version verification.
    #[must_use]
    pub const fn with_validate_version(mut self, validate: bool) -> Self {
        self.validate_version = validate;
        self
    }

    /// Toggles automatic migration of legacy schema representations.
    #[must_use]
    pub const fn with_auto_migrate(mut self, auto: bool) -> Self {
        self.auto_migrate = auto;
        self
    }
}

/// Serializes a [`CdmDocument`] to a multi-line formatted JSON string with 2-space indentation.
///
/// # Errors
/// Returns [`CdmError::Serialization`] if serde encounters an encoding error.
pub fn to_json_pretty(doc: &CdmDocument) -> Result<String> {
    serde_json::to_string_pretty(doc).map_err(CdmError::Serialization)
}

/// Serializes a [`CdmDocument`] to a compact single-line JSON string.
///
/// # Errors
/// Returns [`CdmError::Serialization`] if serde encounters an encoding error.
pub fn to_json_compact(doc: &CdmDocument) -> Result<String> {
    serde_json::to_string(doc).map_err(CdmError::Serialization)
}

/// Streams a serialized compact [`CdmDocument`] into any standard writer.
///
/// # Errors
/// Returns [`CdmError::Serialization`] on encoding failure or [`CdmError::Io`] on write failure.
pub fn to_writer<W: Write>(doc: &CdmDocument, writer: W) -> Result<()> {
    serde_json::to_writer(writer, doc).map_err(CdmError::Serialization)
}

/// Streams a serialized pretty-printed [`CdmDocument`] into any standard writer.
///
/// # Errors
/// Returns [`CdmError::Serialization`] on encoding failure or [`CdmError::Io`] on write failure.
pub fn to_writer_pretty<W: Write>(doc: &CdmDocument, writer: W) -> Result<()> {
    serde_json::to_writer_pretty(writer, doc).map_err(CdmError::Serialization)
}

/// Deserializes a [`CdmDocument`] from a JSON string using default options.
///
/// Enforces default payload ceiling (64 MiB) and schema version checking.
///
/// # Errors
/// Returns [`CdmError::PayloadTooLarge`] if string length exceeds ceiling,
/// [`CdmError::Deserialization`] on invalid syntax, or [`CdmError::UnsupportedSchemaVersion`].
pub fn from_json(json_str: &str) -> Result<CdmDocument> {
    from_json_with_options(json_str, &JsonOptions::default())
}

/// Deserializes a [`CdmDocument`] from a JSON string using custom options.
///
/// # Errors
/// Returns [`CdmError::PayloadTooLarge`] if string length exceeds ceiling,
/// [`CdmError::Deserialization`] on invalid syntax, [`CdmError::UnsupportedSchemaVersion`],
/// or [`CdmError::HashMismatch`] if hash verification is enabled and fails.
pub fn from_json_with_options(json_str: &str, options: &JsonOptions) -> Result<CdmDocument> {
    let actual_bytes = json_str.len();
    if actual_bytes > options.max_bytes {
        return Err(CdmError::PayloadTooLarge {
            actual_bytes,
            max_bytes: options.max_bytes,
        });
    }

    if options.auto_migrate {
        let value: serde_json::Value =
            serde_json::from_str(json_str).map_err(CdmError::Deserialization)?;
        let doc = crate::migration::MigrationRegistry::default().migrate_and_deserialize(value)?;
        post_deserialize_validate(&doc, options)?;
        return Ok(doc);
    }

    let doc: CdmDocument = serde_json::from_str(json_str).map_err(CdmError::Deserialization)?;
    post_deserialize_validate(&doc, options)?;
    Ok(doc)
}

/// Deserializes a [`CdmDocument`] from a byte slice using default options.
///
/// # Errors
/// Returns [`CdmError`] if payload bounds, schema version, or JSON syntax are invalid.
pub fn from_slice(slice: &[u8]) -> Result<CdmDocument> {
    from_slice_with_options(slice, &JsonOptions::default())
}

/// Deserializes a [`CdmDocument`] from a byte slice using custom options.
///
/// # Errors
/// Returns [`CdmError`] if payload bounds, schema version, or JSON syntax are invalid.
pub fn from_slice_with_options(slice: &[u8], options: &JsonOptions) -> Result<CdmDocument> {
    let actual_bytes = slice.len();
    if actual_bytes > options.max_bytes {
        return Err(CdmError::PayloadTooLarge {
            actual_bytes,
            max_bytes: options.max_bytes,
        });
    }

    if options.auto_migrate {
        let value: serde_json::Value =
            serde_json::from_slice(slice).map_err(CdmError::Deserialization)?;
        let doc = crate::migration::MigrationRegistry::default().migrate_and_deserialize(value)?;
        post_deserialize_validate(&doc, options)?;
        return Ok(doc);
    }

    let doc: CdmDocument = serde_json::from_slice(slice).map_err(CdmError::Deserialization)?;
    post_deserialize_validate(&doc, options)?;
    Ok(doc)
}

/// Deserializes a [`CdmDocument`] from an arbitrary reader using default options.
///
/// Bounds read consumption to `DEFAULT_MAX_JSON_BYTES` to prevent memory exhaustion.
///
/// # Errors
/// Returns [`CdmError`] on payload ceiling breach, I/O error, or invalid JSON.
pub fn from_reader<R: Read>(reader: R) -> Result<CdmDocument> {
    from_reader_with_options(reader, &JsonOptions::default())
}

/// Deserializes a [`CdmDocument`] from an arbitrary reader using custom options.
///
/// Reads up to `options.max_bytes + 1` bytes into memory. If the reader exceeds
/// `max_bytes`, returns [`CdmError::PayloadTooLarge`].
///
/// # Errors
/// Returns [`CdmError::PayloadTooLarge`] if stream exceeds maximum bounds,
/// [`CdmError::Io`] on read failure, or [`CdmError::Deserialization`].
pub fn from_reader_with_options<R: Read>(reader: R, options: &JsonOptions) -> Result<CdmDocument> {
    let mut buffer = Vec::new();
    // Read up to limit + 1 to detect overflow without allocating unbounded memory
    let read_limit = (options.max_bytes as u64).saturating_add(1);
    let mut take = reader.take(read_limit);
    take.read_to_end(&mut buffer)?;

    if buffer.len() > options.max_bytes {
        return Err(CdmError::PayloadTooLarge {
            actual_bytes: buffer.len(),
            max_bytes: options.max_bytes,
        });
    }

    if options.auto_migrate {
        let value: serde_json::Value =
            serde_json::from_slice(&buffer).map_err(CdmError::Deserialization)?;
        let doc = crate::migration::MigrationRegistry::default().migrate_and_deserialize(value)?;
        post_deserialize_validate(&doc, options)?;
        return Ok(doc);
    }

    let doc: CdmDocument = serde_json::from_slice(&buffer).map_err(CdmError::Deserialization)?;
    post_deserialize_validate(&doc, options)?;
    Ok(doc)
}

fn post_deserialize_validate(doc: &CdmDocument, options: &JsonOptions) -> Result<()> {
    if options.validate_version {
        let expected_major = crate::SCHEMA_VERSION
            .split('.')
            .next()
            .unwrap_or(crate::SCHEMA_VERSION);
        let found_major = doc
            .schema_version
            .split('.')
            .next()
            .unwrap_or(&doc.schema_version);

        if found_major != expected_major {
            return Err(CdmError::UnsupportedSchemaVersion {
                found: doc.schema_version.clone(),
                supported: crate::SCHEMA_VERSION.to_string(),
            });
        }
    }

    if options.verify_hashes {
        for section in &doc.sections {
            if !section.verify_hash() {
                return Err(CdmError::HashMismatch {
                    section_id: section.section_id.clone(),
                    expected: section.content_hash.clone(),
                    actual: crate::section::CdmSection::compute_hash(&section.body),
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::{CdmDocumentBuilder, CdmSectionBuilder};
    use crate::model::{SourceIdentity, SourceType};
    use crate::section::SectionKind;

    fn sample_document() -> CdmDocument {
        let identity = SourceIdentity::new("https://example.com/test", "https://example.com/test");
        let sec = CdmSectionBuilder::new(
            "sec-json",
            "https://example.com/test#main",
            SectionKind::Prose,
            "Sample text for JSON serialization testing.",
        )
        .title("Test Section")
        .build();

        CdmDocumentBuilder::new(
            identity,
            SourceType::WebpageStatic,
            "https://example.com/test",
        )
        .add_section(sec)
        .build()
    }

    #[test]
    fn test_to_json_pretty_and_compact_roundtrip() {
        let doc = sample_document();

        let pretty = to_json_pretty(&doc).expect("pretty serialization succeeds");
        assert!(pretty.contains('\n'));
        assert!(pretty.contains("  \"schema_version\": \"1.0.0\""));

        let compact = to_json_compact(&doc).expect("compact serialization succeeds");
        assert!(!compact.contains('\n'));

        let from_pretty = from_json(&pretty).expect("deserialization from pretty succeeds");
        let from_compact = from_json(&compact).expect("deserialization from compact succeeds");

        assert_eq!(doc, from_pretty);
        assert_eq!(doc, from_compact);
    }

    #[test]
    fn test_payload_too_large_rejection() {
        let doc = sample_document();
        let json_str = to_json_compact(&doc).unwrap();

        let options = JsonOptions::default().with_max_bytes(50);
        let err = from_json_with_options(&json_str, &options);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), CdmError::PayloadTooLarge { .. }));
    }

    #[test]
    fn test_streaming_writer_and_reader() {
        let doc = sample_document();
        let mut buffer = Vec::new();

        to_writer_pretty(&doc, &mut buffer).expect("writer serialization succeeds");
        assert!(!buffer.is_empty());

        let deserialized = from_reader(&buffer[..]).expect("reader deserialization succeeds");
        assert_eq!(doc, deserialized);
    }

    #[test]
    fn test_hash_verification_option_detects_tamper() {
        let doc = sample_document();
        let mut json_val: serde_json::Value = serde_json::to_value(&doc).unwrap();

        // Tamper with section body in raw JSON
        json_val["sections"][0]["body"] = serde_json::json!("Tampered text content!");

        let tampered_json = serde_json::to_string(&json_val).unwrap();

        // Without verification, deserializes ok
        let normal = from_json(&tampered_json);
        assert!(normal.is_ok());

        // With verification enabled, errors on hash mismatch
        let options = JsonOptions::default().with_verify_hashes(true);
        let verified = from_json_with_options(&tampered_json, &options);
        assert!(verified.is_err());
        assert!(matches!(
            verified.unwrap_err(),
            CdmError::HashMismatch { .. }
        ));
    }

    #[test]
    fn test_unsupported_version_rejection() {
        let doc = sample_document();
        let mut json_val: serde_json::Value = serde_json::to_value(&doc).unwrap();
        json_val["schema_version"] = serde_json::json!("2.0.0");

        let json_str = serde_json::to_string(&json_val).unwrap();
        let err = from_json(&json_str);
        assert!(err.is_err());
        assert!(matches!(
            err.unwrap_err(),
            CdmError::UnsupportedSchemaVersion { .. }
        ));
    }
}
