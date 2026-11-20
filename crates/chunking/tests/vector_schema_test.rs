//! # Integration Tests: Vector Database Ingest Schemas (`tests/vector_schema_test.rs`)

use std::collections::BTreeMap;
use web_repo_to_ai_cdm::{ByteRange, LineRange};
use web_repo_to_ai_chunking::{
    export_chunks_to_vector_target, is_valid_uuid, to_deterministic_uuid, ChromaBatchCollection,
    ChromaRecord, Chunk, ChunkCitation, ChunkExportOptions, ChunkExporter, ChunkId, ChunkKind,
    ChunkSymbolMetadata, CohereRecord, MilvusEntity, OpenAiBatchRequest, PineconeBatch,
    PineconeRecord, QdrantBatch, QdrantPoint, VectorSchemaValidator, VectorTarget, WeaviateBatch,
    WeaviateObject,
};
use web_repo_to_ai_core::DocumentId;

fn create_test_code_chunk() -> Chunk {
    let doc_id = DocumentId::new();

    Chunk {
        chunk_id: ChunkId::new("chk_vec_s1_c0_f9e8d7c6"),
        document_id: doc_id,
        section_id: "sec_code_1".to_string(),
        chunk_index: 0,
        total_chunks: 1,
        title: Some("calculate_hash".to_string()),
        original_path: Some("src/hash.rs".to_string()),
        source_url: "https://github.com/example/repo/blob/abcdef1234/src/hash.rs#L10-L30".to_string(),
        line_range: LineRange::new(10, 30),
        byte_range: ByteRange::new(150, 750),
        content: "pub fn calculate_hash(data: &[u8]) -> [u8; 32] {\n    blake3::hash(data).into()\n}".to_string(),
        raw_content: "pub fn calculate_hash(data: &[u8]) -> [u8; 32] {\n    blake3::hash(data).into()\n}".to_string(),
        context_header: None,
        content_hash: "f9e8d7c6b5a4123456789abcdef0123456789abcdef0123456789abcdef01234".to_string(),
        token_count: 24,
        token_estimator: "exact:o200k_base".to_string(),
        language: Some("rust".to_string()),
        kind: ChunkKind::CodeAstDeclaration,
        ast_symbols: vec![ChunkSymbolMetadata {
            name: "calculate_hash".to_string(),
            kind: web_repo_to_ai_code_intelligence::SymbolKind::Function,
            visibility: web_repo_to_ai_code_intelligence::Visibility::Public,
            signature: "pub fn calculate_hash(data: &[u8]) -> [u8; 32]".to_string(),
            doc_comment: Some("Calculates BLAKE3 hash.".to_string()),
            parent_scope: None,
            line_number: 10,
        }],
        metadata: BTreeMap::new(),
        citation: Some(ChunkCitation {
            citation_id: "cit_code_1".to_string(),
            source_url: "https://github.com/example/repo".to_string(),
            permalink: "https://github.com/example/repo/blob/abcdef1234/src/hash.rs#L10-L30".to_string(),
            display_citation: "src/hash.rs:10-30 (Commit abcdef1)".to_string(),
            markdown_citation: "[src/hash.rs#L10-L30](https://github.com/example/repo/blob/abcdef1234/src/hash.rs#L10-L30)".to_string(),
            file_path: Some("src/hash.rs".to_string()),
            line_span: Some((10, 30)),
            commit_sha: Some("abcdef1234567890".to_string()),
            source_title: Some("example/repo".to_string()),
            anchor: Some("L10-L30".to_string()),
        }),
        cross_references: Vec::new(),
        prev_chunk_id: None,
        next_chunk_id: None,
        parent_chunk_id: None,
    }
}

#[test]
fn test_deterministic_uuid_rfc4122_compliance() {
    let input = "chk_vec_s1_c0_f9e8d7c6";
    let uuid1 = to_deterministic_uuid(input);
    let uuid2 = to_deterministic_uuid(input);

    assert_eq!(uuid1, uuid2, "UUID must be strictly deterministic");
    assert!(
        is_valid_uuid(&uuid1),
        "UUID '{uuid1}' must be valid RFC 4122"
    );

    // Check version 4 (character at index 14 is '4')
    assert_eq!(&uuid1[14..15], "4", "UUID version must be 4");

    // Check variant 1 (character at index 19 is '8', '9', 'a', or 'b')
    let variant_char = uuid1.chars().nth(19).unwrap().to_ascii_lowercase();
    assert!(
        ['8', '9', 'a', 'b'].contains(&variant_char),
        "UUID variant must be RFC 4122 variant 1, got '{variant_char}'"
    );

    // Different input must yield different UUID
    let uuid_different = to_deterministic_uuid("chk_other_123");
    assert_ne!(uuid1, uuid_different);
    assert!(is_valid_uuid(&uuid_different));
}

#[test]
fn test_qdrant_point_export_and_batch() {
    let chunk = create_test_code_chunk();

    // 1. JSONL point export
    let options = ChunkExportOptions::default()
        .with_target(VectorTarget::Qdrant)
        .with_uuid_ids(true);
    let exporter = ChunkExporter::with_options(options);
    let (output, metrics) = exporter
        .export_to_string(std::slice::from_ref(&chunk))
        .expect("Qdrant export");

    assert_eq!(metrics.chunks_exported, 1);
    let point: QdrantPoint = serde_json::from_str(output.trim()).expect("Parse Qdrant point");
    assert!(is_valid_uuid(&point.id), "Qdrant ID must be valid UUID");
    assert!(point.payload.get("content").is_some());
    assert_eq!(point.payload["token_count"], 24);
    assert_eq!(point.payload["file_path"], "src/hash.rs");

    // 2. Batch envelope export
    let batch_options = ChunkExportOptions::default()
        .with_target(VectorTarget::Qdrant)
        .with_batch_envelope(true)
        .with_uuid_ids(true);
    let batch_exporter = ChunkExporter::with_options(batch_options);
    let batch_json = batch_exporter
        .export_batch_json(&[chunk])
        .expect("Qdrant batch");
    let batch: QdrantBatch = serde_json::from_str(&batch_json).expect("Parse Qdrant batch");
    assert_eq!(batch.points.len(), 1);
    assert!(is_valid_uuid(&batch.points[0].id));
}

#[test]
fn test_pinecone_record_export_and_strict_metadata() {
    let chunk = create_test_code_chunk();

    let output =
        export_chunks_to_vector_target(std::slice::from_ref(&chunk), VectorTarget::Pinecone)
            .expect("Pinecone export");
    let record: PineconeRecord =
        serde_json::from_str(output.trim()).expect("Parse Pinecone record");

    assert_eq!(record.id, "chk_vec_s1_c0_f9e8d7c6");
    assert!(
        record.values.is_empty(),
        "Values should be empty for pre-embedding upsert"
    );

    // Pinecone Metadata assertions
    let meta = &record.metadata;
    assert!(
        meta.contains_key("text"),
        "Must contain standard 'text' key"
    );
    assert_eq!(meta["text"].as_str().unwrap(), chunk.content);
    assert_eq!(meta["token_count"].as_i64().unwrap(), 24);
    assert_eq!(meta["line_start"].as_i64().unwrap(), 10);
    assert_eq!(meta["line_end"].as_i64().unwrap(), 30);
    assert_eq!(meta["file_path"].as_str().unwrap(), "src/hash.rs");

    // Symbols array
    let symbols = meta["symbols"].as_array().expect("Symbols must be array");
    assert_eq!(symbols.len(), 1);
    assert_eq!(symbols[0].as_str().unwrap(), "calculate_hash");

    // Verify Pinecone strict type rules: only String, Number, Bool, or Array of Strings. No nulls or objects!
    for (k, v) in meta {
        match v {
            serde_json::Value::String(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::Bool(_) => {}
            serde_json::Value::Array(arr) => {
                for item in arr {
                    assert!(
                        item.is_string(),
                        "Pinecone arrays must only contain strings, found {item:?} for key '{k}'"
                    );
                }
            }
            serde_json::Value::Null => {
                panic!("Pinecone metadata cannot contain null values: key '{k}'");
            }
            serde_json::Value::Object(_) => {
                panic!("Pinecone metadata cannot contain nested objects: key '{k}'");
            }
        }
    }

    // Batch envelope test
    let batch_options = ChunkExportOptions::default()
        .with_target(VectorTarget::Pinecone)
        .with_batch_envelope(true);
    let batch_exporter = ChunkExporter::with_options(batch_options);
    let batch_json = batch_exporter
        .export_batch_json(&[chunk])
        .expect("Pinecone batch");
    let batch: PineconeBatch = serde_json::from_str(&batch_json).expect("Parse Pinecone batch");
    assert_eq!(batch.vectors.len(), 1);
}

#[test]
fn test_chroma_record_and_batch_collection() {
    let chunk = create_test_code_chunk();

    // 1. Line-delimited record
    let output = export_chunks_to_vector_target(std::slice::from_ref(&chunk), VectorTarget::Chroma)
        .expect("Chroma export");
    let record: ChromaRecord = serde_json::from_str(output.trim()).expect("Parse Chroma record");
    assert_eq!(record.id, "chk_vec_s1_c0_f9e8d7c6");
    assert_eq!(record.document, chunk.content);
    assert_eq!(
        record.metadata["file_path"].as_str().unwrap(),
        "src/hash.rs"
    );

    // 2. Batch collection format
    let batch_options = ChunkExportOptions::default()
        .with_target(VectorTarget::Chroma)
        .with_batch_envelope(true);
    let batch_exporter = ChunkExporter::with_options(batch_options);
    let batch_json = batch_exporter
        .export_batch_json(&[chunk])
        .expect("Chroma batch");
    let batch: ChromaBatchCollection =
        serde_json::from_str(&batch_json).expect("Parse Chroma batch collection");

    assert_eq!(batch.ids, vec!["chk_vec_s1_c0_f9e8d7c6"]);
    assert_eq!(batch.documents.len(), 1);
    assert_eq!(batch.metadatas.len(), 1);
    assert_eq!(batch.metadatas[0]["token_count"], 24);
}

#[test]
fn test_milvus_entity_export() {
    let chunk = create_test_code_chunk();

    let output =
        export_chunks_to_vector_target(&[chunk], VectorTarget::Milvus).expect("Milvus export");
    let entity: MilvusEntity = serde_json::from_str(output.trim()).expect("Parse Milvus entity");

    assert_eq!(entity.id, "chk_vec_s1_c0_f9e8d7c6");
    assert_eq!(entity.token_count, 24);
    assert_eq!(entity.line_start, Some(10));
    assert_eq!(entity.line_end, Some(30));
    assert_eq!(entity.language.as_deref(), Some("rust"));
    assert!(entity.citation_permalink.is_some());
}

#[test]
fn test_weaviate_object_and_batch() {
    let chunk = create_test_code_chunk();

    // 1. JSONL Weaviate object
    let options = ChunkExportOptions::default()
        .with_target(VectorTarget::Weaviate)
        .with_weaviate_class("CodeChunk");
    let exporter = ChunkExporter::with_options(options);
    let (output, _) = exporter
        .export_to_string(std::slice::from_ref(&chunk))
        .expect("Weaviate export");

    let obj: WeaviateObject = serde_json::from_str(output.trim()).expect("Parse Weaviate object");
    assert_eq!(obj.class, "CodeChunk");
    assert!(is_valid_uuid(&obj.id), "Weaviate ID must be RFC 4122 UUID");
    assert_eq!(obj.properties["chunk_id"], "chk_vec_s1_c0_f9e8d7c6");

    // 2. Batch Weaviate envelope
    let batch_options = ChunkExportOptions::default()
        .with_target(VectorTarget::Weaviate)
        .with_weaviate_class("CodeChunk")
        .with_batch_envelope(true);
    let batch_exporter = ChunkExporter::with_options(batch_options);
    let batch_json = batch_exporter
        .export_batch_json(&[chunk])
        .expect("Weaviate batch");
    let batch: WeaviateBatch = serde_json::from_str(&batch_json).expect("Parse Weaviate batch");
    assert_eq!(batch.objects.len(), 1);
    assert_eq!(batch.objects[0].class, "CodeChunk");
}

#[test]
fn test_openai_batch_embeddings_export() {
    let chunk = create_test_code_chunk();

    let options = ChunkExportOptions::default()
        .with_target(VectorTarget::OpenAiBatch)
        .with_embedding_model("text-embedding-3-large");
    let exporter = ChunkExporter::with_options(options);
    let (output, _) = exporter
        .export_to_string(std::slice::from_ref(&chunk))
        .expect("OpenAI batch export");

    let req: OpenAiBatchRequest =
        serde_json::from_str(output.trim()).expect("Parse OpenAI batch request");
    assert_eq!(req.custom_id, "chk_vec_s1_c0_f9e8d7c6");
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "/v1/embeddings");
    assert_eq!(req.body.model, "text-embedding-3-large");
    assert_eq!(req.body.input, chunk.content);
}

#[test]
fn test_cohere_record_export() {
    let chunk = create_test_code_chunk();

    let output =
        export_chunks_to_vector_target(std::slice::from_ref(&chunk), VectorTarget::CohereBatch)
            .expect("Cohere export");
    let rec: CohereRecord = serde_json::from_str(output.trim()).expect("Parse Cohere record");

    assert_eq!(rec.id, "chk_vec_s1_c0_f9e8d7c6");
    assert_eq!(rec.text, chunk.content);
    assert!(rec.metadata.is_some());
}

#[test]
fn test_schema_validator_rejections() {
    let validator = VectorSchemaValidator::new();
    let valid_chunk = create_test_code_chunk();

    // 1. Empty chunk ID rejection
    let mut invalid_chunk = valid_chunk.clone();
    invalid_chunk.chunk_id = ChunkId::new("");
    let err = validator
        .validate_chunk(
            &invalid_chunk,
            VectorTarget::GenericJsonl,
            false,
            "Chunk",
            "model",
        )
        .unwrap_err();
    assert!(err.to_string().contains("chunk ID must not be empty"));

    // 2. Empty content payload rejection
    let mut invalid_chunk2 = valid_chunk.clone();
    invalid_chunk2.content = String::new();
    let err2 = validator
        .validate_chunk(
            &invalid_chunk2,
            VectorTarget::GenericJsonl,
            false,
            "Chunk",
            "model",
        )
        .unwrap_err();
    assert!(err2.to_string().contains("empty content payload"));

    // 3. Weaviate invalid class (lowercase initial)
    let err3 = validator
        .validate_chunk(
            &valid_chunk,
            VectorTarget::Weaviate,
            true,
            "invalidClass",
            "model",
        )
        .unwrap_err();
    assert!(err3
        .to_string()
        .contains("must start with an uppercase ASCII letter"));

    // 4. OpenAI Batch context overflow (> 8191 tokens)
    let mut oversized_chunk = valid_chunk;
    oversized_chunk.token_count = 9000;
    let err4 = validator
        .validate_chunk(
            &oversized_chunk,
            VectorTarget::OpenAiBatch,
            false,
            "Chunk",
            "model",
        )
        .unwrap_err();
    assert!(err4
        .to_string()
        .contains("exceeding OpenAI maximum embedding context"));
}
