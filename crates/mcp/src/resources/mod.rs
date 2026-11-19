//! # Built-in MCP Resource Providers
//!
//! Production Model Context Protocol resource providers for converted documents (`cdm://`),
//! completeness reports, archive packages, and local artifact files (`file://`, `artifact://`).

pub mod base64;
pub mod cdm;
pub mod file;
pub mod store;

use std::sync::Arc;

pub use base64::{decode_base64, encode_base64};
pub use cdm::CdmResourceProvider;
pub use file::FileResourceProvider;
pub use store::{DocumentSummary, McpDocumentStore};

use crate::server::registry::ServerRegistry;

/// Registers all default production resource providers into the provided registry.
pub fn register_default_resources(registry: &mut ServerRegistry, store: Arc<McpDocumentStore>) {
    registry.register_resource_provider(Arc::new(CdmResourceProvider::new(store)));
    registry.register_resource_provider(Arc::new(FileResourceProvider::default()));
}
