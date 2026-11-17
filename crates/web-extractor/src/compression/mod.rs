//! # Compression & Decompression Bomb Protection Subsystem
//!
//! Provides bounded decompression for HTTP `Content-Encoding` formats (`gzip`, `deflate`, `br`, `zstd`)
//! and defends against denial-of-service compression bombs and memory exhaustion attacks.

pub mod decompressor;
pub mod encoding;
pub mod limits;

pub use decompressor::BoundedDecompressor;
pub use encoding::ContentEncoding;
pub use limits::BodyLimits;
