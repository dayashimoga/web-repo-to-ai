//! # Static HTML DOM Parsing & Boilerplate Removal Subsystem
//!
//! Provides WHATWG-compliant HTML parsing, document metadata extraction, relative link canonicalization,
//! and heuristic boilerplate removal while strictly preserving content hierarchies, tables, and code blocks.

pub mod cleaner;
pub mod link_resolver;
pub mod metadata;
pub mod parser;
pub mod policy;

pub use cleaner::BoilerplateCleaner;
pub use link_resolver::LinkResolver;
pub use metadata::DocumentMetadata;
pub use parser::{CleanedDocument, DomParser};
pub use policy::BoilerplatePolicy;
