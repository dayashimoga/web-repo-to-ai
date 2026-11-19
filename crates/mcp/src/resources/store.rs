//! # In-Memory Document Store for MCP Resources
//!
//! Provides thread-safe storage, retrieval, and indexing of Canonical Context Documents (`CdmDocument`)
//! generated during conversion sessions or loaded for resource retrieval.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use web_repo_to_ai_cdm::model::CdmDocument;

/// Summary metadata for a stored CDM document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    /// Document UUID string identifier.
    pub document_id: String,
    /// Canonical source URL or repository identifier.
    pub canonical_url: String,
    /// Classified source type (e.g. `webpage_static`, `github_public`).
    pub source_type: String,
    /// Total number of content sections.
    pub section_count: usize,
    /// ISO-8601 UTC timestamp when acquired.
    pub retrieved_at: String,
    /// Document title if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Thread-safe in-memory store for converted documents.
#[derive(Debug, Default, Clone)]
pub struct McpDocumentStore {
    inner: Arc<RwLock<StoreInner>>,
}

#[derive(Debug, Default)]
struct StoreInner {
    by_id: HashMap<String, CdmDocument>,
    url_to_id: HashMap<String, String>,
}

impl McpDocumentStore {
    /// Creates an empty document store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stores a `CdmDocument` and returns its canonical document ID.
    pub fn insert(&self, doc: CdmDocument) -> String {
        let id_str = doc.document_id.to_string();
        let url = doc.canonical_url.clone();

        let mut inner = self.inner.write().expect("lock not poisoned");
        inner.url_to_id.insert(url, id_str.clone());
        inner.by_id.insert(id_str.clone(), doc);
        id_str
    }

    /// Retrieves a document by its document ID or canonical URL.
    #[must_use]
    pub fn get(&self, id_or_url: &str) -> Option<CdmDocument> {
        let inner = self.inner.read().expect("lock not poisoned");
        if let Some(doc) = inner.by_id.get(id_or_url) {
            return Some(doc.clone());
        }
        if let Some(id) = inner.url_to_id.get(id_or_url) {
            return inner.by_id.get(id).cloned();
        }
        None
    }

    /// Returns summaries of all documents currently held in the store.
    #[must_use]
    pub fn list(&self) -> Vec<DocumentSummary> {
        let inner = self.inner.read().expect("lock not poisoned");
        inner
            .by_id
            .values()
            .map(|doc| DocumentSummary {
                document_id: doc.document_id.to_string(),
                canonical_url: doc.canonical_url.clone(),
                source_type: doc.source_type.as_str().to_string(),
                section_count: doc.sections.len(),
                retrieved_at: doc.retrieved_at.to_rfc3339(),
                title: doc.metadata.title.clone(),
            })
            .collect()
    }

    /// Removes a document by ID.
    pub fn remove(&self, id: &str) -> Option<CdmDocument> {
        let mut inner = self.inner.write().expect("lock not poisoned");
        if let Some(doc) = inner.by_id.remove(id) {
            inner.url_to_id.remove(&doc.canonical_url);
            Some(doc)
        } else {
            None
        }
    }

    /// Clears all stored documents.
    pub fn clear(&self) {
        let mut inner = self.inner.write().expect("lock not poisoned");
        inner.by_id.clear();
        inner.url_to_id.clear();
    }

    /// Returns the number of documents currently stored.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.read().expect("lock not poisoned").by_id.len()
    }

    /// Returns true if store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner
            .read()
            .expect("lock not poisoned")
            .by_id
            .is_empty()
    }
}
