//! # Local Filesystem Artifact Storage Backend
//!
//! Provides durable, atomic, and traversal-guarded local filesystem storage
//! for conversion artifacts, Canonical Context Documents (CDM), and blob caches.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use web_repo_to_ai_core::config::StorageConfig;
use web_repo_to_ai_core::error::CoreError;

use crate::schema::ArtifactFormat;

/// Windows reserved device names that must not be used as path components.
const WINDOWS_RESERVED_NAMES: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Standard storage key formatting helpers for conversion jobs and blobs.
pub mod layout {
    use super::*;

    /// Constructs the standard relative storage key for a job's exported artifact.
    #[must_use]
    pub fn job_artifact_key(job_id: Uuid, format: ArtifactFormat) -> String {
        format!("jobs/{job_id}/artifact.{}", format.file_extension())
    }

    /// Constructs the standard relative directory key for all artifacts belonging to a job.
    #[must_use]
    pub fn job_dir_key(job_id: Uuid) -> String {
        format!("jobs/{job_id}")
    }

    /// Constructs the standard relative storage key for a batch's exported consolidated artifact.
    #[must_use]
    pub fn batch_artifact_key(batch_id: Uuid, extension: &str) -> String {
        format!("batches/{batch_id}/batch.{extension}")
    }

    /// Constructs the standard relative directory key for all artifacts belonging to a batch.
    #[must_use]
    pub fn batch_dir_key(batch_id: Uuid) -> String {
        format!("batches/{batch_id}")
    }

    /// Constructs the standard two-tier sharded storage key for a content-addressed blob.
    #[must_use]
    pub fn blob_key(blake3_hex: &str) -> String {
        let prefix = if blake3_hex.len() >= 2 {
            &blake3_hex[..2]
        } else {
            "00"
        };
        format!("blobs/{prefix}/{blake3_hex}.bin")
    }
}

/// Metadata describing a stored artifact file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactFileMetadata {
    /// Relative storage key within the storage root.
    pub key: String,
    /// File size in bytes.
    pub byte_size: u64,
    /// Timestamp when the file was created, if available from the filesystem.
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp when the file was last modified.
    pub modified_at: DateTime<Utc>,
    /// Whether the file is marked read-only.
    pub is_readonly: bool,
}

/// Disk usage statistics across stored artifacts.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StorageUsage {
    /// Total bytes consumed by stored artifact files.
    pub total_artifact_bytes: u64,
    /// Total number of stored artifact files.
    pub artifact_count: usize,
}

/// Local filesystem storage backend with atomic write semantics and path traversal defenses.
#[derive(Debug, Clone)]
pub struct LocalFsStorage {
    base_dir: PathBuf,
    max_storage_bytes: u64,
}

impl LocalFsStorage {
    /// Instantiates a new `LocalFsStorage` targeting the given base directory.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if `base_dir` path is empty.
    pub fn new(base_dir: impl Into<PathBuf>) -> Result<Self, CoreError> {
        let base_dir = base_dir.into();
        if base_dir.as_os_str().is_empty() {
            return Err(CoreError::Validation {
                field: "base_dir".to_string(),
                reason: "base_dir cannot be empty".to_string(),
            });
        }

        Ok(Self {
            base_dir,
            max_storage_bytes: 15 * 1024 * 1024 * 1024, // 15 GB default
        })
    }

    /// Instantiates a `LocalFsStorage` from the shared application [`StorageConfig`].
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if `base_dir` path is empty.
    pub fn with_config(config: &StorageConfig) -> Result<Self, CoreError> {
        let mut storage = Self::new(&config.base_dir)?;
        storage.max_storage_bytes = config.max_storage_bytes;
        Ok(storage)
    }

    /// Instantiates a `LocalFsStorage` with explicit maximum storage limits.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Validation`] if `base_dir` path is empty.
    pub fn with_limits(
        base_dir: impl Into<PathBuf>,
        max_storage_bytes: u64,
    ) -> Result<Self, CoreError> {
        let mut storage = Self::new(base_dir)?;
        storage.max_storage_bytes = max_storage_bytes;
        Ok(storage)
    }

    /// Returns the configured base directory path.
    #[must_use]
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Returns the maximum allowable storage budget in bytes.
    #[must_use]
    pub const fn max_storage_bytes(&self) -> u64 {
        self.max_storage_bytes
    }

    /// Validates a relative storage key and resolves it to a safe absolute filesystem path.
    ///
    /// # Path Traversal Defenses
    ///
    /// - Rejects null bytes (`\0`).
    /// - Rejects absolute paths (leading `/`, `\`, or Windows drive letters like `C:`).
    /// - Rejects parent traversal components (`..`) or current directory references (`.`).
    /// - Rejects Windows reserved device names (`CON`, `PRN`, `AUX`, `NUL`, `COM1-9`, `LPT1-9`).
    /// - Normalizes path separators and ensures the resolved path is strictly confined to `base_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Security`] if path traversal or illegal components are detected.
    /// Returns [`CoreError::Validation`] if the key is empty.
    pub fn resolve_path(&self, key: &str) -> Result<PathBuf, CoreError> {
        let key_trimmed = key.trim();
        if key_trimmed.is_empty() {
            return Err(CoreError::Validation {
                field: "key".to_string(),
                reason: "Storage key cannot be empty".to_string(),
            });
        }

        if key.contains('\0') {
            return Err(CoreError::Security(
                "Null byte detected in storage key".to_string(),
            ));
        }

        let raw_path = Path::new(key_trimmed);

        // Disallow absolute paths
        if raw_path.is_absolute() {
            return Err(CoreError::Security(format!(
                "Absolute paths are forbidden in storage keys: '{key}'"
            )));
        }

        let mut sanitized = PathBuf::new();
        for component in raw_path.components() {
            match component {
                Component::Normal(c) => {
                    let s = c.to_string_lossy();

                    // Check for Windows reserved device names
                    let stem = Path::new(&*s)
                        .file_stem()
                        .map(|stem| stem.to_string_lossy().to_ascii_lowercase())
                        .unwrap_or_default();

                    if WINDOWS_RESERVED_NAMES.contains(&stem.as_str()) {
                        return Err(CoreError::Security(format!(
                            "Storage key contains Windows reserved device name '{s}': '{key}'"
                        )));
                    }

                    // Check for invalid trailing spaces or periods (Windows-specific file hijacking)
                    if s.ends_with(' ') || s.ends_with('.') {
                        return Err(CoreError::Security(format!(
                            "Path component has trailing dot or space: '{s}' in '{key}'"
                        )));
                    }

                    sanitized.push(c);
                }
                Component::ParentDir => {
                    return Err(CoreError::Security(format!(
                        "Path traversal attempt ('..') detected in storage key: '{key}'"
                    )));
                }
                Component::CurDir => {
                    // Skip current directory references safely
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(CoreError::Security(format!(
                        "Root or drive prefix forbidden in storage key: '{key}'"
                    )));
                }
            }
        }

        if sanitized.as_os_str().is_empty() {
            return Err(CoreError::Validation {
                field: "key".to_string(),
                reason: "Storage key resolves to empty relative path".to_string(),
            });
        }

        Ok(self.base_dir.join(sanitized))
    }

    /// Atomically writes data to the target storage key.
    ///
    /// Writes content to a unique staging file in the target directory (`.<name>.tmp.<uuid>`),
    /// calls `sync_all()` to flush to storage media, and atomically renames the staging file
    /// into the destination path. If any error occurs, the temporary file is removed.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::PayloadTooLarge`] if `data.len()` exceeds `max_storage_bytes`.
    /// Returns [`CoreError::Security`] if the key violates path safety rules.
    /// Returns [`CoreError::Storage`] if filesystem operations fail.
    pub async fn put(&self, key: &str, data: &[u8]) -> Result<u64, CoreError> {
        let byte_len = data.len() as u64;
        if byte_len > self.max_storage_bytes {
            return Err(CoreError::PayloadTooLarge {
                actual_bytes: byte_len,
                max_bytes: self.max_storage_bytes,
            });
        }

        let dest_path = self.resolve_path(key)?;

        let parent_dir = dest_path.parent().ok_or_else(|| {
            CoreError::Storage(format!("Cannot determine parent directory for '{key}'"))
        })?;

        tokio::fs::create_dir_all(parent_dir).await.map_err(|err| {
            CoreError::Storage(format!(
                "Failed to create storage directory '{}': {err}",
                parent_dir.display()
            ))
        })?;

        let file_name = dest_path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "artifact".to_string());

        let temp_file_name = format!(".{file_name}.tmp.{}", Uuid::now_v7());
        let temp_path = parent_dir.join(temp_file_name);

        // Staging write scope
        let write_result: Result<(), std::io::Error> = async {
            let mut file = tokio::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp_path)
                .await?;

            file.write_all(data).await?;
            file.sync_all().await?;
            drop(file);
            Ok(())
        }
        .await;

        if let Err(err) = write_result {
            let _ = tokio::fs::remove_file(&temp_path).await;
            return Err(CoreError::Storage(format!(
                "Failed to write artifact staging file for '{key}': {err}"
            )));
        }

        // Atomic rename / replace
        if let Err(rename_err) = tokio::fs::rename(&temp_path, &dest_path).await {
            // Windows fallback: if destination exists, remove and retry rename
            let _ = tokio::fs::remove_file(&dest_path).await;
            if let Err(retry_err) = tokio::fs::rename(&temp_path, &dest_path).await {
                let _ = tokio::fs::remove_file(&temp_path).await;
                return Err(CoreError::Storage(format!(
                    "Failed to atomically rename artifact to '{}': primary error: {rename_err}, fallback error: {retry_err}",
                    dest_path.display()
                )));
            }
        }

        Ok(byte_len)
    }

    /// Reads the entire contents of a stored artifact into memory.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact does not exist.
    /// Returns [`CoreError::Security`] if the key violates safety rules.
    /// Returns [`CoreError::Storage`] if read operations fail.
    pub async fn get(&self, key: &str) -> Result<Vec<u8>, CoreError> {
        let path = self.resolve_path(key)?;

        match tokio::fs::read(&path).await {
            Ok(bytes) => Ok(bytes),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(CoreError::NotFound {
                resource_type: "artifact",
                id: key.to_string(),
            }),
            Err(err) => Err(CoreError::Storage(format!(
                "Failed to read artifact at '{key}': {err}"
            ))),
        }
    }

    /// Opens an artifact file for asynchronous streaming reads.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact does not exist.
    /// Returns [`CoreError::Security`] if the key violates safety rules.
    /// Returns [`CoreError::Storage`] if open fails.
    pub async fn open_read(&self, key: &str) -> Result<tokio::fs::File, CoreError> {
        let path = self.resolve_path(key)?;

        match tokio::fs::File::open(&path).await {
            Ok(file) => Ok(file),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Err(CoreError::NotFound {
                resource_type: "artifact",
                id: key.to_string(),
            }),
            Err(err) => Err(CoreError::Storage(format!(
                "Failed to open artifact file for reading at '{key}': {err}"
            ))),
        }
    }

    /// Checks whether an artifact file exists at the given storage key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Security`] if the key violates safety rules.
    /// Returns [`CoreError::Storage`] if query fails.
    pub async fn exists(&self, key: &str) -> Result<bool, CoreError> {
        let path = self.resolve_path(key)?;
        match tokio::fs::metadata(&path).await {
            Ok(meta) => Ok(meta.is_file()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(CoreError::Storage(format!(
                "Failed to inspect artifact existence at '{key}': {err}"
            ))),
        }
    }

    /// Deletes a stored artifact file.
    ///
    /// Returns `Ok(true)` if the file existed and was removed, or `Ok(false)` if it did not exist.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Security`] if the key violates safety rules.
    /// Returns [`CoreError::Storage`] if deletion fails.
    pub async fn delete(&self, key: &str) -> Result<bool, CoreError> {
        let path = self.resolve_path(key)?;
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(CoreError::Storage(format!(
                "Failed to delete artifact file '{key}': {err}"
            ))),
        }
    }

    /// Recursively removes an entire directory of artifacts (e.g. all artifacts for a job).
    ///
    /// Returns `Ok(true)` if the directory existed and was deleted, or `Ok(false)` if it did not exist.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Security`] if the prefix violates safety rules.
    /// Returns [`CoreError::Storage`] if deletion fails.
    pub async fn delete_directory(&self, prefix: &str) -> Result<bool, CoreError> {
        let path = self.resolve_path(prefix)?;
        match tokio::fs::remove_dir_all(&path).await {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(CoreError::Storage(format!(
                "Failed to delete artifact directory '{prefix}': {err}"
            ))),
        }
    }

    /// Retrieves filesystem metadata for a stored artifact.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact is missing.
    /// Returns [`CoreError::Security`] if the key violates safety rules.
    /// Returns [`CoreError::Storage`] if metadata inspection fails.
    pub async fn metadata(&self, key: &str) -> Result<ArtifactFileMetadata, CoreError> {
        let path = self.resolve_path(key)?;

        let meta = match tokio::fs::metadata(&path).await {
            Ok(meta) => meta,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(CoreError::NotFound {
                    resource_type: "artifact",
                    id: key.to_string(),
                });
            }
            Err(err) => {
                return Err(CoreError::Storage(format!(
                    "Failed to read metadata for '{key}': {err}"
                )));
            }
        };

        let modified_at = meta
            .modified()
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(|_| Utc::now());

        let created_at = meta.created().ok().map(DateTime::<Utc>::from);

        Ok(ArtifactFileMetadata {
            key: key.to_string(),
            byte_size: meta.len(),
            created_at,
            modified_at,
            is_readonly: meta.permissions().readonly(),
        })
    }

    /// Recursively lists all stored artifact keys matching an optional directory prefix.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if directory traversal fails.
    pub async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, CoreError> {
        let target_dir = match prefix {
            Some(p) => self.resolve_path(p)?,
            None => self.base_dir.clone(),
        };

        if !tokio::fs::try_exists(&target_dir).await.unwrap_or(false) {
            return Ok(Vec::new());
        }

        let mut results = Vec::new();
        let mut dirs_to_visit = vec![target_dir];

        while let Some(current_dir) = dirs_to_visit.pop() {
            let mut read_dir = match tokio::fs::read_dir(&current_dir).await {
                Ok(entries) => entries,
                Err(err) => {
                    return Err(CoreError::Storage(format!(
                        "Failed to read directory '{}': {err}",
                        current_dir.display()
                    )));
                }
            };

            while let Some(entry) = read_dir.next_entry().await.map_err(|err| {
                CoreError::Storage(format!("Failed to read directory entry: {err}"))
            })? {
                let file_type = entry.file_type().await.map_err(|err| {
                    CoreError::Storage(format!("Failed to query file type: {err}"))
                })?;

                let path = entry.path();
                if file_type.is_dir() {
                    dirs_to_visit.push(path);
                } else if file_type.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    // Ignore internal staging files
                    if !file_name.starts_with('.') {
                        if let Ok(rel) = path.strip_prefix(&self.base_dir) {
                            let key = rel.to_string_lossy().replace('\\', "/");
                            results.push(key);
                        }
                    }
                }
            }
        }

        results.sort();
        Ok(results)
    }

    /// Computes the total byte capacity consumed by stored artifacts in `base_dir`.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if directory inspection fails.
    pub async fn storage_capacity(&self) -> Result<StorageUsage, CoreError> {
        if !tokio::fs::try_exists(&self.base_dir).await.unwrap_or(false) {
            return Ok(StorageUsage::default());
        }

        let mut usage = StorageUsage::default();
        let mut dirs_to_visit = vec![self.base_dir.clone()];

        while let Some(current_dir) = dirs_to_visit.pop() {
            let mut read_dir = match tokio::fs::read_dir(&current_dir).await {
                Ok(entries) => entries,
                Err(err) => {
                    return Err(CoreError::Storage(format!(
                        "Failed to read directory '{}': {err}",
                        current_dir.display()
                    )));
                }
            };

            while let Some(entry) = read_dir.next_entry().await.map_err(|err| {
                CoreError::Storage(format!("Failed to read directory entry: {err}"))
            })? {
                let file_type = entry.file_type().await.map_err(|err| {
                    CoreError::Storage(format!("Failed to query entry type: {err}"))
                })?;

                let path = entry.path();
                if file_type.is_dir() {
                    dirs_to_visit.push(path);
                } else if file_type.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if !file_name.starts_with('.') {
                        if let Ok(meta) = entry.metadata().await {
                            usage.total_artifact_bytes += meta.len();
                            usage.artifact_count += 1;
                        }
                    }
                }
            }
        }

        Ok(usage)
    }

    /// Scans the storage directory for stale temporary staging files (`.*.tmp.*`) older than `max_age` and removes them.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if directory scanning fails.
    pub async fn cleanup_stale_staging_files(
        &self,
        max_age: chrono::Duration,
    ) -> Result<(usize, u64), CoreError> {
        let base_dir = &self.base_dir;
        if !tokio::fs::try_exists(base_dir).await.unwrap_or(false) {
            return Ok((0, 0));
        }

        let mut deleted_count = 0;
        let mut bytes_freed = 0;
        let mut dirs_to_visit = vec![base_dir.clone()];
        let now = Utc::now();

        while let Some(current_dir) = dirs_to_visit.pop() {
            let mut read_dir = match tokio::fs::read_dir(&current_dir).await {
                Ok(entries) => entries,
                Err(err) => {
                    return Err(CoreError::Storage(format!(
                        "Failed to read directory '{}' during staging cleanup: {err}",
                        current_dir.display()
                    )));
                }
            };

            while let Some(entry) = read_dir.next_entry().await.map_err(|err| {
                CoreError::Storage(format!(
                    "Failed to read directory entry during staging cleanup: {err}"
                ))
            })? {
                let file_type = entry.file_type().await.map_err(|err| {
                    CoreError::Storage(format!("Failed to query file type: {err}"))
                })?;

                let path = entry.path();
                if file_type.is_dir() {
                    dirs_to_visit.push(path);
                } else if file_type.is_file() {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    if file_name.starts_with('.') && file_name.contains(".tmp.") {
                        if let Ok(meta) = entry.metadata().await {
                            let modified_at =
                                meta.modified().map(DateTime::<Utc>::from).unwrap_or(now);
                            let age = now.signed_duration_since(modified_at);
                            if age >= max_age {
                                let size = meta.len();
                                if tokio::fs::remove_file(&path).await.is_ok() {
                                    deleted_count += 1;
                                    bytes_freed += size;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok((deleted_count, bytes_freed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// RAII guard creating and reliably removing an isolated test directory.
    struct TestDir(PathBuf);

    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("web-repo-to-ai-storage-test-{}", Uuid::now_v7()));
            std::fs::create_dir_all(&path).expect("Failed to create isolated test dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[tokio::test]
    async fn test_local_fs_put_get_exists_delete() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        let key = "jobs/test-job/artifact.md";
        let content = b"# Markdown Header\n\nContent for testing.";

        assert!(!storage.exists(key).await.expect("Exists check failed"));

        let bytes_written = storage.put(key, content).await.expect("Put failed");
        assert_eq!(bytes_written, content.len() as u64);

        assert!(storage.exists(key).await.expect("Exists check failed"));

        let read_back = storage.get(key).await.expect("Get failed");
        assert_eq!(read_back, content);

        let meta = storage.metadata(key).await.expect("Metadata query failed");
        assert_eq!(meta.key, key);
        assert_eq!(meta.byte_size, content.len() as u64);

        let deleted = storage.delete(key).await.expect("Delete failed");
        assert!(deleted);
        assert!(!storage.exists(key).await.expect("Exists check failed"));

        // Deleting non-existent file returns false
        let delete_again = storage.delete(key).await.expect("Delete failed");
        assert!(!delete_again);
    }

    #[tokio::test]
    async fn test_local_fs_atomic_overwrite() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        let key = "jobs/test-job/output.txt";
        let v1 = b"Version 1 content";
        let v2 = b"Version 2 updated content with more bytes";

        storage.put(key, v1).await.expect("Put v1 failed");
        assert_eq!(storage.get(key).await.expect("Get failed"), v1);

        // Atomic overwrite
        storage.put(key, v2).await.expect("Put v2 failed");
        assert_eq!(storage.get(key).await.expect("Get failed"), v2);
    }

    #[test]
    fn test_local_fs_path_traversal_rejections() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        // Traversal attempts
        assert!(storage.resolve_path("../outside.txt").is_err());
        assert!(storage.resolve_path("../../etc/passwd").is_err());
        assert!(storage.resolve_path("jobs/../../secret.key").is_err());
        assert!(storage.resolve_path("a/b/../../../c").is_err());

        // Absolute path attempts
        assert!(storage.resolve_path("/root/file.txt").is_err());
        assert!(storage.resolve_path("C:\\Windows\\system32").is_err());
        assert!(storage.resolve_path("\\\\server\\share\\file").is_err());

        // Null byte
        assert!(storage.resolve_path("jobs/test\0file.txt").is_err());

        // Reserved Windows device names
        assert!(storage.resolve_path("con").is_err());
        assert!(storage.resolve_path("jobs/con/file.txt").is_err());
        assert!(storage.resolve_path("nul.txt").is_err());
        assert!(storage.resolve_path("aux.json").is_err());
        assert!(storage.resolve_path("prn").is_err());

        // Empty / blank
        assert!(storage.resolve_path("").is_err());
        assert!(storage.resolve_path("   ").is_err());

        // Valid safe relative keys
        assert!(storage.resolve_path("jobs/job-1/output.md").is_ok());
        assert!(storage.resolve_path("blobs/ab/abcdef123456.bin").is_ok());
    }

    #[tokio::test]
    async fn test_local_fs_delete_directory() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        let job_id = Uuid::now_v7();
        let key1 = format!("jobs/{job_id}/artifact.md");
        let key2 = format!("jobs/{job_id}/artifact.json");
        let other_key = "jobs/other-job/artifact.txt";

        storage
            .put(&key1, b"md content")
            .await
            .expect("Put key1 failed");
        storage
            .put(&key2, b"json content")
            .await
            .expect("Put key2 failed");
        storage
            .put(other_key, b"other content")
            .await
            .expect("Put other failed");

        let job_dir = format!("jobs/{job_id}");
        let deleted = storage
            .delete_directory(&job_dir)
            .await
            .expect("Delete dir failed");
        assert!(deleted);

        assert!(!storage.exists(&key1).await.expect("Exists failed"));
        assert!(!storage.exists(&key2).await.expect("Exists failed"));
        assert!(storage.exists(other_key).await.expect("Exists failed"));
    }

    #[tokio::test]
    async fn test_local_fs_list_and_capacity() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        let initial_usage = storage.storage_capacity().await.expect("Capacity failed");
        assert_eq!(initial_usage.artifact_count, 0);
        assert_eq!(initial_usage.total_artifact_bytes, 0);

        let k1 = "alpha/doc1.txt";
        let k2 = "alpha/doc2.txt";
        let k3 = "beta/doc3.txt";

        storage.put(k1, b"12345").await.expect("Put k1 failed"); // 5 bytes
        storage.put(k2, b"1234567890").await.expect("Put k2 failed"); // 10 bytes
        storage
            .put(k3, b"123456789012345")
            .await
            .expect("Put k3 failed"); // 15 bytes

        let usage = storage.storage_capacity().await.expect("Capacity failed");
        assert_eq!(usage.artifact_count, 3);
        assert_eq!(usage.total_artifact_bytes, 30);

        let all_keys = storage.list(None).await.expect("List all failed");
        assert_eq!(all_keys, vec![k1, k2, k3]);

        let alpha_keys = storage
            .list(Some("alpha"))
            .await
            .expect("List alpha failed");
        assert_eq!(alpha_keys, vec![k1, k2]);
    }

    #[tokio::test]
    async fn test_local_fs_not_found_errors() {
        let test_dir = TestDir::new();
        let storage = LocalFsStorage::new(test_dir.path()).expect("Valid storage initialization");

        let err = storage.get("non-existent-key.txt").await.unwrap_err();
        match err {
            CoreError::NotFound { resource_type, id } => {
                assert_eq!(resource_type, "artifact");
                assert_eq!(id, "non-existent-key.txt");
            }
            other => panic!("Expected NotFound, got: {other:?}"),
        }

        let open_err = storage.open_read("non-existent-key.txt").await.unwrap_err();
        match open_err {
            CoreError::NotFound { resource_type, id } => {
                assert_eq!(resource_type, "artifact");
                assert_eq!(id, "non-existent-key.txt");
            }
            other => panic!("Expected NotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_local_fs_payload_too_large() {
        let test_dir = TestDir::new();
        // 100 byte limit
        let storage = LocalFsStorage::with_limits(test_dir.path(), 100).expect("Init failed");

        let small = vec![0u8; 50];
        let large = vec![0u8; 150];

        assert!(storage.put("small.bin", &small).await.is_ok());

        let err = storage.put("large.bin", &large).await.unwrap_err();
        match err {
            CoreError::PayloadTooLarge {
                actual_bytes,
                max_bytes,
            } => {
                assert_eq!(actual_bytes, 150);
                assert_eq!(max_bytes, 100);
            }
            other => panic!("Expected PayloadTooLarge, got: {other:?}"),
        }
    }

    #[test]
    fn test_layout_key_generation() {
        let job_id = Uuid::now_v7();
        let md_key = layout::job_artifact_key(job_id, ArtifactFormat::Markdown);
        assert_eq!(md_key, format!("jobs/{job_id}/artifact.md"));

        let zip_key = layout::job_artifact_key(job_id, ArtifactFormat::Zip);
        assert_eq!(zip_key, format!("jobs/{job_id}/artifact.zip"));

        let dir_key = layout::job_dir_key(job_id);
        assert_eq!(dir_key, format!("jobs/{job_id}"));

        let batch_id = Uuid::now_v7();
        let batch_zip = layout::batch_artifact_key(batch_id, "zip");
        assert_eq!(batch_zip, format!("batches/{batch_id}/batch.zip"));

        let batch_dir = layout::batch_dir_key(batch_id);
        assert_eq!(batch_dir, format!("batches/{batch_id}"));

        let blob = layout::blob_key("abcdef0123456789");
        assert_eq!(blob, "blobs/ab/abcdef0123456789.bin");
    }
}
