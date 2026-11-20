//! # Vector Schema Validator (`crates/chunking/src/export/validator.rs`)
//!
//! Validates chunks against the structural and typing constraints of target vector databases
//! before serialization, preventing silent ingestion failures or rejected bulk uploads.

use crate::error::ChunkingError;
use crate::export::schema::{is_valid_uuid, to_deterministic_uuid, VectorTarget};
use crate::model::Chunk;

/// Validator ensuring chunk payloads conform to target vector database schema constraints.
#[derive(Debug, Clone, Copy, Default)]
pub struct VectorSchemaValidator;

impl VectorSchemaValidator {
    /// Constructs a new schema validator.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Validates a single chunk against the constraints of the specified vector target.
    ///
    /// # Errors
    /// Returns [`ChunkingError::ValidationError`] if any target constraint is violated.
    pub fn validate_chunk(
        &self,
        chunk: &Chunk,
        target: VectorTarget,
        use_uuid: bool,
        weaviate_class: &str,
        embedding_model: &str,
    ) -> Result<(), ChunkingError> {
        // Universal validation
        if chunk.chunk_id.as_str().is_empty() {
            return Err(ChunkingError::ValidationError(
                "chunk ID must not be empty".to_string(),
            ));
        }
        if chunk.content.is_empty() {
            return Err(ChunkingError::ValidationError(format!(
                "chunk '{}' has empty content payload",
                chunk.chunk_id
            )));
        }
        if chunk.content_hash.is_empty() {
            return Err(ChunkingError::ValidationError(format!(
                "chunk '{}' has missing content_hash",
                chunk.chunk_id
            )));
        }

        // UUID validation when requested or mandatory
        if use_uuid || target == VectorTarget::Weaviate {
            let uuid = to_deterministic_uuid(chunk.chunk_id.as_str());
            if !is_valid_uuid(&uuid) {
                return Err(ChunkingError::ValidationError(format!(
                    "generated UUID '{uuid}' for chunk '{}' is not a valid RFC 4122 UUID",
                    chunk.chunk_id
                )));
            }
        }

        // Target-specific validation
        match target {
            VectorTarget::GenericJsonl => {
                if chunk.source_url.is_empty() {
                    return Err(ChunkingError::ValidationError(format!(
                        "chunk '{}' has empty source_url in generic JSONL export",
                        chunk.chunk_id
                    )));
                }
            }
            VectorTarget::Qdrant => {
                if use_uuid {
                    let id = to_deterministic_uuid(chunk.chunk_id.as_str());
                    if !is_valid_uuid(&id) {
                        return Err(ChunkingError::ValidationError(format!(
                            "Qdrant point ID '{id}' must be a valid UUID when uuid mode is enabled"
                        )));
                    }
                }
            }
            VectorTarget::Pinecone => {
                let id = chunk.chunk_id.as_str();
                if id.len() > 512 {
                    return Err(ChunkingError::ValidationError(format!(
                        "Pinecone vector ID for chunk '{id}' exceeds maximum length of 512 bytes (actual: {})",
                        id.len()
                    )));
                }
                if !id.is_ascii() {
                    return Err(ChunkingError::ValidationError(format!(
                        "Pinecone vector ID '{id}' must be valid ASCII"
                    )));
                }
            }
            VectorTarget::Chroma => {
                if chunk.content.trim().is_empty() {
                    return Err(ChunkingError::ValidationError(format!(
                        "Chroma document content for chunk '{}' cannot be whitespace-only",
                        chunk.chunk_id
                    )));
                }
            }
            VectorTarget::Milvus => {
                if chunk.chunk_id.as_str().len() > 1024 {
                    return Err(ChunkingError::ValidationError(format!(
                        "Milvus primary key ID for chunk '{}' exceeds 1024 bytes",
                        chunk.chunk_id
                    )));
                }
            }
            VectorTarget::Weaviate => {
                if weaviate_class.is_empty() {
                    return Err(ChunkingError::ValidationError(
                        "Weaviate class name cannot be empty".to_string(),
                    ));
                }
                let first_char = weaviate_class.chars().next().unwrap_or(' ');
                if !first_char.is_ascii_uppercase() {
                    return Err(ChunkingError::ValidationError(format!(
                        "Weaviate class name '{weaviate_class}' must start with an uppercase ASCII letter"
                    )));
                }
            }
            VectorTarget::OpenAiBatch => {
                let custom_id = chunk.chunk_id.as_str();
                if custom_id.len() > 64 {
                    return Err(ChunkingError::ValidationError(format!(
                        "OpenAI Batch custom_id '{custom_id}' exceeds maximum allowed length of 64 characters (actual: {})",
                        custom_id.len()
                    )));
                }
                if embedding_model.is_empty() {
                    return Err(ChunkingError::ValidationError(
                        "OpenAI Batch embedding model name cannot be empty".to_string(),
                    ));
                }
                // OpenAI max input context size for text-embedding-3-small/large is 8191 tokens (~32k chars)
                if chunk.token_count > 8191 {
                    return Err(ChunkingError::ValidationError(format!(
                        "chunk '{}' has {} tokens, exceeding OpenAI maximum embedding context of 8191 tokens",
                        chunk.chunk_id, chunk.token_count
                    )));
                }
            }
            VectorTarget::CohereBatch => {
                if chunk.content.trim().is_empty() {
                    return Err(ChunkingError::ValidationError(format!(
                        "Cohere embed text for chunk '{}' cannot be whitespace-only",
                        chunk.chunk_id
                    )));
                }
            }
        }

        Ok(())
    }
}
