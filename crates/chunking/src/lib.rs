//! # Chunking Engine (`web-repo-to-ai-chunking`)
//!
//! Splits documents along semantic and syntactic boundaries:
//! - Heading-aware and section-aware markdown splitting
//! - Function, struct, class, and method AST boundary chunking
//! - Structured table header retention and spec partitioning
//! - Configurable token overlap and citation metadata injection
//! - Deterministic chunk ID and hash generation

#![warn(missing_docs)]

pub mod ast;
pub mod citation;
pub mod cross_ref;
pub mod engine;
pub mod error;
pub mod export;
pub mod golden;
pub mod markdown;
pub mod model;
pub mod structured;
pub mod token;
pub mod window;

pub use ast::chunk_code_section;
pub use citation::generate_chunk_citation;
pub use cross_ref::CrossReferenceResolver;
pub use engine::{chunk_document, chunk_text, Chunker};
pub use error::{ChunkResult, ChunkingError};
pub use export::{
    export_chunks_to_jsonl, export_chunks_to_vector_target, is_valid_uuid, to_deterministic_uuid,
    ChromaBatchCollection, ChromaRecord, ChunkExportMetrics, ChunkExportOptions, ChunkExporter,
    CohereRecord, GenericRagRecord, MilvusEntity, OpenAiBatchBody, OpenAiBatchRequest,
    PineconeBatch, PineconeRecord, QdrantBatch, QdrantPoint, VectorSchemaValidator, VectorTarget,
    WeaviateBatch, WeaviateObject,
};
pub use golden::{
    evaluate_golden_fixture, get_all_golden_fixtures, run_all_golden_benchmarks,
    score_chunk_relevance, ChunkingArchetype, ChunkingFidelityMetrics, ChunkingGoldenFixture,
    ChunkingGoldenReport, ChunkingGoldenResult, RetrievalBenchmarkQuery,
};
pub use markdown::chunk_markdown_section;
pub use model::{
    Chunk, ChunkCitation, ChunkContext, ChunkCrossReference, ChunkId, ChunkKind,
    ChunkSymbolMetadata, ChunkingConfig, ChunkingReport, ChunkingResult, ChunkingStrategy,
    CrossReferenceKind,
};
pub use structured::chunk_structured_section;
pub use token::TokenCounter;
pub use window::chunk_sliding_window;

/// Returns the unique crate identifier name.
#[must_use]
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking_metadata() {
        assert_eq!(crate_name(), "web-repo-to-ai-chunking");
    }
}
