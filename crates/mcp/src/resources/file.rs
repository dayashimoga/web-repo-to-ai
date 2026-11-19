//! # Local File & Artifact Resource Provider
//!
//! Provides secure Model Context Protocol access to exported local artifacts and documents
//! under `file://` and `artifact://` URI schemes, strictly enforcing path traversal boundaries
//! against a configured allowed root directory.

use async_trait::async_trait;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info};

use crate::error::McpError;
use crate::protocol::types::{ReadResourceResult, Resource, ResourceContents, ResourceTemplate};
use crate::resources::base64::encode_base64;
use crate::server::registry::DynamicResourceProvider;

/// Dynamic resource provider for filesystem files and exported package artifacts.
#[derive(Debug, Clone)]
pub struct FileResourceProvider {
    allowed_root: PathBuf,
}

impl Default for FileResourceProvider {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

impl FileResourceProvider {
    /// Creates a new provider restricted strictly to the specified root directory.
    #[must_use]
    pub fn new(allowed_root: impl Into<PathBuf>) -> Self {
        let root = allowed_root.into();
        let canonical_root = root.canonicalize().unwrap_or(root);
        Self {
            allowed_root: canonical_root,
        }
    }

    /// Helper extracting and validating relative path from URI.
    fn resolve_safe_path(&self, uri: &str) -> Result<PathBuf, McpError> {
        let raw_path = if let Some(p) = uri.strip_prefix("file://") {
            p
        } else if let Some(p) = uri.strip_prefix("artifact://") {
            p
        } else {
            return Err(McpError::ResourceError(format!(
                "Unsupported file URI scheme in '{uri}'"
            )));
        };

        let decoded_path = percent_decode(raw_path);
        if decoded_path.contains('\0') {
            return Err(McpError::ResourceError(
                "Access denied: null byte in path forbidden".to_string(),
            ));
        }

        // Trim leading slashes for Windows / Unix normalization
        let trimmed_path = decoded_path.trim_start_matches('/');
        let target_path = self.allowed_root.join(trimmed_path);

        let canonical_target = target_path.canonicalize().map_err(|e| {
            McpError::ResourceError(format!("Cannot resolve file '{decoded_path}': {e}"))
        })?;

        // Strict boundary defense: target must be inside allowed_root
        if !canonical_target.starts_with(&self.allowed_root) {
            error!(
                target: "mcp::resources::file",
                path = %decoded_path,
                root = %self.allowed_root.display(),
                "Path traversal rejected"
            );
            return Err(McpError::ResourceError(
                "Access denied: path traversal beyond root boundary forbidden".to_string(),
            ));
        }

        if !canonical_target.is_file() {
            return Err(McpError::ResourceError(format!(
                "Target '{decoded_path}' is not a regular file"
            )));
        }

        Ok(canonical_target)
    }

    /// Guesses MIME type from file extension.
    fn guess_mime(path: &Path) -> &'static str {
        match path.extension().and_then(|e| e.to_str()).unwrap_or("") {
            "md" | "markdown" => "text/markdown",
            "txt" | "text" => "text/plain",
            "json" => "application/json",
            "jsonl" => "application/x-ndjson",
            "html" | "htm" => "text/html",
            "zip" => "application/zip",
            "rs" => "text/x-rust",
            "py" => "text/x-python",
            "js" => "application/javascript",
            "ts" => "application/typescript",
            _ => "application/octet-stream",
        }
    }
}

fn percent_decode(input: &str) -> String {
    let mut result = Vec::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(hex_str) = std::str::from_utf8(&bytes[i + 1..i + 3]) {
                if let Ok(byte) = u8::from_str_radix(hex_str, 16) {
                    result.push(byte);
                    i += 3;
                    continue;
                }
            }
        }
        result.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&result).into_owned()
}

#[async_trait]
impl DynamicResourceProvider for FileResourceProvider {
    fn can_handle(&self, uri: &str) -> bool {
        uri.starts_with("file://") || uri.starts_with("artifact://")
    }

    async fn read(&self, uri: &str) -> Result<ReadResourceResult, McpError> {
        debug!(target: "mcp::resources::file", uri = %uri, "Reading filesystem resource");

        let path = self.resolve_safe_path(uri)?;
        let mime = Self::guess_mime(&path);

        let bytes = fs::read(&path).map_err(|e| {
            McpError::ResourceError(format!("Failed to read file '{}': {e}", path.display()))
        })?;

        let contents = match String::from_utf8(bytes) {
            Ok(text_str) => ResourceContents {
                uri: uri.to_string(),
                mime_type: Some(mime.to_string()),
                text: Some(text_str),
                blob: None,
            },
            Err(e) => {
                let raw_bytes = e.into_bytes();
                ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some(mime.to_string()),
                    text: None,
                    blob: Some(encode_base64(&raw_bytes)),
                }
            }
        };

        info!(target: "mcp::resources::file", uri = %uri, "Filesystem resource read successfully");
        Ok(ReadResourceResult {
            contents: vec![contents],
        })
    }

    fn list_resources(&self) -> Vec<Resource> {
        Vec::new()
    }

    fn list_templates(&self) -> Vec<ResourceTemplate> {
        vec![
            ResourceTemplate::new("artifact://{path}", "Exported Local Artifact")
                .with_description("Retrieves exported context artifact or bundle by relative path")
                .with_mime_type("application/octet-stream"),
            ResourceTemplate::new("file://{path}", "Local Monorepo File")
                .with_description(
                    "Retrieves source file from within the permitted workspace boundary",
                )
                .with_mime_type("text/plain"),
        ]
    }
}
