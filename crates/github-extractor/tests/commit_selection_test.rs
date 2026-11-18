//! Integration tests for GitHub commit selection and immutable revision pinning (`crates/github-extractor`).

use std::time::Duration;

use axum::extract::Path;
use axum::http::{HeaderMap as AxumHeaderMap, StatusCode};
use axum::response::{IntoResponse, Json};
use axum::routing::get;
use axum::Router;
use chrono::{TimeZone, Utc};
use serde_json::json;
use tokio::net::TcpListener;

use web_repo_to_ai_github_extractor::{
    CommitAuthor, CommitSignatureVerification, GitHubApiClient, GitHubCommitMetadata,
    GitHubExtractorError, GitHubUrlParser, PinnedRevision,
};

#[test]
fn test_commit_author_and_verification_data_structures() {
    let now = Utc::now();
    let author = CommitAuthor::new("Octocat", "octocat@github.com", now);
    assert_eq!(author.name, "Octocat");
    assert_eq!(author.email, "octocat@github.com");
    assert_eq!(author.date, now);

    // Serialization & Deserialization
    let json_str = serde_json::to_string(&author).expect("must serialize");
    let deserialized: CommitAuthor = serde_json::from_str(&json_str).expect("must deserialize");
    assert_eq!(author, deserialized);

    let verification = CommitSignatureVerification {
        verified: true,
        reason: "valid".to_string(),
        signature: Some("-----BEGIN PGP SIGNATURE-----\n...".to_string()),
        payload: Some("tree abc\n...".to_string()),
    };
    assert!(verification.verified);
    assert_eq!(verification.reason, "valid");

    let ver_json = serde_json::to_string(&verification).expect("must serialize");
    let ver_deserialized: CommitSignatureVerification =
        serde_json::from_str(&ver_json).expect("must deserialize");
    assert_eq!(verification, ver_deserialized);
}

#[test]
fn test_commit_metadata_helpers() {
    let t1 = Utc.timestamp_opt(1700000000, 0).unwrap();
    let t2 = Utc.timestamp_opt(1700001000, 0).unwrap();

    let meta = GitHubCommitMetadata {
        sha: "a1b2c3d4e5f60123456789abcdef0123456789ab".to_string(),
        node_id: Some("C_kwDOA123".to_string()),
        message:
            "feat(core): implement zero-drift commit resolution\n\nDetailed body paragraph here."
                .to_string(),
        author: Some(CommitAuthor::new("Author Name", "author@example.com", t1)),
        committer: Some(CommitAuthor::new(
            "Committer Name",
            "committer@example.com",
            t2,
        )),
        tree_sha: "tree111111111111111111111111111111111111".to_string(),
        parents: vec!["parent0000000000000000000000000000000000".to_string()],
        comment_count: 3,
        verification: Some(CommitSignatureVerification {
            verified: true,
            reason: "valid".to_string(),
            signature: None,
            payload: None,
        }),
    };

    // short_sha
    assert_eq!(meta.short_sha(), "a1b2c3d");

    // summary
    assert_eq!(
        meta.summary(),
        "feat(core): implement zero-drift commit resolution"
    );

    // commit_date prioritizes committer date
    assert_eq!(meta.commit_date(), Some(t2));

    // Fallback to author date when committer is None
    let meta_no_committer = GitHubCommitMetadata {
        committer: None,
        ..meta.clone()
    };
    assert_eq!(meta_no_committer.commit_date(), Some(t1));

    // None when both author and committer are None
    let meta_no_author_or_committer = GitHubCommitMetadata {
        author: None,
        committer: None,
        ..meta.clone()
    };
    assert_eq!(meta_no_author_or_committer.commit_date(), None);

    // Short SHA under 7 chars
    let short_meta = GitHubCommitMetadata {
        sha: "abc".to_string(),
        ..meta
    };
    assert_eq!(short_meta.short_sha(), "abc");
}

#[test]
fn test_pinned_revision_properties_and_cdm_mapping() {
    let target = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates/core",
    )
    .expect("must parse");

    let t = Utc.timestamp_opt(1700000000, 0).unwrap();
    let meta = GitHubCommitMetadata {
        sha: "9999999999999999999999999999999999999999".to_string(),
        node_id: None,
        message: "Initial commit".to_string(),
        author: Some(CommitAuthor::new("Dev", "dev@example.com", t)),
        committer: Some(CommitAuthor::new("Dev", "dev@example.com", t)),
        tree_sha: "tree888888888888888888888888888888888888".to_string(),
        parents: Vec::new(),
        comment_count: 0,
        verification: None,
    };

    let pinned = PinnedRevision::new(
        target.clone(),
        "9999999999999999999999999999999999999999",
        "tree888888888888888888888888888888888888",
        meta.clone(),
        Some("main".to_string()),
    );

    // Accessors
    assert_eq!(pinned.target(), &target);
    assert_eq!(
        pinned.commit_sha(),
        "9999999999999999999999999999999999999999"
    );
    assert_eq!(pinned.short_sha(), "9999999");
    assert_eq!(
        pinned.root_tree_sha(),
        "tree888888888888888888888888888888888888"
    );
    assert_eq!(pinned.effective_ref(), Some("main"));
    assert_eq!(pinned.metadata().message, "Initial commit");

    // SHA matching
    assert!(pinned.matches_sha("9999999999999999999999999999999999999999"));
    assert!(pinned.matches_sha("9999999"));
    assert!(!pinned.matches_sha("8888888"));

    // CDM RevisionInfo conversion
    let rev_info = pinned.to_revision_info();
    assert_eq!(
        rev_info.git_commit.as_deref(),
        Some("9999999999999999999999999999999999999999")
    );
    assert_eq!(rev_info.git_branch.as_deref(), Some("main"));
    assert_eq!(rev_info.git_tag, None);
    assert!(rev_info.http_last_modified.is_some());

    // CDM SourceIdentity mapping
    let source_id = pinned.to_source_identity("https://github.com/dayashimoga/web-repo-to-ai");
    assert_eq!(source_id.git_ref.as_deref(), Some("main"));
    assert_eq!(source_id.domain.as_deref(), Some("github.com"));

    // Tag target revision mapping
    let tag_target = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/releases/tag/v1.0.0",
    )
    .expect("must parse");
    let pinned_tag = PinnedRevision::new(
        tag_target,
        "9999999999999999999999999999999999999999",
        "tree888888888888888888888888888888888888",
        meta.clone(),
        Some("v1.0.0".to_string()),
    );
    let tag_rev_info = pinned_tag.to_revision_info();
    assert_eq!(tag_rev_info.git_branch, None);
    assert_eq!(tag_rev_info.git_tag.as_deref(), Some("v1.0.0"));

    // Tree target revision mapping
    let tree_target = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/feature-branch/crates/core",
    )
    .expect("must parse");
    let pinned_tree = PinnedRevision::new(
        tree_target,
        "9999999999999999999999999999999999999999",
        "tree888888888888888888888888888888888888",
        meta.clone(),
        Some("feature-branch".to_string()),
    );
    let tree_rev_info = pinned_tree.to_revision_info();
    assert_eq!(tree_rev_info.git_branch.as_deref(), Some("feature-branch"));
    assert_eq!(tree_rev_info.git_tag, None);

    // Repo root with version-like tag ref
    let repo_target = GitHubUrlParser::parse_url("https://github.com/dayashimoga/web-repo-to-ai")
        .expect("must parse");
    let pinned_vtag = PinnedRevision::new(
        repo_target.clone(),
        "9999999999999999999999999999999999999999",
        "tree888888888888888888888888888888888888",
        meta.clone(),
        Some("v2.5.0".to_string()),
    );
    let vtag_rev_info = pinned_vtag.to_revision_info();
    assert_eq!(vtag_rev_info.git_branch, None);
    assert_eq!(vtag_rev_info.git_tag.as_deref(), Some("v2.5.0"));

    // Repo root with no effective ref (None, None)
    let pinned_no_ref = PinnedRevision::new(
        repo_target,
        "9999999999999999999999999999999999999999",
        "tree888888888888888888888888888888888888",
        meta.clone(),
        None,
    );
    let no_ref_info = pinned_no_ref.to_revision_info();
    assert_eq!(no_ref_info.git_branch, None);
    assert_eq!(no_ref_info.git_tag, None);

    // SourceIdentity fallback when effective_ref is None falls back to commit SHA
    let source_id_no_ref =
        pinned_no_ref.to_source_identity("https://github.com/dayashimoga/web-repo-to-ai");
    assert_eq!(
        source_id_no_ref.git_ref.as_deref(),
        Some("9999999999999999999999999999999999999999")
    );

    // Serialization & Deserialization of PinnedRevision
    let pinned_json = serde_json::to_string(&pinned).expect("must serialize pinned revision");
    let pinned_deserialized: PinnedRevision =
        serde_json::from_str(&pinned_json).expect("must deserialize pinned revision");
    assert_eq!(pinned.commit_sha(), pinned_deserialized.commit_sha());
    assert_eq!(pinned.root_tree_sha(), pinned_deserialized.root_tree_sha());
}

// -----------------------------------------------------------------------------
// Live In-Process HTTP Server Integration Tests
// -----------------------------------------------------------------------------

#[tokio::test]
async fn test_live_http_commit_selection_and_pinning() {
    let app = Router::new().route(
        "/repos/:owner/:repo/commits/:commit_ref",
        get(
            |Path((_owner, _repo, commit_ref)): Path<(String, String, String)>| async move {
                let mut headers = AxumHeaderMap::new();
                headers.insert("x-ratelimit-limit", "5000".parse().unwrap());
                headers.insert("x-ratelimit-remaining", "4995".parse().unwrap());
                headers.insert("x-ratelimit-reset", "1700000000".parse().unwrap());

                match commit_ref.as_str() {
                    "HEAD" | "main" | "c0ffee1111111111111111111111111111111111" => {
                        let body = json!({
                            "sha": "c0ffee1111111111111111111111111111111111",
                            "node_id": "MDY6Q29tbWl0MTIzNDU2",
                            "commit": {
                                "message": "feat(core): release production v1.0.0\n\nFull description",
                                "author": {
                                    "name": "Octocat",
                                    "email": "octocat@github.com",
                                    "date": "2026-03-15T12:00:00Z"
                                },
                                "committer": {
                                    "name": "GitHub Staff",
                                    "email": "staff@github.com",
                                    "date": "2026-03-15T12:05:00Z"
                                },
                                "tree": {
                                    "sha": "tree000000000000000000000000000000000000",
                                    "url": "https://api.github.com/repos/owner/repo/git/trees/tree0000"
                                },
                                "comment_count": 5,
                                "verification": {
                                    "verified": true,
                                    "reason": "valid",
                                    "signature": "-----BEGIN PGP SIGNATURE-----\n...",
                                    "payload": "tree tree0000..."
                                }
                            },
                            "parents": [
                                {
                                    "sha": "parent1111111111111111111111111111111111",
                                    "url": "https://api.github.com/repos/owner/repo/commits/parent1111"
                                }
                            ]
                        });
                        (StatusCode::OK, headers, Json(body)).into_response()
                    }
                    "nonexistent" => {
                        let body = json!({
                            "message": "No commit found for SHA: nonexistent",
                            "documentation_url": "https://docs.github.com/rest/commits"
                        });
                        (StatusCode::NOT_FOUND, headers, Json(body)).into_response()
                    }
                    _ => (StatusCode::INTERNAL_SERVER_ERROR, headers, "Internal Server Error").into_response(),
                }
            },
        ),
    );

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("must bind ephemeral test port");
    let local_addr = listener.local_addr().expect("must get local address");
    let server_base_url = format!("http://{}", local_addr);

    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("server must serve requests");
    });

    let client = GitHubApiClient::new()
        .with_base_url_override(&server_base_url)
        .with_auth_token("ghp_TestToken1234567890abcdef12345678")
        .with_timeout(Duration::from_secs(5));

    let target = GitHubUrlParser::parse_url("https://github.com/test-owner/test-repo")
        .expect("must parse target");

    // 1. Get commit by explicit SHA
    let (commit_meta, rate_limit) = client
        .get_commit(&target, "c0ffee1111111111111111111111111111111111")
        .await
        .expect("must retrieve commit");

    assert_eq!(commit_meta.sha, "c0ffee1111111111111111111111111111111111");
    assert_eq!(commit_meta.short_sha(), "c0ffee1");
    assert_eq!(
        commit_meta.summary(),
        "feat(core): release production v1.0.0"
    );
    assert_eq!(
        commit_meta.tree_sha,
        "tree000000000000000000000000000000000000"
    );
    assert_eq!(
        commit_meta.parents,
        vec!["parent1111111111111111111111111111111111".to_string()]
    );
    assert_eq!(commit_meta.comment_count, 5);

    let committer = commit_meta.committer.expect("committer must be present");
    assert_eq!(committer.name, "GitHub Staff");
    assert_eq!(committer.email, "staff@github.com");

    let author = commit_meta.author.expect("author must be present");
    assert_eq!(author.name, "Octocat");
    assert_eq!(author.email, "octocat@github.com");

    let verification = commit_meta
        .verification
        .expect("verification must be present");
    assert!(verification.verified);
    assert_eq!(verification.reason, "valid");

    let rl = rate_limit.expect("rate limit must be parsed");
    assert_eq!(rl.limit, 5000);
    assert_eq!(rl.remaining, 4995);

    // 2. Resolve commit SHA for target with no ref (defaults to HEAD)
    let (resolved_sha, resolved_meta, _) = client
        .resolve_commit_sha(&target, None)
        .await
        .expect("must resolve HEAD commit");
    assert_eq!(resolved_sha, "c0ffee1111111111111111111111111111111111");
    assert_eq!(resolved_meta.sha, resolved_sha);

    // 3. Resolve commit SHA for branch "main"
    let (branch_sha, _, _) = client
        .resolve_commit_sha(&target, Some("main"))
        .await
        .expect("must resolve main branch commit");
    assert_eq!(branch_sha, "c0ffee1111111111111111111111111111111111");

    // 4. Pin revision for target
    let pinned = client
        .pin_revision(&target)
        .await
        .expect("must pin revision");
    assert_eq!(
        pinned.commit_sha(),
        "c0ffee1111111111111111111111111111111111"
    );
    assert_eq!(
        pinned.root_tree_sha(),
        "tree000000000000000000000000000000000000"
    );
    assert_eq!(pinned.short_sha(), "c0ffee1");
    assert!(pinned.matches_sha("c0ffee1"));

    // 5. Commit not found returns ResourceNotFound
    let not_found_err = client
        .get_commit(&target, "nonexistent")
        .await
        .expect_err("nonexistent commit must fail with 404");

    match not_found_err {
        GitHubExtractorError::ResourceNotFound { message, .. } => {
            assert!(message.contains("No commit found"));
            assert!(!message.contains("ghp_TestToken"));
        }
        other => panic!("expected ResourceNotFound, got: {other:?}"),
    }

    // 6. Token redaction verification on 500 server error
    let server_err = client
        .get_commit(&target, "unknown-500-trigger")
        .await
        .expect_err("500 error must fail");

    let err_str = server_err.to_string();
    assert!(!err_str.contains("ghp_TestToken"));
}
