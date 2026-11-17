//! # Pluggable Storage Backend Abstraction
//!
//! Provides the unified [`StorageBackend`] trait, dynamic dispatch wrapper [`DynStorage`],
//! factory instantiation via [`create_storage_backend`], and an in-memory implementation.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::io::Cursor;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use web_repo_to_ai_core::config::{StorageBackend as BackendType, StorageConfig};
use web_repo_to_ai_core::error::CoreError;

use crate::local_fs::{ArtifactFileMetadata, LocalFsStorage, StorageUsage};

/// Pinned asynchronous reader stream returned by storage backends.
pub type AsyncReadStream = Pin<Box<dyn tokio::io::AsyncRead + Send>>;

/// Dynamic, thread-safe trait object reference for storage backends.
pub type DynStorage = Arc<dyn StorageBackend>;

/// Unified asynchronous abstraction for artifact and blob storage drivers.
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Returns the concrete backend driver category.
    fn backend_type(&self) -> BackendType;

    /// Atomically stores data at the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] or [`CoreError::Security`] if write fails.
    async fn put(&self, key: &str, data: &[u8]) -> Result<u64, CoreError>;

    /// Retrieves the entire byte content stored at the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact does not exist.
    async fn get(&self, key: &str) -> Result<Vec<u8>, CoreError>;

    /// Opens an asynchronous streaming reader for the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact does not exist.
    async fn open_read(&self, key: &str) -> Result<AsyncReadStream, CoreError>;

    /// Checks if an artifact exists at the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if probe fails.
    async fn exists(&self, key: &str) -> Result<bool, CoreError>;

    /// Deletes the artifact stored at the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if deletion fails.
    async fn delete(&self, key: &str) -> Result<bool, CoreError>;

    /// Recursively deletes all artifacts matching the directory prefix.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if deletion fails.
    async fn delete_directory(&self, prefix: &str) -> Result<bool, CoreError>;

    /// Inspects metadata for the artifact at the specified key.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::NotFound`] if the artifact is missing.
    async fn metadata(&self, key: &str) -> Result<ArtifactFileMetadata, CoreError>;

    /// Recursively lists all artifact keys matching an optional directory prefix.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if directory traversal fails.
    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, CoreError>;

    /// Computes aggregate capacity and file count for stored artifacts.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError::Storage`] if capacity query fails.
    async fn storage_capacity(&self) -> Result<StorageUsage, CoreError>;
}

#[async_trait]
impl StorageBackend for LocalFsStorage {
    fn backend_type(&self) -> BackendType {
        BackendType::LocalFs
    }

    async fn put(&self, key: &str, data: &[u8]) -> Result<u64, CoreError> {
        self.put(key, data).await
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, CoreError> {
        self.get(key).await
    }

    async fn open_read(&self, key: &str) -> Result<AsyncReadStream, CoreError> {
        let file = self.open_read(key).await?;
        Ok(Box::pin(file))
    }

    async fn exists(&self, key: &str) -> Result<bool, CoreError> {
        self.exists(key).await
    }

    async fn delete(&self, key: &str) -> Result<bool, CoreError> {
        self.delete(key).await
    }

    async fn delete_directory(&self, prefix: &str) -> Result<bool, CoreError> {
        self.delete_directory(prefix).await
    }

    async fn metadata(&self, key: &str) -> Result<ArtifactFileMetadata, CoreError> {
        self.metadata(key).await
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, CoreError> {
        self.list(prefix).await
    }

    async fn storage_capacity(&self) -> Result<StorageUsage, CoreError> {
        self.storage_capacity().await
    }
}

/// Fully functional, thread-safe in-memory storage backend.
///
/// Stores artifacts in a synchronized hash map for testing, ephemeral processing,
/// and environments without physical disk access.
#[derive(Debug, Default)]
pub struct InMemoryStorage {
    entries: RwLock<HashMap<String, Vec<u8>>>,
    created_timestamps: RwLock<HashMap<String, chrono::DateTime<Utc>>>,
    max_storage_bytes: u64,
}

impl InMemoryStorage {
    /// Instantiates an empty `InMemoryStorage` with a 15 GB ceiling.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            created_timestamps: RwLock::new(HashMap::new()),
            max_storage_bytes: 15 * 1024 * 1024 * 1024,
        }
    }

    /// Instantiates an `InMemoryStorage` with a specific maximum byte limit.
    #[must_use]
    pub fn with_limit(max_storage_bytes: u64) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            created_timestamps: RwLock::new(HashMap::new()),
            max_storage_bytes,
        }
    }

    /// Validates storage key against path traversal and illegal characters.
    fn validate_key(key: &str) -> Result<String, CoreError> {
        let trimmed = key.trim().replace('\\', "/");
        if trimmed.is_empty() {
            return Err(CoreError::Validation {
                field: "key".to_string(),
                reason: "Storage key cannot be empty".to_string(),
            });
        }

        if trimmed.contains('\0') {
            return Err(CoreError::Security("Null byte in storage key".to_string()));
        }

        if trimmed.starts_with('/') {
            return Err(CoreError::Security(
                "Absolute path in storage key".to_string(),
            ));
        }

        for part in trimmed.split('/') {
            if part == ".." {
                return Err(CoreError::Security(
                    "Path traversal ('..') in storage key".to_string(),
                ));
            }
        }

        Ok(trimmed)
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    fn backend_type(&self) -> BackendType {
        BackendType::LocalFs
    }

    async fn put(&self, key: &str, data: &[u8]) -> Result<u64, CoreError> {
        let valid_key = Self::validate_key(key)?;
        let byte_len = data.len() as u64;

        if byte_len > self.max_storage_bytes {
            return Err(CoreError::PayloadTooLarge {
                actual_bytes: byte_len,
                max_bytes: self.max_storage_bytes,
            });
        }

        let mut entries = self
            .entries
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let mut timestamps = self
            .created_timestamps
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        entries.insert(valid_key.clone(), data.to_vec());
        timestamps.entry(valid_key).or_insert_with(Utc::now);

        Ok(byte_len)
    }

    async fn get(&self, key: &str) -> Result<Vec<u8>, CoreError> {
        let valid_key = Self::validate_key(key)?;
        let entries = self
            .entries
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        entries
            .get(&valid_key)
            .cloned()
            .ok_or_else(|| CoreError::NotFound {
                resource_type: "artifact",
                id: key.to_string(),
            })
    }

    async fn open_read(&self, key: &str) -> Result<AsyncReadStream, CoreError> {
        let bytes = self.get(key).await?;
        let cursor = Cursor::new(bytes);
        Ok(Box::pin(cursor))
    }

    async fn exists(&self, key: &str) -> Result<bool, CoreError> {
        let valid_key = match Self::validate_key(key) {
            Ok(k) => k,
            Err(_) => return Ok(false),
        };

        let entries = self
            .entries
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        Ok(entries.contains_key(&valid_key))
    }

    async fn delete(&self, key: &str) -> Result<bool, CoreError> {
        let valid_key = Self::validate_key(key)?;

        let mut entries = self
            .entries
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let mut timestamps = self
            .created_timestamps
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        timestamps.remove(&valid_key);
        Ok(entries.remove(&valid_key).is_some())
    }

    async fn delete_directory(&self, prefix: &str) -> Result<bool, CoreError> {
        let valid_prefix = Self::validate_key(prefix)?;
        let prefix_slash = if valid_prefix.ends_with('/') {
            valid_prefix
        } else {
            format!("{valid_prefix}/")
        };

        let mut entries = self
            .entries
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let mut timestamps = self
            .created_timestamps
            .write()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let matching_keys: Vec<String> = entries
            .keys()
            .filter(|k| k.starts_with(&prefix_slash))
            .cloned()
            .collect();

        if matching_keys.is_empty() {
            return Ok(false);
        }

        for k in &matching_keys {
            entries.remove(k);
            timestamps.remove(k);
        }

        Ok(true)
    }

    async fn metadata(&self, key: &str) -> Result<ArtifactFileMetadata, CoreError> {
        let valid_key = Self::validate_key(key)?;
        let entries = self
            .entries
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let timestamps = self
            .created_timestamps
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let data = entries.get(&valid_key).ok_or_else(|| CoreError::NotFound {
            resource_type: "artifact",
            id: key.to_string(),
        })?;

        let created_at = timestamps.get(&valid_key).copied();

        Ok(ArtifactFileMetadata {
            key: key.to_string(),
            byte_size: data.len() as u64,
            created_at,
            modified_at: created_at.unwrap_or_else(Utc::now),
            is_readonly: false,
        })
    }

    async fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, CoreError> {
        let entries = self
            .entries
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let mut keys: Vec<String> = match prefix {
            Some(p) => {
                let valid_prefix = Self::validate_key(p)?;
                let prefix_slash = if valid_prefix.ends_with('/') {
                    valid_prefix.clone()
                } else {
                    format!("{valid_prefix}/")
                };

                entries
                    .keys()
                    .filter(|k| k.starts_with(&prefix_slash) || *k == &valid_prefix)
                    .cloned()
                    .collect()
            }
            None => entries.keys().cloned().collect(),
        };

        keys.sort();
        Ok(keys)
    }

    async fn storage_capacity(&self) -> Result<StorageUsage, CoreError> {
        let entries = self
            .entries
            .read()
            .map_err(|_| CoreError::Storage("Poisoned lock in InMemoryStorage".to_string()))?;

        let total_artifact_bytes = entries.values().map(|v| v.len() as u64).sum();
        let artifact_count = entries.len();

        Ok(StorageUsage {
            total_artifact_bytes,
            artifact_count,
        })
    }
}

/// Creates a thread-safe, heap-allocated storage backend driver from application configuration.
///
/// # Errors
///
/// Returns [`CoreError::Config`] if the backend is unsupported or invalid.
/// Returns [`CoreError::Validation`] if configuration parameters fail validation.
pub fn create_storage_backend(config: &StorageConfig) -> Result<DynStorage, CoreError> {
    match config.backend {
        BackendType::LocalFs => {
            let storage = LocalFsStorage::with_config(config)?;
            Ok(Arc::new(storage))
        }
        BackendType::S3 => Err(CoreError::Config(
            "S3 storage backend is not yet supported in this build; please configure 'localfs'"
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};
    use uuid::Uuid;

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir()
                .join(format!("web-repo-to-ai-backend-test-{}", Uuid::now_v7()));
            std::fs::create_dir_all(&path).expect("Create test dir");
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
    async fn test_local_fs_through_dyn_storage() {
        let test_dir = TestDir::new();
        let config = StorageConfig {
            backend: BackendType::LocalFs,
            base_dir: test_dir.path().to_path_buf(),
            max_storage_bytes: 10 * 1024 * 1024,
            artifact_retention_days: 7,
        };

        let storage: DynStorage = create_storage_backend(&config).expect("Factory failed");
        assert_eq!(storage.backend_type(), BackendType::LocalFs);

        let key = "jobs/test-dyn/artifact.json";
        let payload = br#"{"key": "value"}"#;

        assert!(!storage.exists(key).await.expect("Exists failed"));

        let written = storage.put(key, payload).await.expect("Put failed");
        assert_eq!(written, payload.len() as u64);

        assert!(storage.exists(key).await.expect("Exists failed"));

        let fetched = storage.get(key).await.expect("Get failed");
        assert_eq!(fetched, payload);

        let meta = storage.metadata(key).await.expect("Metadata failed");
        assert_eq!(meta.byte_size, payload.len() as u64);

        let keys = storage
            .list(Some("jobs/test-dyn"))
            .await
            .expect("List failed");
        assert_eq!(keys, vec![key]);

        let deleted = storage.delete(key).await.expect("Delete failed");
        assert!(deleted);
        assert!(!storage.exists(key).await.expect("Exists failed"));
    }

    #[tokio::test]
    async fn test_in_memory_storage_crud_lifecycle() {
        let storage = InMemoryStorage::new();
        assert_eq!(storage.backend_type(), BackendType::LocalFs);

        let key = "docs/readme.md";
        let content = b"# In-Memory README";

        assert!(!storage.exists(key).await.expect("Exists failed"));

        let bytes = storage.put(key, content).await.expect("Put failed");
        assert_eq!(bytes, content.len() as u64);

        assert!(storage.exists(key).await.expect("Exists failed"));

        let read = storage.get(key).await.expect("Get failed");
        assert_eq!(read, content);

        let meta = storage.metadata(key).await.expect("Metadata failed");
        assert_eq!(meta.byte_size, content.len() as u64);

        let usage = storage.storage_capacity().await.expect("Capacity failed");
        assert_eq!(usage.artifact_count, 1);
        assert_eq!(usage.total_artifact_bytes, content.len() as u64);

        let deleted = storage.delete(key).await.expect("Delete failed");
        assert!(deleted);
        assert!(!storage.exists(key).await.expect("Exists failed"));
    }

    #[tokio::test]
    async fn test_in_memory_storage_directory_deletion() {
        let storage = InMemoryStorage::new();

        storage.put("job-1/file1.txt", b"1").await.unwrap();
        storage.put("job-1/file2.txt", b"2").await.unwrap();
        storage.put("job-2/file3.txt", b"3").await.unwrap();

        assert_eq!(storage.list(None).await.unwrap().len(), 3);

        let deleted = storage.delete_directory("job-1").await.unwrap();
        assert!(deleted);

        let remaining = storage.list(None).await.unwrap();
        assert_eq!(remaining, vec!["job-2/file3.txt"]);
    }

    #[tokio::test]
    async fn test_in_memory_storage_path_traversal() {
        let storage = InMemoryStorage::new();
        assert!(storage.put("../secret.txt", b"bad").await.is_err());
        assert!(storage.put("/etc/shadow", b"bad").await.is_err());
        assert!(storage
            .put("folder/../../escape.txt", b"bad")
            .await
            .is_err());
        assert!(storage.put("folder/null\0byte.txt", b"bad").await.is_err());
    }

    #[test]
    fn test_create_storage_backend_s3_unsupported() {
        let config = StorageConfig {
            backend: BackendType::S3,
            base_dir: PathBuf::from("./data"),
            max_storage_bytes: 1024,
            artifact_retention_days: 1,
        };

        let err = create_storage_backend(&config).unwrap_err();
        match err {
            CoreError::Config(msg) => {
                assert!(msg.contains("S3 storage backend is not yet supported"));
            }
            other => panic!("Expected Config error, got: {other:?}"),
        }
    }
}
