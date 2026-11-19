//! Integration test suite for Git tree diffing, commit and blob incremental caching.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use web_repo_to_ai_github_extractor::{
    compute_git_blob_sha, ClassifiedFile, FileCategory, GitBlob, GitCacheStore, GitFileMode,
    GitHubTarget, GitTreeEntry, GitTreeEntryType, GitTreeResponse, InMemoryGitCacheStore,
    IncrementalBlobExtractor, SourceExtractionConfig, TreeIncrementalDiff,
};

#[test]
fn test_tree_incremental_diff_classification() {
    let sha_unchanged = "1111111111111111111111111111111111111111";
    let sha_prev_mod = "2222222222222222222222222222222222222222";
    let sha_curr_mod = "3333333333333333333333333333333333333333";
    let sha_deleted = "4444444444444444444444444444444444444444";
    let sha_added = "5555555555555555555555555555555555555555";

    let previous = vec![
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_unchanged,
            Some(100),
        ),
        GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_prev_mod,
            Some(200),
        ),
        GitTreeEntry::new(
            "README.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_deleted,
            Some(50),
        ),
        // Subtree directory entry should be ignored in blob comparison
        GitTreeEntry::new(
            "src",
            GitFileMode::Subdirectory,
            GitTreeEntryType::Tree,
            "tree111111111111111111111111111111111111",
            None,
        ),
    ];

    let current = vec![
        GitTreeEntry::new(
            "src/lib.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_unchanged,
            Some(100),
        ),
        GitTreeEntry::new(
            "src/main.rs",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_curr_mod,
            Some(250),
        ),
        GitTreeEntry::new(
            "docs/API.md",
            GitFileMode::RegularFile,
            GitTreeEntryType::Blob,
            sha_added,
            Some(80),
        ),
    ];

    let diff = TreeIncrementalDiff::compute(&previous, &current)
        .with_commits("prev_commit_sha", "curr_commit_sha");

    assert_eq!(diff.previous_commit_sha.as_deref(), Some("prev_commit_sha"));
    assert_eq!(diff.current_commit_sha.as_deref(), Some("curr_commit_sha"));

    assert_eq!(diff.unchanged_count(), 1);
    assert_eq!(diff.modified_count(), 1);
    assert_eq!(diff.added_count(), 1);
    assert_eq!(diff.deleted_count(), 1);
    assert_eq!(diff.total_current_blobs(), 3);

    assert_eq!(diff.unchanged_blobs[0].path, "src/lib.rs");
    assert_eq!(diff.modified_blobs[0].path, "src/main.rs");
    assert_eq!(diff.added_blobs[0].path, "docs/API.md");
    assert_eq!(diff.deleted_blobs[0].path, "README.md");

    // Reusable SHAs
    let reusable = diff.reusable_shas();
    assert_eq!(reusable.len(), 1);
    assert!(reusable.contains(sha_unchanged));

    // Ratio: 1 / 3
    let ratio = diff.unchanged_ratio();
    assert!((ratio - (1.0 / 3.0)).abs() < 1e-6);
}

#[tokio::test]
async fn test_in_memory_git_cache_store_operations() {
    let store = InMemoryGitCacheStore::new();
    let body = b"fn main() { println!(\"Hello World\"); }";
    let sha = compute_git_blob_sha(body);
    let blob = GitBlob::new(sha.clone(), body.to_vec(), "raw");

    // Cache miss
    assert!(store.get_cached_blob(&sha).await.unwrap().is_none());

    // Store and hit
    store.store_cached_blob(&blob).await.unwrap();
    let retrieved = store
        .get_cached_blob(&sha)
        .await
        .unwrap()
        .expect("blob must exist");
    assert_eq!(retrieved.sha, sha);
    assert_eq!(retrieved.raw_bytes, body.to_vec());

    // Case insensitive lookup
    let upper_retrieved = store
        .get_cached_blob(&sha.to_uppercase())
        .await
        .unwrap()
        .expect("upper sha must resolve");
    assert_eq!(upper_retrieved.sha, sha);

    // Batch query
    let queries = vec![
        sha.clone(),
        "nonexistent0000000000000000000000000000".to_string(),
    ];
    let hits = store.check_cached_blobs(&queries).await.unwrap();
    assert_eq!(hits.len(), 1);
    assert!(hits.contains(&sha));

    // Tree caching
    let tree_response = GitTreeResponse {
        sha: "root_tree_sha".to_string(),
        url: None,
        tree: vec![],
        truncated: false,
    };
    assert!(store
        .get_cached_tree("octocat", "Hello-World", "c0ffee")
        .await
        .unwrap()
        .is_none());

    store
        .store_cached_tree("octocat", "Hello-World", "c0ffee", &tree_response)
        .await
        .unwrap();

    let cached_tree = store
        .get_cached_tree("OCTOCAT", "hello-world", "C0FFEE")
        .await
        .unwrap()
        .expect("tree must be cached");
    assert_eq!(cached_tree.sha, "root_tree_sha");
    assert_eq!(store.tree_count().await, 1);
    assert_eq!(store.blob_count().await, 1);
}

#[tokio::test]
async fn test_incremental_blob_extractor_cold_and_warm_cache() {
    let target = GitHubTarget::for_repo("octocat", "Hello-World");
    let cache = InMemoryGitCacheStore::new();
    let config = SourceExtractionConfig::new();

    let content1 = b"pub fn add(a: i32, b: i32) -> i32 { a + b }";
    let content2 = b"pub fn sub(a: i32, b: i32) -> i32 { a - b }";

    let sha1 = compute_git_blob_sha(content1);
    let sha2 = compute_git_blob_sha(content2);

    let files = vec![
        ClassifiedFile {
            path: "src/add.rs".to_string(),
            sha: sha1.clone(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            size: Some(content1.len() as u64),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "src/sub.rs".to_string(),
            sha: sha2.clone(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            size: Some(content2.len() as u64),
            language: Some("rust".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
    ];

    let network_fetch_count = Arc::new(AtomicUsize::new(0));

    // 1. Cold extraction: cache is empty
    let (report1, stats1) = {
        let fetch_counter = Arc::clone(&network_fetch_count);
        let c1 = content1.to_vec();
        let c2 = content2.to_vec();
        let s1 = sha1.clone();
        let s2 = sha2.clone();

        IncrementalBlobExtractor::extract_with_fetcher(
            &target,
            &files,
            &config,
            &cache,
            move |requested_sha| {
                fetch_counter.fetch_add(1, Ordering::SeqCst);
                let bytes = if requested_sha == s1 {
                    c1.clone()
                } else if requested_sha == s2 {
                    c2.clone()
                } else {
                    panic!("unexpected sha: {requested_sha}");
                };
                let blob = GitBlob::new(requested_sha, bytes, "raw");
                async move { Ok(blob) }
            },
        )
        .await
    };

    assert_eq!(stats1.total_files, 2);
    assert_eq!(stats1.cached_files, 0);
    assert_eq!(stats1.fetched_files, 2);
    assert_eq!(stats1.failed_files, 0);
    assert_eq!(network_fetch_count.load(Ordering::SeqCst), 2);
    assert_eq!(report1.extracted.len(), 2);
    assert_eq!(stats1.cache_hit_ratio(), 0.0);

    // 2. Warm extraction: all files are now cached
    let (report2, stats2) = {
        let fetch_counter = Arc::clone(&network_fetch_count);
        IncrementalBlobExtractor::extract_with_fetcher(
            &target,
            &files,
            &config,
            &cache,
            move |_requested_sha| {
                fetch_counter.fetch_add(1, Ordering::SeqCst);
                async move { panic!("Network should NEVER be called on warm cache hit!") }
            },
        )
        .await
    };

    assert_eq!(stats2.total_files, 2);
    assert_eq!(stats2.cached_files, 2);
    assert_eq!(stats2.fetched_files, 0);
    assert_eq!(stats2.failed_files, 0);
    // Network count must remain 2 (zero additional calls)
    assert_eq!(network_fetch_count.load(Ordering::SeqCst), 2);
    assert_eq!(report2.extracted.len(), 2);
    assert_eq!(stats2.cache_hit_ratio(), 1.0);
    assert_eq!(stats2.byte_hit_ratio(), 1.0);
}

#[tokio::test]
async fn test_incremental_blob_extractor_mixed_hits_and_binary() {
    let target = GitHubTarget::for_repo("octocat", "Hello-World");
    let cache = InMemoryGitCacheStore::new();
    let config = SourceExtractionConfig::new();

    let cached_content = b"console.log('cached');";
    let cached_sha = compute_git_blob_sha(cached_content);
    cache
        .store_cached_blob(&GitBlob::new(
            cached_sha.clone(),
            cached_content.to_vec(),
            "raw",
        ))
        .await
        .unwrap();

    let new_content = b"console.log('new');";
    let new_sha = compute_git_blob_sha(new_content);

    let files = vec![
        ClassifiedFile {
            path: "cached.js".to_string(),
            sha: cached_sha.clone(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            size: Some(cached_content.len() as u64),
            language: Some("javascript".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "new.js".to_string(),
            sha: new_sha.clone(),
            category: FileCategory::SourceCode,
            mode: GitFileMode::RegularFile,
            size: Some(new_content.len() as u64),
            language: Some("javascript".to_string()),
            is_eligible: true,
            exclusion_reason: None,
        },
        ClassifiedFile {
            path: "binary.bin".to_string(),
            sha: "bin0000000000000000000000000000000000000".to_string(),
            category: FileCategory::Binary,
            mode: GitFileMode::RegularFile,
            size: Some(1024),
            language: None,
            is_eligible: false,
            exclusion_reason: Some("Binary file detected".to_string()),
        },
    ];

    let network_fetches = Arc::new(AtomicUsize::new(0));
    let n_counter = Arc::clone(&network_fetches);
    let n_bytes = new_content.to_vec();

    let (report, stats) = IncrementalBlobExtractor::extract_with_fetcher(
        &target,
        &files,
        &config,
        &cache,
        move |sha| {
            n_counter.fetch_add(1, Ordering::SeqCst);
            let b = n_bytes.clone();
            async move { Ok(GitBlob::new(sha, b, "raw")) }
        },
    )
    .await;

    assert_eq!(stats.total_files, 2); // 2 eligible files
    assert_eq!(stats.cached_files, 1);
    assert_eq!(stats.fetched_files, 1);
    assert_eq!(stats.unsupported_files, 1); // 1 binary file
    assert_eq!(network_fetches.load(Ordering::SeqCst), 1);
    assert_eq!(report.extracted.len(), 2);
    assert_eq!(report.unsupported.len(), 1);
    assert_eq!(stats.cache_hit_ratio(), 0.5);
}
