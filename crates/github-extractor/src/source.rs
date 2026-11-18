//! # GitHub Source Code Acquisition
//!
//! Provides asynchronous, rate-limit-conscious source file extraction via the GitHub Git Blobs API
//! and raw endpoints. Enforces Git object SHA-1 verification, BLAKE3 and SHA-256 cryptographic
//! digests, binary inspection, bounded concurrent batch extraction, and conversion into Canonical
//! Context Document Model ([`CdmSection`]) representations.

use std::borrow::Cow;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sha1::Digest;
use web_repo_to_ai_cdm::{
    CdmSection, CdmSectionBuilder, CompletenessReport, SectionKind, SectionTokenMetrics,
};

use crate::classification::{detect_language, ClassifiedFile, DEFAULT_MAX_FILE_SIZE_BYTES};
use crate::error::GitHubExtractorError;

/// Default concurrent request concurrency for batch blob acquisition.
pub const DEFAULT_EXTRACTION_CONCURRENCY: usize = 10;

/// Number of initial bytes inspected to determine if content contains binary/null bytes.
pub const BINARY_SNIFF_WINDOW_BYTES: usize = 8000;

/// Pure Rust RFC 4648 base64 decoder with tolerance for whitespace and newlines.
///
/// Designed to parse standard base64 payloads returned by GitHub REST API (`GET /git/blobs/{sha}`).
pub fn decode_base64(input: &str) -> Result<Vec<u8>, String> {
    let mut clean = Vec::with_capacity(input.len());
    for b in input.bytes() {
        if !b.is_ascii_whitespace() {
            clean.push(b);
        }
    }

    if clean.is_empty() {
        return Ok(Vec::new());
    }

    if clean.len() % 4 != 0 {
        return Err(format!(
            "Base64 input length ({}) is not a multiple of 4",
            clean.len()
        ));
    }

    let mut output = Vec::with_capacity((clean.len() / 4) * 3);

    let decode_val = |b: u8| -> Result<u32, String> {
        match b {
            b'A'..=b'Z' => Ok((b - b'A') as u32),
            b'a'..=b'z' => Ok((b - b'a' + 26) as u32),
            b'0'..=b'9' => Ok((b - b'0' + 52) as u32),
            b'+' => Ok(62),
            b'/' => Ok(63),
            b'=' => Ok(0), // Handled at chunk boundary
            _ => Err(format!(
                "Invalid character '{}' in base64 payload",
                b as char
            )),
        }
    };

    let chunks = clean.chunks_exact(4);
    for chunk in chunks {
        let b0 = decode_val(chunk[0])?;
        let b1 = decode_val(chunk[1])?;
        let b2 = decode_val(chunk[2])?;
        let b3 = decode_val(chunk[3])?;

        let triple = (b0 << 18) | (b1 << 12) | (b2 << 6) | b3;

        output.push(((triple >> 16) & 0xFF) as u8);
        if chunk[2] != b'=' {
            output.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            output.push((triple & 0xFF) as u8);
        }
    }

    Ok(output)
}

/// Computes the standard Git object SHA-1 hex digest for a blob: `sha1("blob <size>\0<content>")`.
#[must_use]
pub fn compute_git_blob_sha(bytes: &[u8]) -> String {
    let mut hasher = sha1::Sha1::new();
    let header = format!("blob {}\0", bytes.len());
    hasher.update(header.as_bytes());
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Computes the cryptographic BLAKE3 256-bit hex digest for content integrity.
#[must_use]
pub fn compute_blake3_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

/// Computes the standard SHA-256 hex digest for content integrity.
#[must_use]
pub fn compute_sha256_hash(bytes: &[u8]) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Inspects content bytes for null bytes (`\0`) within the initial sniff window
/// to identify binary content.
#[must_use]
pub fn is_binary_content(bytes: &[u8]) -> bool {
    let window = if bytes.len() > BINARY_SNIFF_WINDOW_BYTES {
        &bytes[..BINARY_SNIFF_WINDOW_BYTES]
    } else {
        bytes
    };

    window.contains(&0)
}

/// Raw Git blob retrieved from the GitHub Git Blobs API or raw endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitBlob {
    /// Expected or verified 40-character Git SHA-1 object hash.
    pub sha: String,
    /// Byte length of the raw content.
    pub size: usize,
    /// Transfer encoding format (e.g. `"base64"`, `"raw"`).
    pub encoding: String,
    /// Raw unencoded bytes of the blob.
    pub raw_bytes: Vec<u8>,
}

impl GitBlob {
    /// Creates a new `GitBlob` from raw bytes.
    #[must_use]
    pub fn new(sha: impl Into<String>, raw_bytes: Vec<u8>, encoding: impl Into<String>) -> Self {
        let size = raw_bytes.len();
        Self {
            sha: sha.into(),
            size,
            encoding: encoding.into(),
            raw_bytes,
        }
    }

    /// Attempts to parse raw bytes as a valid UTF-8 string slice.
    pub fn content_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.raw_bytes)
    }

    /// Returns a lossy UTF-8 representation of the blob body.
    #[must_use]
    pub fn content_lossy(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.raw_bytes)
    }

    /// Checks whether the blob contains binary or null bytes.
    #[must_use]
    pub fn is_binary(&self) -> bool {
        is_binary_content(&self.raw_bytes)
    }

    /// Verifies whether the calculated Git object SHA matches the reported `sha`.
    #[must_use]
    pub fn verify_git_sha(&self) -> bool {
        let calculated = compute_git_blob_sha(&self.raw_bytes);
        calculated.eq_ignore_ascii_case(&self.sha)
    }
}

/// Configuration settings governing source code file acquisition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceExtractionConfig {
    /// Maximum concurrent HTTP requests during batch acquisition.
    pub concurrency: usize,
    /// Maximum allowed file size in bytes before skipping as oversized.
    pub max_file_size_bytes: u64,
    /// Whether to verify Git object SHA-1 hashes against downloaded blob bytes.
    pub verify_git_sha: bool,
    /// Individual request timeout.
    pub timeout: Duration,
}

impl Default for SourceExtractionConfig {
    fn default() -> Self {
        Self {
            concurrency: DEFAULT_EXTRACTION_CONCURRENCY,
            max_file_size_bytes: DEFAULT_MAX_FILE_SIZE_BYTES,
            verify_git_sha: true,
            timeout: Duration::from_secs(30),
        }
    }
}

impl SourceExtractionConfig {
    /// Creates a new `SourceExtractionConfig` with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the maximum concurrent worker requests.
    #[must_use]
    pub const fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = if concurrency == 0 { 1 } else { concurrency };
        self
    }

    /// Configures the upper file size threshold in bytes.
    #[must_use]
    pub const fn with_max_file_size(mut self, max_bytes: u64) -> Self {
        self.max_file_size_bytes = max_bytes;
        self
    }

    /// Configures whether Git SHA-1 hashes are verified.
    #[must_use]
    pub const fn with_verify_git_sha(mut self, verify: bool) -> Self {
        self.verify_git_sha = verify;
        self
    }

    /// Configures the individual request timeout.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }
}

/// An extracted, verified, and normalized source code file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedSourceFile {
    /// Normalized relative repository path (e.g. `src/main.rs`).
    pub path: String,
    /// 40-character Git object SHA-1.
    pub git_sha: String,
    /// Normalized UTF-8 text content.
    pub content: String,
    /// Size in bytes of the content string.
    pub byte_size: usize,
    /// Line count of the content.
    pub line_count: usize,
    /// Detected programming language (e.g. `"rust"`, `"typescript"`).
    pub language: Option<String>,
    /// Cryptographic BLAKE3 digest of the content bytes.
    pub blake3_hash: String,
    /// Cryptographic SHA-256 digest of the content bytes.
    pub sha256_hash: String,
    /// Whether content was truncated due to budget constraints.
    pub is_truncated: bool,
}

impl ExtractedSourceFile {
    /// Constructs a verified `ExtractedSourceFile` from raw blob bytes and relative path.
    pub fn from_blob(
        path: impl Into<String>,
        blob: &GitBlob,
        config: &SourceExtractionConfig,
    ) -> Result<Self, GitHubExtractorError> {
        let path = path.into();

        if config.verify_git_sha && !blob.verify_git_sha() {
            let actual = compute_git_blob_sha(&blob.raw_bytes);
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!(
                    "Git blob SHA-1 mismatch for '{path}': expected '{}', got '{actual}'",
                    blob.sha
                ),
            });
        }

        if blob.is_binary() {
            return Err(GitHubExtractorError::BlobError {
                sha: blob.sha.clone(),
                message: format!("Binary or null bytes detected in source file '{path}'"),
            });
        }

        let content = blob.content_lossy().into_owned();
        let byte_size = content.len();
        let line_count = if content.is_empty() {
            0
        } else {
            content.lines().count().max(1)
        };

        let file_name = path.rsplit('/').next().unwrap_or(&path);
        let extension = file_name.rsplit_once('.').map(|(_, ext)| ext);
        let language = detect_language(file_name, extension);

        let blake3_hash = compute_blake3_hash(content.as_bytes());
        let sha256_hash = compute_sha256_hash(content.as_bytes());

        Ok(Self {
            path,
            git_sha: blob.sha.clone(),
            content,
            byte_size,
            line_count,
            language,
            blake3_hash,
            sha256_hash,
            is_truncated: false,
        })
    }

    /// Converts this extracted source file into a Canonical Context Document Model [`CdmSection`].
    #[must_use]
    pub fn to_cdm_section(
        &self,
        section_id: impl Into<String>,
        source_url: impl Into<String>,
    ) -> CdmSection {
        let lines = self.line_count as u32;
        let mut builder =
            CdmSectionBuilder::new(section_id, source_url, SectionKind::CodeFile, &self.content)
                .title(&self.path)
                .original_path(&self.path)
                .line_range(1, lines.max(1))
                .byte_range(0, self.byte_size as u64)
                .content_hash(&self.blake3_hash)
                .token_metrics(SectionTokenMetrics::compute_baseline(&self.content));

        if let Some(ref lang) = self.language {
            builder = builder.language(lang);
        }

        let mut section = builder.build();
        section
            .attributes
            .insert("git_blob_sha".to_string(), self.git_sha.clone());
        section
            .attributes
            .insert("sha256".to_string(), self.sha256_hash.clone());

        section
    }
}

/// Detailed failure record when an individual source file fails extraction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailedSourceExtraction {
    /// Relative repository path of the failed file.
    pub path: String,
    /// Git blob SHA.
    pub git_sha: String,
    /// Sanitized error description.
    pub error: String,
}

/// Consolidated audit and diagnostic report from batch source code acquisition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceExtractionReport {
    /// Successfully acquired and verified source code files.
    pub extracted: Vec<ExtractedSourceFile>,
    /// Files that encountered errors during fetch or validation.
    pub failed: Vec<FailedSourceExtraction>,
    /// Files deemed unsupported (e.g. discovered as binary upon content inspection).
    pub unsupported: Vec<ClassifiedFile>,
    /// Total bytes acquired across all extracted source files.
    pub total_bytes: u64,
    /// Total lines spanned across all extracted source files.
    pub total_lines: usize,
    /// Total duration elapsed during the acquisition batch.
    pub duration: Duration,
}

impl SourceExtractionReport {
    /// Creates a new empty `SourceExtractionReport`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            extracted: Vec::new(),
            failed: Vec::new(),
            unsupported: Vec::new(),
            total_bytes: 0,
            total_lines: 0,
            duration: Duration::ZERO,
        }
    }

    /// Appends a successfully extracted source file and updates summary metrics.
    pub fn add_extracted(&mut self, file: ExtractedSourceFile) {
        self.total_bytes += file.byte_size as u64;
        self.total_lines += file.line_count;
        self.extracted.push(file);
    }

    /// Records a failed extraction.
    pub fn add_failed(
        &mut self,
        path: impl Into<String>,
        git_sha: impl Into<String>,
        error: impl Into<String>,
    ) {
        self.failed.push(FailedSourceExtraction {
            path: path.into(),
            git_sha: git_sha.into(),
            error: error.into(),
        });
    }

    /// Records an unsupported file.
    pub fn add_unsupported(&mut self, file: ClassifiedFile) {
        self.unsupported.push(file);
    }

    /// Evaluates whether all eligible attempted files were successfully extracted without failure.
    #[must_use]
    pub fn is_all_successful(&self) -> bool {
        self.failed.is_empty()
    }

    /// Translates this source extraction report into a CDM [`CompletenessReport`].
    ///
    /// Validates the canonical formula invariant:
    /// `discovered_count == extracted_count + excluded_count + failed_count + unsupported_count`
    #[must_use]
    pub fn to_completeness_report(
        &self,
        total_discovered: u64,
        total_excluded: u64,
    ) -> CompletenessReport {
        let extracted = self.extracted.len() as u64;
        let failed = self.failed.len() as u64;
        let unsupported = self.unsupported.len() as u64;
        let eligible = extracted + failed;

        CompletenessReport::new(
            total_discovered,
            eligible,
            extracted,
            total_excluded,
            failed,
            unsupported,
        )
    }

    /// Converts all extracted source files into a sequence of CDM [`CdmSection`]s.
    #[must_use]
    pub fn to_cdm_sections(&self, base_url: &str) -> Vec<CdmSection> {
        let clean_base = base_url.trim_end_matches('/');
        self.extracted
            .iter()
            .enumerate()
            .map(|(idx, file)| {
                let section_id = format!("sec-source-{:04}", idx + 1);
                let source_url = format!("{clean_base}/{}", file.path);
                file.to_cdm_section(section_id, source_url)
            })
            .collect()
    }
}

impl Default for SourceExtractionReport {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::GitFileMode;
    use web_repo_to_ai_cdm::LineRange;

    #[test]
    fn test_base64_decode_valid() {
        let raw = "Hello World! Rust Git Extraction.";
        let encoded = "SGVsbG8gV29ybGQhIFJ1c3QgR2l0IEV4dHJhY3Rpb24u";
        let decoded = decode_base64(encoded).expect("Valid base64 must decode");
        assert_eq!(String::from_utf8(decoded).unwrap(), raw);

        // With newlines and spaces
        let encoded_with_ws = "SGVsbG8g\nV29ybGQh\r\n IFJ1c3Qg\tR2l0IEV4dHJhY3Rpb24u\n";
        let decoded_ws =
            decode_base64(encoded_with_ws).expect("Base64 with whitespace must decode");
        assert_eq!(String::from_utf8(decoded_ws).unwrap(), raw);
    }

    #[test]
    fn test_base64_decode_padding_variants() {
        assert_eq!(decode_base64("").unwrap(), Vec::<u8>::new());
        assert_eq!(decode_base64("YQ==").unwrap(), b"a");
        assert_eq!(decode_base64("YWI=").unwrap(), b"ab");
        assert_eq!(decode_base64("YWJj").unwrap(), b"abc");
    }

    #[test]
    fn test_git_blob_sha_calculation() {
        // Echo "hello world" into `git hash-object --stdin` yields 3b18e512dba79e4c8300dd08aeb37f8e728b8dad
        let content = b"hello world\n";
        let sha = compute_git_blob_sha(content);
        assert_eq!(sha, "3b18e512dba79e4c8300dd08aeb37f8e728b8dad");
    }

    #[test]
    fn test_cryptographic_hashes_and_binary_detection() {
        let content = b"fn main() {\n    println!(\"Hello\");\n}\n";
        let blake3 = compute_blake3_hash(content);
        assert_eq!(blake3.len(), 64);

        let sha256 = compute_sha256_hash(content);
        assert_eq!(sha256.len(), 64);

        assert!(!is_binary_content(content));

        let mut binary_data = content.to_vec();
        binary_data.insert(10, 0); // Inject null byte
        assert!(is_binary_content(&binary_data));
    }

    #[test]
    fn test_extracted_source_file_to_cdm_section() {
        let code = "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n";
        let blob_bytes = code.as_bytes().to_vec();
        let expected_sha = compute_git_blob_sha(&blob_bytes);

        let blob = GitBlob::new(expected_sha, blob_bytes, "raw");
        let config = SourceExtractionConfig::default();

        let extracted = ExtractedSourceFile::from_blob("src/math.rs", &blob, &config)
            .expect("Source file should extract cleanly");

        assert_eq!(extracted.path, "src/math.rs");
        assert_eq!(extracted.language.as_deref(), Some("rust"));
        assert_eq!(extracted.line_count, 3);
        assert_eq!(extracted.byte_size, code.len());

        let section =
            extracted.to_cdm_section("sec-1", "https://github.com/org/repo/blob/main/src/math.rs");
        assert_eq!(section.section_id, "sec-1");
        assert_eq!(section.kind, SectionKind::CodeFile);
        assert_eq!(section.original_path.as_deref(), Some("src/math.rs"));
        assert_eq!(section.language.as_deref(), Some("rust"));
        assert_eq!(section.content_hash, extracted.blake3_hash);
        assert_eq!(section.line_range, Some(LineRange::new(1, 3)));
        assert_eq!(
            section.attributes.get("git_blob_sha"),
            Some(&extracted.git_sha)
        );
    }

    #[test]
    fn test_source_extraction_report_completeness_mapping() {
        let mut report = SourceExtractionReport::new();

        let blob = GitBlob::new("sha111", b"let x = 1;\n".to_vec(), "raw");
        let config = SourceExtractionConfig::default().with_verify_git_sha(false);
        let extracted = ExtractedSourceFile::from_blob("src/a.rs", &blob, &config).unwrap();
        report.add_extracted(extracted);

        report.add_failed("src/corrupt.rs", "sha222", "Network timeout");

        let unsupported_file = ClassifiedFile {
            path: "data/asset.bin".to_string(),
            category: crate::classification::FileCategory::Binary,
            mode: GitFileMode::RegularFile,
            sha: "sha333".to_string(),
            size: Some(1024),
            language: None,
            is_eligible: false,
            exclusion_reason: Some("Binary format".to_string()),
        };
        report.add_unsupported(unsupported_file);

        // Discovered = 5 total: 1 extracted + 1 failed + 1 unsupported + 2 excluded
        let cdm = report.to_completeness_report(5, 2);
        assert!(cdm.formula_valid());
        assert_eq!(cdm.discovered_count, 5);
        assert_eq!(cdm.extracted_count, 1);
        assert_eq!(cdm.failed_count, 1);
        assert_eq!(cdm.unsupported_count, 1);
        assert_eq!(cdm.excluded_count, 2);
    }
}
