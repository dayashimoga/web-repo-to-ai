//! # Source Attribution, HTTP Provenance, and Metadata Subsystem
//!
//! Provides comprehensive provenance tracking:
//! - Source attribution, canonical URL resolution, and citation generation
//! - HTTP transport headers, redirect transitions, status codes, and pinned IPs
//! - Dual SHA-256 and BLAKE3 cryptographic content hashes
//! - Rich metadata extraction from `<head>`, OpenGraph, Twitter Cards, and Schema.org JSON-LD
//! - Diagnostic warnings and extraction anomaly classification

pub mod extractor;
pub mod model;

pub use extractor::{MetadataExtractor, ProvenanceBuilder};
pub use model::{
    BrowserDiagnostics, CapturedMetadata, ContentHashes, HttpProvenance, ProvenanceCitations,
    ProvenanceWarning, RedirectHopRecord, SourceAttribution, WarningSeverity,
};
