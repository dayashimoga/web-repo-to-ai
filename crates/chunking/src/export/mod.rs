//! # Chunk Export & Vector Ingest Module (`crates/chunking/src/export/`)
//!
//! Provides streaming JSONL and vector database ingest schema exports for [`Chunk`] collections.
//!
//! Supports:
//! - **Generic RAG JSONL**: Standard newline-delimited JSON Lines records.
//! - **Qdrant**: Points with payload and optional deterministic RFC 4122 UUIDs.
//! - **Pinecone**: Vectors with flattened metadata conforming to Pinecone's strict type rules.
//! - **Chroma**: Document records and collection batch dictionaries.
//! - **Milvus**: Tabular entity rows with source coordinates and content hashes.
//! - **Weaviate**: Data objects with deterministic RFC 4122 UUIDs and class properties.
//! - **OpenAI Batch Embeddings**: `/v1/embeddings` batch request records.
//! - **Cohere Embed**: Batch records with document metadata.

pub mod schema;
pub mod validator;

use std::io::Write;

use serde::{Deserialize, Serialize};

use crate::error::ChunkingError;
use crate::model::Chunk;
pub use schema::{
    is_valid_uuid, to_deterministic_uuid, ChromaBatchCollection, ChromaRecord, CohereRecord,
    GenericRagRecord, MilvusEntity, OpenAiBatchBody, OpenAiBatchRequest, PineconeBatch,
    PineconeRecord, QdrantBatch, QdrantPoint, VectorTarget, WeaviateBatch, WeaviateObject,
};
pub use validator::VectorSchemaValidator;

/// Configuration options controlling chunk export and vector serialization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkExportOptions {
    /// Target vector database or ingestion format.
    pub target: VectorTarget,
    /// Whether to force IDs to be RFC 4122 deterministic UUIDs (mandatory for Weaviate).
    pub uuid_ids: bool,
    /// Weaviate class name (default: `"DocumentChunk"`).
    pub weaviate_class: String,
    /// Embedding model name for OpenAI Batch (default: `"text-embedding-3-small"`).
    pub embedding_model: String,
    /// Whether to export as a single JSON batch envelope (e.g. `{"points": [...]}`) instead of JSONL.
    pub batch_envelope: bool,
    /// Whether to perform schema validation prior to serializing each chunk.
    pub validate_schema: bool,
    /// Optional maximum cumulative token budget across exported chunks.
    pub max_token_budget: Option<usize>,
}

impl Default for ChunkExportOptions {
    fn default() -> Self {
        Self {
            target: VectorTarget::GenericJsonl,
            uuid_ids: false,
            weaviate_class: "DocumentChunk".to_string(),
            embedding_model: "text-embedding-3-small".to_string(),
            batch_envelope: false,
            validate_schema: true,
            max_token_budget: None,
        }
    }
}

impl ChunkExportOptions {
    /// Constructs default export options targeting Generic RAG JSONL.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target vector database or export format.
    #[must_use]
    pub const fn with_target(mut self, target: VectorTarget) -> Self {
        self.target = target;
        self
    }

    /// Sets whether to use deterministic RFC 4122 UUIDs for point/record IDs.
    #[must_use]
    pub const fn with_uuid_ids(mut self, uuid_ids: bool) -> Self {
        self.uuid_ids = uuid_ids;
        self
    }

    /// Sets the Weaviate class name.
    #[must_use]
    pub fn with_weaviate_class(mut self, class: impl Into<String>) -> Self {
        self.weaviate_class = class.into();
        self
    }

    /// Sets the OpenAI embedding model name.
    #[must_use]
    pub fn with_embedding_model(mut self, model: impl Into<String>) -> Self {
        self.embedding_model = model.into();
        self
    }

    /// Sets whether to serialize as a batch envelope object rather than line-delimited JSONL.
    #[must_use]
    pub const fn with_batch_envelope(mut self, batch: bool) -> Self {
        self.batch_envelope = batch;
        self
    }

    /// Sets whether to run schema validation before serialization.
    #[must_use]
    pub const fn with_validate_schema(mut self, validate: bool) -> Self {
        self.validate_schema = validate;
        self
    }

    /// Sets an optional maximum token budget cap across all exported chunks.
    #[must_use]
    pub const fn with_max_token_budget(mut self, budget: Option<usize>) -> Self {
        self.max_token_budget = budget;
        self
    }
}

/// Telemetry metrics recorded during chunk export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChunkExportMetrics {
    /// Number of chunks successfully exported.
    pub chunks_exported: usize,
    /// Total bytes written to the sink.
    pub bytes_written: usize,
    /// Total lines written (in JSONL mode).
    pub lines_written: usize,
    /// Total token count across exported chunks.
    pub tokens_exported: usize,
    /// Whether export was halted early by a token budget cap.
    pub truncated: bool,
}

/// Production exporter serializing chunks into streaming JSONL and vector ingest schemas.
#[derive(Debug, Clone, Default)]
pub struct ChunkExporter {
    options: ChunkExportOptions,
    validator: VectorSchemaValidator,
}

impl ChunkExporter {
    /// Creates a new chunk exporter with default options.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new chunk exporter with custom options.
    #[must_use]
    pub fn with_options(options: ChunkExportOptions) -> Self {
        Self {
            options,
            validator: VectorSchemaValidator::new(),
        }
    }

    /// Exports chunks directly to any destination implementing [`std::io::Write`].
    ///
    /// # Errors
    /// Returns [`ChunkingError`] if validation fails, serialization fails, or an I/O error occurs.
    pub fn export_to_writer<W: Write>(
        &self,
        chunks: &[Chunk],
        writer: &mut W,
    ) -> Result<ChunkExportMetrics, ChunkingError> {
        if self.options.batch_envelope {
            return self.export_batch_to_writer(chunks, writer);
        }

        let mut metrics = ChunkExportMetrics::default();
        let mut cumulative_tokens = 0usize;

        for chunk in chunks {
            if let Some(budget) = self.options.max_token_budget {
                if cumulative_tokens.saturating_add(chunk.token_count) > budget {
                    metrics.truncated = true;
                    break;
                }
            }

            if self.options.validate_schema {
                self.validator.validate_chunk(
                    chunk,
                    self.options.target,
                    self.options.uuid_ids,
                    &self.options.weaviate_class,
                    &self.options.embedding_model,
                )?;
            }

            let line = self.serialize_chunk_record(chunk)?;
            let bytes = line.as_bytes();
            writer.write_all(bytes)?;
            writer.write_all(b"\n")?;

            metrics.bytes_written += bytes.len() + 1;
            metrics.lines_written += 1;
            metrics.chunks_exported += 1;
            metrics.tokens_exported += chunk.token_count;
            cumulative_tokens += chunk.token_count;
        }

        writer.flush()?;
        Ok(metrics)
    }

    /// Exports chunks to an in-memory string.
    ///
    /// # Errors
    /// Returns [`ChunkingError`] on validation or formatting failure.
    pub fn export_to_string(
        &self,
        chunks: &[Chunk],
    ) -> Result<(String, ChunkExportMetrics), ChunkingError> {
        let mut buffer = Vec::with_capacity(chunks.len().saturating_mul(512));
        let metrics = self.export_to_writer(chunks, &mut buffer)?;
        let output = String::from_utf8(buffer)
            .map_err(|e| ChunkingError::ExportError(format!("UTF-8 encoding failure: {e}")))?;
        Ok((output, metrics))
    }

    /// Exports chunks as a single batch JSON envelope (e.g. `{"points": [...]}`).
    ///
    /// # Errors
    /// Returns [`ChunkingError`] on validation or serialization failure.
    pub fn export_batch_json(&self, chunks: &[Chunk]) -> Result<String, ChunkingError> {
        let mut buffer = Vec::with_capacity(chunks.len().saturating_mul(512));
        self.export_batch_to_writer(chunks, &mut buffer)?;
        String::from_utf8(buffer)
            .map_err(|e| ChunkingError::ExportError(format!("UTF-8 encoding failure: {e}")))
    }

    /// Internal batch envelope writer.
    fn export_batch_to_writer<W: Write>(
        &self,
        chunks: &[Chunk],
        writer: &mut W,
    ) -> Result<ChunkExportMetrics, ChunkingError> {
        let mut metrics = ChunkExportMetrics::default();
        let mut eligible_chunks = Vec::new();
        let mut cumulative_tokens = 0usize;

        for chunk in chunks {
            if let Some(budget) = self.options.max_token_budget {
                if cumulative_tokens.saturating_add(chunk.token_count) > budget {
                    metrics.truncated = true;
                    break;
                }
            }

            if self.options.validate_schema {
                self.validator.validate_chunk(
                    chunk,
                    self.options.target,
                    self.options.uuid_ids,
                    &self.options.weaviate_class,
                    &self.options.embedding_model,
                )?;
            }

            metrics.chunks_exported += 1;
            metrics.tokens_exported += chunk.token_count;
            cumulative_tokens += chunk.token_count;
            eligible_chunks.push(chunk);
        }

        let json_value = match self.options.target {
            VectorTarget::Qdrant => {
                let points: Vec<QdrantPoint> = eligible_chunks
                    .iter()
                    .map(|c| QdrantPoint::from_chunk(c, self.options.uuid_ids))
                    .collect();
                serde_json::to_value(QdrantBatch { points })?
            }
            VectorTarget::Pinecone => {
                let vectors: Vec<PineconeRecord> = eligible_chunks
                    .iter()
                    .map(|c| PineconeRecord::from_chunk(c))
                    .collect();
                serde_json::to_value(PineconeBatch { vectors })?
            }
            VectorTarget::Chroma => {
                let ids: Vec<String> = eligible_chunks
                    .iter()
                    .map(|c| c.chunk_id.as_str().to_string())
                    .collect();
                let documents: Vec<String> =
                    eligible_chunks.iter().map(|c| c.content.clone()).collect();
                let metadatas = eligible_chunks
                    .iter()
                    .map(|c| ChromaRecord::from_chunk(c).metadata)
                    .collect();
                serde_json::to_value(ChromaBatchCollection {
                    ids,
                    documents,
                    metadatas,
                })?
            }
            VectorTarget::Weaviate => {
                let objects: Vec<WeaviateObject> = eligible_chunks
                    .iter()
                    .map(|c| WeaviateObject::from_chunk(c, &self.options.weaviate_class))
                    .collect();
                serde_json::to_value(WeaviateBatch { objects })?
            }
            VectorTarget::GenericJsonl
            | VectorTarget::Milvus
            | VectorTarget::OpenAiBatch
            | VectorTarget::CohereBatch => {
                let records: Vec<serde_json::Value> = eligible_chunks
                    .iter()
                    .map(|c| self.serialize_chunk_to_value(c))
                    .collect::<Result<_, _>>()?;
                serde_json::Value::Array(records)
            }
        };

        let json_bytes = serde_json::to_vec_pretty(&json_value)?;
        writer.write_all(&json_bytes)?;
        writer.flush()?;

        metrics.bytes_written = json_bytes.len();
        metrics.lines_written = json_bytes.iter().filter(|&&b| b == b'\n').count() + 1;

        Ok(metrics)
    }

    /// Serializes a single chunk into its target-specific JSON representation.
    pub fn serialize_chunk_record(&self, chunk: &Chunk) -> Result<String, ChunkingError> {
        let val = self.serialize_chunk_to_value(chunk)?;
        serde_json::to_string(&val).map_err(Into::into)
    }

    fn serialize_chunk_to_value(&self, chunk: &Chunk) -> Result<serde_json::Value, ChunkingError> {
        match self.options.target {
            VectorTarget::GenericJsonl => {
                let record = GenericRagRecord::from(chunk);
                serde_json::to_value(record).map_err(Into::into)
            }
            VectorTarget::Qdrant => {
                let point = QdrantPoint::from_chunk(chunk, self.options.uuid_ids);
                serde_json::to_value(point).map_err(Into::into)
            }
            VectorTarget::Pinecone => {
                let rec = PineconeRecord::from_chunk(chunk);
                serde_json::to_value(rec).map_err(Into::into)
            }
            VectorTarget::Chroma => {
                let rec = ChromaRecord::from_chunk(chunk);
                serde_json::to_value(rec).map_err(Into::into)
            }
            VectorTarget::Milvus => {
                let entity = MilvusEntity::from(chunk);
                serde_json::to_value(entity).map_err(Into::into)
            }
            VectorTarget::Weaviate => {
                let obj = WeaviateObject::from_chunk(chunk, &self.options.weaviate_class);
                serde_json::to_value(obj).map_err(Into::into)
            }
            VectorTarget::OpenAiBatch => {
                let req = OpenAiBatchRequest::from_chunk(chunk, &self.options.embedding_model);
                serde_json::to_value(req).map_err(Into::into)
            }
            VectorTarget::CohereBatch => {
                let rec = CohereRecord::from_chunk(chunk);
                serde_json::to_value(rec).map_err(Into::into)
            }
        }
    }
}

/// Convenience function exporting chunks directly to newline-delimited Generic RAG JSONL.
///
/// # Errors
/// Returns [`ChunkingError`] on serialization failure.
pub fn export_chunks_to_jsonl(chunks: &[Chunk]) -> Result<String, ChunkingError> {
    let exporter = ChunkExporter::new();
    let (output, _) = exporter.export_to_string(chunks)?;
    Ok(output)
}

/// Convenience function exporting chunks directly to the specified vector target format.
///
/// # Errors
/// Returns [`ChunkingError`] on validation or serialization failure.
pub fn export_chunks_to_vector_target(
    chunks: &[Chunk],
    target: VectorTarget,
) -> Result<String, ChunkingError> {
    let options = ChunkExportOptions::default().with_target(target);
    let exporter = ChunkExporter::with_options(options);
    let (output, _) = exporter.export_to_string(chunks)?;
    Ok(output)
}
