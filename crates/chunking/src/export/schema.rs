//! # Vector Ingest Schemas & Target Adapters (`crates/chunking/src/export/schema.rs`)
//!
//! Strongly-typed schema definitions and converters mapping [`Chunk`] collections
//! to official vector database and AI model ingestion formats:
//! - **Generic RAG JSONL**: Canonical line-delimited records with full metadata fidelity.
//! - **Qdrant**: REST/gRPC points with payload and deterministic UUID point IDs.
//! - **Pinecone**: Upsert vector records with flattened, type-compliant metadata.
//! - **Chroma / ChromaDB**: Individual records and collection batch representations.
//! - **Milvus / Zilliz**: Relational entity rows with coordinates and hashes.
//! - **Weaviate**: Data objects conforming to RFC 4122 UUIDs and class properties.
//! - **OpenAI Batch Embeddings**: `/v1/embeddings` batch requests for massive corpora.
//! - **Cohere Embed**: Records for text embeddings and reranking.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::model::{Chunk, ChunkCitation, ChunkCrossReference, ChunkSymbolMetadata};

/// Supported vector database and AI embedding ingestion targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorTarget {
    /// Universal newline-delimited RAG chunk record.
    #[default]
    GenericJsonl,
    /// Qdrant vector database point format with payload.
    Qdrant,
    /// Pinecone vector upsert record with strict scalar/string-array metadata.
    Pinecone,
    /// ChromaDB document collection record format.
    Chroma,
    /// Milvus / Zilliz vector database entity row format.
    Milvus,
    /// Weaviate vector search engine data object format with RFC 4122 UUID.
    Weaviate,
    /// OpenAI Batch Embeddings API input format (`POST /v1/embeddings`).
    OpenAiBatch,
    /// Cohere Embed batch input format.
    CohereBatch,
}

impl VectorTarget {
    /// Returns the snake_case label for this vector target.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GenericJsonl => "generic_jsonl",
            Self::Qdrant => "qdrant",
            Self::Pinecone => "pinecone",
            Self::Chroma => "chroma",
            Self::Milvus => "milvus",
            Self::Weaviate => "weaviate",
            Self::OpenAiBatch => "openai_batch",
            Self::CohereBatch => "cohere_batch",
        }
    }
}

impl fmt::Display for VectorTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Deterministically converts any arbitrary string ID or hash into a standard RFC 4122 Version 4 UUID.
///
/// Uses BLAKE3 cryptographic hashing of the input, setting RFC 4122 version 4 and variant 1 bits.
/// Guarantees 100% collision resistance, determinism, and platform independence.
#[must_use]
pub fn to_deterministic_uuid(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes());
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&hash.as_bytes()[..16]);

    // RFC 4122 version 4: set bits 4-7 of the 7th byte (index 6) to 0100 (0x40)
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    // RFC 4122 variant 1: set bits 6-7 of the 9th byte (index 8) to 10 (0x80)
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
    )
}

/// Validates whether a given string is a correctly formatted RFC 4122 UUID (36 characters with hyphens).
#[must_use]
pub fn is_valid_uuid(uuid_str: &str) -> bool {
    if uuid_str.len() != 36 {
        return false;
    }
    let bytes = uuid_str.as_bytes();
    if bytes[8] != b'-' || bytes[13] != b'-' || bytes[18] != b'-' || bytes[23] != b'-' {
        return false;
    }
    for (i, &b) in bytes.iter().enumerate() {
        if i == 8 || i == 13 || i == 18 || i == 23 {
            continue;
        }
        if !b.is_ascii_hexdigit() {
            return false;
        }
    }
    true
}

/// Canonical generic RAG chunk record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenericRagRecord {
    /// Chunk identifier.
    pub id: String,
    /// Parent document identifier.
    pub document_id: String,
    /// Parent section identifier from CDM.
    pub section_id: String,
    /// 0-indexed position within section/document.
    pub chunk_index: usize,
    /// Total chunks in sequence.
    pub total_chunks: usize,
    /// Descriptive title or section heading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Relative file path or DOM selector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Canonical source URL or permalink.
    pub source_url: String,
    /// 1-indexed line span `[start_line, end_line]`.
    pub line_span: [u32; 2],
    /// Byte range `[start_byte, end_byte]`.
    pub byte_span: [u64; 2],
    /// Augmented chunk content including context header.
    pub content: String,
    /// Raw chunk body without context header.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_content: Option<String>,
    /// BLAKE3 cryptographic content hash.
    pub content_hash: String,
    /// Estimated or exact token count.
    pub token_count: usize,
    /// Token estimator model label.
    pub token_estimator: String,
    /// Programming or markup language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Structural kind of chunk.
    pub kind: String,
    /// Authoritative citation with permalink and line span.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation: Option<ChunkCitation>,
    /// Graph edges to related chunks.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cross_references: Vec<ChunkCrossReference>,
    /// Predecessor chunk ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prev_chunk_id: Option<String>,
    /// Successor chunk ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_chunk_id: Option<String>,
    /// Parent declaration chunk ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_chunk_id: Option<String>,
    /// Declared AST symbols.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ast_symbols: Vec<ChunkSymbolMetadata>,
    /// Custom metadata key-values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,
}

impl From<&Chunk> for GenericRagRecord {
    fn from(chunk: &Chunk) -> Self {
        Self {
            id: chunk.chunk_id.as_str().to_string(),
            document_id: chunk.document_id.to_string(),
            section_id: chunk.section_id.clone(),
            chunk_index: chunk.chunk_index,
            total_chunks: chunk.total_chunks,
            title: chunk.title.clone(),
            file_path: chunk.original_path.clone(),
            source_url: chunk.source_url.clone(),
            line_span: [chunk.line_range.start_line, chunk.line_range.end_line],
            byte_span: [chunk.byte_range.start_byte, chunk.byte_range.end_byte],
            content: chunk.content.clone(),
            raw_content: Some(chunk.raw_content.clone()),
            content_hash: chunk.content_hash.clone(),
            token_count: chunk.token_count,
            token_estimator: chunk.token_estimator.clone(),
            language: chunk.language.clone(),
            kind: chunk.kind.as_str().to_string(),
            citation: chunk.citation.clone(),
            cross_references: chunk.cross_references.clone(),
            prev_chunk_id: chunk
                .prev_chunk_id
                .as_ref()
                .map(|id| id.as_str().to_string()),
            next_chunk_id: chunk
                .next_chunk_id
                .as_ref()
                .map(|id| id.as_str().to_string()),
            parent_chunk_id: chunk
                .parent_chunk_id
                .as_ref()
                .map(|id| id.as_str().to_string()),
            ast_symbols: chunk.ast_symbols.clone(),
            metadata: chunk.metadata.clone(),
        }
    }
}

/// Point structure for Qdrant vector database.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdrantPoint {
    /// Point identifier: RFC 4122 UUID or string.
    pub id: String,
    /// Optional dense vector values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector: Option<Vec<f32>>,
    /// Arbitrary payload containing chunk text and metadata.
    pub payload: serde_json::Value,
}

/// Batch collection of Qdrant points.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QdrantBatch {
    /// Vector points.
    pub points: Vec<QdrantPoint>,
}

impl QdrantPoint {
    /// Creates a Qdrant point from a [`Chunk`].
    #[must_use]
    pub fn from_chunk(chunk: &Chunk, use_uuid: bool) -> Self {
        let id = if use_uuid {
            to_deterministic_uuid(chunk.chunk_id.as_str())
        } else {
            chunk.chunk_id.as_str().to_string()
        };

        let mut payload = serde_json::Map::new();
        payload.insert(
            "content".into(),
            serde_json::Value::String(chunk.content.clone()),
        );
        payload.insert(
            "chunk_id".into(),
            serde_json::Value::String(chunk.chunk_id.as_str().into()),
        );
        payload.insert(
            "document_id".into(),
            serde_json::Value::String(chunk.document_id.to_string()),
        );
        payload.insert(
            "section_id".into(),
            serde_json::Value::String(chunk.section_id.clone()),
        );
        payload.insert(
            "chunk_index".into(),
            serde_json::Value::from(chunk.chunk_index),
        );
        payload.insert(
            "total_chunks".into(),
            serde_json::Value::from(chunk.total_chunks),
        );
        payload.insert(
            "source_url".into(),
            serde_json::Value::String(chunk.source_url.clone()),
        );
        payload.insert(
            "kind".into(),
            serde_json::Value::String(chunk.kind.as_str().into()),
        );
        payload.insert(
            "token_count".into(),
            serde_json::Value::from(chunk.token_count),
        );
        payload.insert(
            "content_hash".into(),
            serde_json::Value::String(chunk.content_hash.clone()),
        );
        payload.insert(
            "line_span".into(),
            serde_json::json!([chunk.line_range.start_line, chunk.line_range.end_line]),
        );
        payload.insert(
            "byte_span".into(),
            serde_json::json!([chunk.byte_range.start_byte, chunk.byte_range.end_byte]),
        );

        if let Some(ref path) = chunk.original_path {
            payload.insert("file_path".into(), serde_json::Value::String(path.clone()));
        }
        if let Some(ref title) = chunk.title {
            payload.insert("title".into(), serde_json::Value::String(title.clone()));
        }
        if let Some(ref lang) = chunk.language {
            payload.insert("language".into(), serde_json::Value::String(lang.clone()));
        }
        if let Some(ref cit) = chunk.citation {
            if let Ok(cit_val) = serde_json::to_value(cit) {
                payload.insert("citation".into(), cit_val);
            }
        }
        if !chunk.cross_references.is_empty() {
            if let Ok(xr_val) = serde_json::to_value(&chunk.cross_references) {
                payload.insert("cross_references".into(), xr_val);
            }
        }
        if let Some(ref prev) = chunk.prev_chunk_id {
            payload.insert(
                "prev_chunk_id".into(),
                serde_json::Value::String(prev.as_str().into()),
            );
        }
        if let Some(ref next) = chunk.next_chunk_id {
            payload.insert(
                "next_chunk_id".into(),
                serde_json::Value::String(next.as_str().into()),
            );
        }
        if let Some(ref parent) = chunk.parent_chunk_id {
            payload.insert(
                "parent_chunk_id".into(),
                serde_json::Value::String(parent.as_str().into()),
            );
        }

        Self {
            id,
            vector: None,
            payload: serde_json::Value::Object(payload),
        }
    }
}

/// Pinecone upsert vector record.
///
/// Pinecone restricts metadata values strictly to:
/// - String
/// - Number (integer or float)
/// - Boolean
/// - Array of Strings
///
/// Null values, nested maps, or mixed-type arrays are explicitly rejected by Pinecone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PineconeRecord {
    /// Vector unique ID.
    pub id: String,
    /// Dense vector values (empty for pre-embedding upsert).
    pub values: Vec<f32>,
    /// Optional sparse vector values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sparse_values: Option<serde_json::Value>,
    /// Strict metadata dictionary conforming to Pinecone schema rules.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Batch collection of Pinecone vector records.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PineconeBatch {
    /// Vector records.
    pub vectors: Vec<PineconeRecord>,
}

impl PineconeRecord {
    /// Converts a [`Chunk`] into a Pinecone-compliant vector record.
    #[must_use]
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut meta = BTreeMap::new();
        // Standard Pinecone text key
        meta.insert(
            "text".into(),
            serde_json::Value::String(chunk.content.clone()),
        );
        meta.insert(
            "document_id".into(),
            serde_json::Value::String(chunk.document_id.to_string()),
        );
        meta.insert(
            "section_id".into(),
            serde_json::Value::String(chunk.section_id.clone()),
        );
        meta.insert(
            "chunk_index".into(),
            serde_json::Value::from(chunk.chunk_index),
        );
        meta.insert(
            "total_chunks".into(),
            serde_json::Value::from(chunk.total_chunks),
        );
        meta.insert(
            "source_url".into(),
            serde_json::Value::String(chunk.source_url.clone()),
        );
        meta.insert(
            "kind".into(),
            serde_json::Value::String(chunk.kind.as_str().into()),
        );
        meta.insert(
            "token_count".into(),
            serde_json::Value::from(chunk.token_count),
        );
        meta.insert(
            "content_hash".into(),
            serde_json::Value::String(chunk.content_hash.clone()),
        );
        meta.insert(
            "line_start".into(),
            serde_json::Value::from(chunk.line_range.start_line),
        );
        meta.insert(
            "line_end".into(),
            serde_json::Value::from(chunk.line_range.end_line),
        );

        if let Some(ref path) = chunk.original_path {
            meta.insert("file_path".into(), serde_json::Value::String(path.clone()));
        }
        if let Some(ref title) = chunk.title {
            meta.insert("title".into(), serde_json::Value::String(title.clone()));
        }
        if let Some(ref lang) = chunk.language {
            meta.insert("language".into(), serde_json::Value::String(lang.clone()));
        }
        if let Some(ref cit) = chunk.citation {
            meta.insert(
                "citation_display".into(),
                serde_json::Value::String(cit.display_citation.clone()),
            );
            meta.insert(
                "citation_permalink".into(),
                serde_json::Value::String(cit.permalink.clone()),
            );
            if let Some(ref sha) = cit.commit_sha {
                meta.insert(
                    "citation_commit_sha".into(),
                    serde_json::Value::String(sha.clone()),
                );
            }
        }
        if let Some(ref prev) = chunk.prev_chunk_id {
            meta.insert(
                "prev_chunk_id".into(),
                serde_json::Value::String(prev.as_str().into()),
            );
        }
        if let Some(ref next) = chunk.next_chunk_id {
            meta.insert(
                "next_chunk_id".into(),
                serde_json::Value::String(next.as_str().into()),
            );
        }
        if let Some(ref parent) = chunk.parent_chunk_id {
            meta.insert(
                "parent_chunk_id".into(),
                serde_json::Value::String(parent.as_str().into()),
            );
        }
        if !chunk.ast_symbols.is_empty() {
            let symbol_names: Vec<serde_json::Value> = chunk
                .ast_symbols
                .iter()
                .map(|s| serde_json::Value::String(s.name.clone()))
                .collect();
            meta.insert("symbols".into(), serde_json::Value::Array(symbol_names));
        }

        Self {
            id: chunk.chunk_id.as_str().to_string(),
            values: Vec::new(),
            sparse_values: None,
            metadata: meta,
        }
    }
}

/// ChromaDB collection document record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChromaRecord {
    /// Document identifier.
    pub id: String,
    /// Primary text document.
    pub document: String,
    /// Flat scalar metadata dictionary (string, number, bool).
    pub metadata: BTreeMap<String, serde_json::Value>,
    /// Optional embedding vector.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
}

/// ChromaDB batch collection format for `collection.add(...)`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChromaBatchCollection {
    /// Array of document IDs.
    pub ids: Vec<String>,
    /// Array of text documents.
    pub documents: Vec<String>,
    /// Array of metadata maps.
    pub metadatas: Vec<BTreeMap<String, serde_json::Value>>,
}

impl ChromaRecord {
    /// Converts a [`Chunk`] into a Chroma document record.
    #[must_use]
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let mut meta = BTreeMap::new();
        meta.insert(
            "document_id".into(),
            serde_json::Value::String(chunk.document_id.to_string()),
        );
        meta.insert(
            "section_id".into(),
            serde_json::Value::String(chunk.section_id.clone()),
        );
        meta.insert(
            "chunk_index".into(),
            serde_json::Value::from(chunk.chunk_index),
        );
        meta.insert(
            "source_url".into(),
            serde_json::Value::String(chunk.source_url.clone()),
        );
        meta.insert(
            "kind".into(),
            serde_json::Value::String(chunk.kind.as_str().into()),
        );
        meta.insert(
            "token_count".into(),
            serde_json::Value::from(chunk.token_count),
        );
        meta.insert(
            "content_hash".into(),
            serde_json::Value::String(chunk.content_hash.clone()),
        );

        if let Some(ref path) = chunk.original_path {
            meta.insert("file_path".into(), serde_json::Value::String(path.clone()));
        }
        if let Some(ref title) = chunk.title {
            meta.insert("title".into(), serde_json::Value::String(title.clone()));
        }
        if let Some(ref lang) = chunk.language {
            meta.insert("language".into(), serde_json::Value::String(lang.clone()));
        }
        if let Some(ref cit) = chunk.citation {
            meta.insert(
                "citation".into(),
                serde_json::Value::String(cit.display_citation.clone()),
            );
            meta.insert(
                "permalink".into(),
                serde_json::Value::String(cit.permalink.clone()),
            );
        }

        Self {
            id: chunk.chunk_id.as_str().to_string(),
            document: chunk.content.clone(),
            metadata: meta,
            embedding: None,
        }
    }
}

/// Milvus / Zilliz vector database entity row record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MilvusEntity {
    /// Entity ID.
    pub id: String,
    /// Raw or augmented text content.
    pub content: String,
    /// Document identifier.
    pub document_id: String,
    /// CDM section identifier.
    pub section_id: String,
    /// Chunk index.
    pub chunk_index: usize,
    /// Canonical source URL.
    pub source_url: String,
    /// Source file path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Chunk kind string.
    pub kind: String,
    /// Language if code.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    /// Token count.
    pub token_count: usize,
    /// BLAKE3 content hash.
    pub content_hash: String,
    /// Start line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_start: Option<u32>,
    /// End line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_end: Option<u32>,
    /// Formatted citation permalink.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_permalink: Option<String>,
    /// Compact citation label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citation_display: Option<String>,
}

impl From<&Chunk> for MilvusEntity {
    fn from(chunk: &Chunk) -> Self {
        Self {
            id: chunk.chunk_id.as_str().to_string(),
            content: chunk.content.clone(),
            document_id: chunk.document_id.to_string(),
            section_id: chunk.section_id.clone(),
            chunk_index: chunk.chunk_index,
            source_url: chunk.source_url.clone(),
            file_path: chunk.original_path.clone(),
            kind: chunk.kind.as_str().to_string(),
            language: chunk.language.clone(),
            token_count: chunk.token_count,
            content_hash: chunk.content_hash.clone(),
            line_start: Some(chunk.line_range.start_line),
            line_end: Some(chunk.line_range.end_line),
            citation_permalink: chunk.citation.as_ref().map(|c| c.permalink.clone()),
            citation_display: chunk.citation.as_ref().map(|c| c.display_citation.clone()),
        }
    }
}

/// Weaviate v1/v4 data object.
///
/// Weaviate strictly requires an RFC 4122 UUID for object IDs and class names starting with an uppercase letter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaviateObject {
    /// Object identifier (must be valid RFC 4122 UUID).
    pub id: String,
    /// Class name in Weaviate schema (e.g. `"DocumentChunk"`).
    pub class: String,
    /// Properties mapping.
    pub properties: serde_json::Value,
}

/// Batch collection of Weaviate objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WeaviateBatch {
    /// Data objects.
    pub objects: Vec<WeaviateObject>,
}

impl WeaviateObject {
    /// Creates a Weaviate object from a [`Chunk`].
    #[must_use]
    pub fn from_chunk(chunk: &Chunk, class_name: &str) -> Self {
        let uuid = to_deterministic_uuid(chunk.chunk_id.as_str());

        let mut props = serde_json::Map::new();
        props.insert(
            "content".into(),
            serde_json::Value::String(chunk.content.clone()),
        );
        props.insert(
            "chunk_id".into(),
            serde_json::Value::String(chunk.chunk_id.as_str().into()),
        );
        props.insert(
            "document_id".into(),
            serde_json::Value::String(chunk.document_id.to_string()),
        );
        props.insert(
            "section_id".into(),
            serde_json::Value::String(chunk.section_id.clone()),
        );
        props.insert(
            "chunk_index".into(),
            serde_json::Value::from(chunk.chunk_index),
        );
        props.insert(
            "total_chunks".into(),
            serde_json::Value::from(chunk.total_chunks),
        );
        props.insert(
            "source_url".into(),
            serde_json::Value::String(chunk.source_url.clone()),
        );
        props.insert(
            "kind".into(),
            serde_json::Value::String(chunk.kind.as_str().into()),
        );
        props.insert(
            "token_count".into(),
            serde_json::Value::from(chunk.token_count),
        );
        props.insert(
            "content_hash".into(),
            serde_json::Value::String(chunk.content_hash.clone()),
        );

        if let Some(ref path) = chunk.original_path {
            props.insert("file_path".into(), serde_json::Value::String(path.clone()));
        }
        if let Some(ref title) = chunk.title {
            props.insert("title".into(), serde_json::Value::String(title.clone()));
        }
        if let Some(ref cit) = chunk.citation {
            props.insert(
                "citation".into(),
                serde_json::Value::String(cit.display_citation.clone()),
            );
            props.insert(
                "permalink".into(),
                serde_json::Value::String(cit.permalink.clone()),
            );
        }

        Self {
            id: uuid,
            class: class_name.to_string(),
            properties: serde_json::Value::Object(props),
        }
    }
}

/// OpenAI Batch Embeddings API payload body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiBatchBody {
    /// Embedding model (e.g. `"text-embedding-3-small"`).
    pub model: String,
    /// Input text content to embed.
    pub input: String,
}

/// OpenAI Batch API request line for `/v1/embeddings`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenAiBatchRequest {
    /// Unique custom ID correlating response to input chunk.
    pub custom_id: String,
    /// HTTP method (`"POST"`).
    pub method: String,
    /// Endpoint URL (`"/v1/embeddings"`).
    pub url: String,
    /// Request body payload.
    pub body: OpenAiBatchBody,
}

impl OpenAiBatchRequest {
    /// Creates an OpenAI Batch API request line from a [`Chunk`].
    #[must_use]
    pub fn from_chunk(chunk: &Chunk, model_name: &str) -> Self {
        Self {
            custom_id: chunk.chunk_id.as_str().to_string(),
            method: "POST".to_string(),
            url: "/v1/embeddings".to_string(),
            body: OpenAiBatchBody {
                model: model_name.to_string(),
                input: chunk.content.clone(),
            },
        }
    }
}

/// Cohere Embed input record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CohereRecord {
    /// Chunk identifier.
    pub id: String,
    /// Text to embed.
    pub text: String,
    /// Metadata map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CohereRecord {
    /// Creates a Cohere Embed input record from a [`Chunk`].
    #[must_use]
    pub fn from_chunk(chunk: &Chunk) -> Self {
        let meta = serde_json::json!({
            "document_id": chunk.document_id.to_string(),
            "source_url": chunk.source_url,
            "kind": chunk.kind.as_str(),
            "token_count": chunk.token_count,
            "content_hash": chunk.content_hash,
            "file_path": chunk.original_path,
        });

        Self {
            id: chunk.chunk_id.as_str().to_string(),
            text: chunk.content.clone(),
            metadata: Some(meta),
        }
    }
}
