//! Integration tests for GitHub repository URL identification, parsing,
//! and target decomposition (`crates/github-extractor`).

use std::str::FromStr;
use web_repo_to_ai_github_extractor::{
    is_default_host, is_reserved_github_path, validate_and_sanitize_subpath, validate_owner,
    validate_repo, GitHubExtractorError, GitHubTarget, GitHubTargetType, GitHubUrlParser,
    DEFAULT_GITHUB_API_BASE, DEFAULT_GITHUB_HOST,
};

#[test]
fn test_parse_standard_https_repo_urls() {
    let urls = [
        "https://github.com/rust-lang/rust",
        "https://github.com/rust-lang/rust/",
        "https://github.com/rust-lang/rust.git",
        "https://www.github.com/rust-lang/rust",
    ];

    for url in urls {
        let target = GitHubUrlParser::parse_url(url).expect("must parse standard HTTPS repo URL");
        assert_eq!(target.owner(), "rust-lang");
        assert_eq!(target.repo(), "rust");
        assert_eq!(target.target_type(), &GitHubTargetType::RepositoryRoot);
        assert_eq!(target.ref_spec(), None);
        assert_eq!(target.subpath(), None);
        assert!(target.is_root());
        assert!(!target.is_tree());
        assert!(!target.is_blob());
        assert!(!target.is_commit());
        assert_eq!(target.nwo(), "rust-lang/rust");
        assert_eq!(
            target.canonical_repo_url(),
            "https://github.com/rust-lang/rust"
        );
        assert_eq!(
            target.clone_url_https(),
            "https://github.com/rust-lang/rust.git"
        );
        assert_eq!(target.clone_url_ssh(), "git@github.com:rust-lang/rust.git");
        assert_eq!(target.api_base_url(), DEFAULT_GITHUB_API_BASE);
        assert_eq!(
            target.api_repo_url(),
            "https://api.github.com/repos/rust-lang/rust"
        );
        assert_eq!(
            target.api_trees_url("abc1234", true),
            "https://api.github.com/repos/rust-lang/rust/git/trees/abc1234?recursive=1"
        );
        assert_eq!(
            target.api_commits_url(Some("main")),
            "https://api.github.com/repos/rust-lang/rust/commits?sha=main"
        );
    }
}

#[test]
fn test_from_str_trait_implementation() {
    let target = GitHubTarget::from_str("https://github.com/dayashimoga/web-repo-to-ai")
        .expect("FromStr trait must work");
    assert_eq!(target.owner(), "dayashimoga");
    assert_eq!(target.repo(), "web-repo-to-ai");
}

#[test]
fn test_parse_http_normalized_to_https() {
    let target = GitHubUrlParser::parse_url("http://github.com/tokio-rs/tokio")
        .expect("must parse http URL");
    assert_eq!(target.owner(), "tokio-rs");
    assert_eq!(target.repo(), "tokio");
    assert_eq!(
        target.canonical_repo_url(),
        "https://github.com/tokio-rs/tokio"
    );
}

#[test]
fn test_parse_shorthand_notation() {
    let target = GitHubUrlParser::parse_url("torvalds/linux")
        .expect("must parse simple shorthand owner/repo");
    assert_eq!(target.owner(), "torvalds");
    assert_eq!(target.repo(), "linux");
    assert_eq!(target.target_type(), &GitHubTargetType::RepositoryRoot);
    assert_eq!(target.ref_spec(), None);

    let target_ref =
        GitHubUrlParser::parse_url("torvalds/linux@v6.8").expect("must parse shorthand with @ref");
    assert_eq!(target_ref.owner(), "torvalds");
    assert_eq!(target_ref.repo(), "linux");
    assert_eq!(target_ref.ref_spec(), Some("v6.8"));
    assert_eq!(
        target_ref.canonical_target_url(),
        "https://github.com/torvalds/linux/tree/v6.8"
    );
}

#[test]
fn test_parse_ssh_urls() {
    let target1 = GitHubUrlParser::parse_url("git@github.com:serde-rs/serde.git")
        .expect("must parse git@ SSH URL");
    assert_eq!(target1.owner(), "serde-rs");
    assert_eq!(target1.repo(), "serde");
    assert_eq!(target1.host(), DEFAULT_GITHUB_HOST);

    let target2 = GitHubUrlParser::parse_url("ssh://git@github.com:serde-rs/serde")
        .expect("must parse ssh://git@ URL");
    assert_eq!(target2.owner(), "serde-rs");
    assert_eq!(target2.repo(), "serde");

    let target3 =
        GitHubUrlParser::parse_url("git@github.com:serde-rs/serde/tree/master/serde_derive")
            .expect("must parse SSH URL with tree and subpath");
    assert_eq!(target3.owner(), "serde-rs");
    assert_eq!(target3.repo(), "serde");
    assert_eq!(target3.ref_spec(), Some("master"));
    assert_eq!(target3.subpath(), Some("serde_derive"));
}

#[test]
fn test_parse_git_transport_urls() {
    let target = GitHubUrlParser::parse_url("git://github.com/facebook/react.git")
        .expect("must parse git:// URL");
    assert_eq!(target.owner(), "facebook");
    assert_eq!(target.repo(), "react");
}

#[test]
fn test_parse_tree_and_subpath_urls() {
    // Branch root
    let target_tree_root =
        GitHubUrlParser::parse_url("https://github.com/dayashimoga/web-repo-to-ai/tree/main")
            .expect("must parse tree root");
    assert_eq!(target_tree_root.ref_spec(), Some("main"));
    assert_eq!(target_tree_root.subpath(), None);
    assert!(target_tree_root.is_tree());

    // Tree with single-level subdirectory
    let target_tree_sub = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates",
    )
    .expect("must parse tree with subpath");
    assert_eq!(target_tree_sub.ref_spec(), Some("main"));
    assert_eq!(target_tree_sub.subpath(), Some("crates"));
    assert!(target_tree_sub.is_tree());
    assert_eq!(
        target_tree_sub.canonical_target_url(),
        "https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates"
    );

    // Tree with deeply nested subdirectory
    let target_deep = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/v1.0.0/crates/github-extractor/src",
    )
    .expect("must parse deep tree");
    assert_eq!(target_deep.ref_spec(), Some("v1.0.0"));
    assert_eq!(target_deep.subpath(), Some("crates/github-extractor/src"));
}

#[test]
fn test_parse_blob_and_raw_urls() {
    let target_blob = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/blob/main/crates/cdm/Cargo.toml",
    )
    .expect("must parse blob URL");

    assert!(target_blob.is_blob());
    assert_eq!(target_blob.ref_spec(), Some("main"));
    assert_eq!(target_blob.subpath(), Some("crates/cdm/Cargo.toml"));
    assert_eq!(
        target_blob.canonical_target_url(),
        "https://github.com/dayashimoga/web-repo-to-ai/blob/main/crates/cdm/Cargo.toml"
    );
    assert_eq!(
        target_blob.raw_blob_url("main", "crates/cdm/Cargo.toml"),
        "https://raw.githubusercontent.com/dayashimoga/web-repo-to-ai/main/crates/cdm/Cargo.toml"
    );

    let target_raw = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/raw/main/README.md",
    )
    .expect("must parse raw URL");
    assert!(target_raw.is_blob());
    assert_eq!(target_raw.subpath(), Some("README.md"));
}

#[test]
fn test_parse_commit_shas() {
    // 7-character short SHA
    let target_short =
        GitHubUrlParser::parse_url("https://github.com/rust-lang/rust/commit/7ee62dd")
            .expect("must parse short commit SHA");
    assert!(target_short.is_commit());
    assert_eq!(target_short.ref_spec(), Some("7ee62dd"));

    // 40-character full SHA
    let full_sha = "0123456789abcdef0123456789abcdef01234567";
    let target_full = GitHubUrlParser::parse_url(&format!(
        "https://github.com/rust-lang/rust/commit/{full_sha}"
    ))
    .expect("must parse full commit SHA");
    assert!(target_full.is_commit());
    assert_eq!(target_full.ref_spec(), Some(full_sha));

    // Malformed commit SHA (non-hex chars)
    let err_non_hex =
        GitHubUrlParser::parse_url("https://github.com/rust-lang/rust/commit/xyz123").unwrap_err();
    assert!(matches!(
        err_non_hex,
        GitHubExtractorError::InvalidRef { .. }
    ));
}

#[test]
fn test_parse_releases_tags_commits_and_pulls() {
    // Releases list
    let target_rel = GitHubUrlParser::parse_url("https://github.com/tokio-rs/tokio/releases")
        .expect("must parse releases URL");
    assert_eq!(target_rel.target_type(), &GitHubTargetType::Releases);

    // Release tag
    let target_tag =
        GitHubUrlParser::parse_url("https://github.com/tokio-rs/tokio/releases/tag/tokio-1.40.0")
            .expect("must parse release tag");
    assert_eq!(
        target_tag.target_type(),
        &GitHubTargetType::Tag {
            tag_name: "tokio-1.40.0".to_string()
        }
    );
    assert_eq!(target_tag.ref_spec(), Some("tokio-1.40.0"));

    // Commits view
    let target_commits =
        GitHubUrlParser::parse_url("https://github.com/tokio-rs/tokio/commits/master")
            .expect("must parse commits view");
    assert_eq!(
        target_commits.target_type(),
        &GitHubTargetType::Commits {
            ref_name: Some("master".to_string())
        }
    );

    // Pull request
    let target_pr = GitHubUrlParser::parse_url("https://github.com/rust-lang/rust/pull/12345")
        .expect("must parse PR URL");
    assert_eq!(
        target_pr.target_type(),
        &GitHubTargetType::PullRequest { number: 12345 }
    );

    // Issue
    let target_issue = GitHubUrlParser::parse_url("https://github.com/rust-lang/rust/issues/67890")
        .expect("must parse Issue URL");
    assert_eq!(
        target_issue.target_type(),
        &GitHubTargetType::Issue { number: 67890 }
    );
}

#[test]
fn test_path_traversal_attacks_strictly_rejected() {
    let traversal_urls = [
        "https://github.com/owner/repo/tree/main/../secret",
        "https://github.com/owner/repo/tree/main/foo/../../bar",
        "https://github.com/owner/repo/blob/main/..%2fsecret",
        "https://github.com/owner/repo/tree/main/dir/%2e%2e/passwd",
    ];

    for url in traversal_urls {
        let res = GitHubUrlParser::parse_url(url);
        assert!(
            res.is_err(),
            "Expected traversal attempt '{url}' to be rejected"
        );
        match res.unwrap_err() {
            GitHubExtractorError::InvalidSubpath { .. } | GitHubExtractorError::UrlSecurity(_) => {}
            other => panic!("Unexpected error type for traversal '{url}': {other:?}"),
        }
    }
}

#[test]
fn test_subpath_sanitization_and_normalization() {
    assert_eq!(
        validate_and_sanitize_subpath("crates\\github-extractor\\src").unwrap(),
        Some("crates/github-extractor/src".to_string())
    );
    assert_eq!(
        validate_and_sanitize_subpath("///crates//cdm///").unwrap(),
        Some("crates/cdm".to_string())
    );
    assert_eq!(validate_and_sanitize_subpath("   ").unwrap(), None);
    assert_eq!(validate_and_sanitize_subpath("").unwrap(), None);
    assert!(validate_and_sanitize_subpath("foo/../bar").is_err());
    assert!(validate_and_sanitize_subpath("foo/\0/bar").is_err());
}

#[test]
fn test_security_embedded_credentials_rejected() {
    let cred_urls = [
        "https://user:password@github.com/owner/repo",
        "https://user@github.com/owner/repo",
    ];

    for url in cred_urls {
        let res = GitHubUrlParser::parse_url(url);
        assert!(
            res.is_err(),
            "Credentials in URL '{url}' must be rejected for security"
        );
    }
}

#[test]
fn test_security_ssrf_and_invalid_hosts_rejected() {
    let bad_hosts = [
        "https://127.0.0.1/owner/repo",
        "https://169.254.169.254/owner/repo",
        "https://10.0.0.1/owner/repo",
        "https://evil-spoof.com/owner/repo",
        "https://gitlab.com/owner/repo",
    ];

    for url in bad_hosts {
        let res = GitHubUrlParser::parse_url(url);
        assert!(res.is_err(), "Non-GitHub host '{url}' must be rejected");
    }
}

#[test]
fn test_reserved_github_system_paths_rejected() {
    let reserved_urls = [
        "https://github.com/features",
        "https://github.com/pricing",
        "https://github.com/login",
        "https://github.com/settings",
        "https://github.com/explore",
        "https://github.com/trending",
    ];

    for url in reserved_urls {
        let res = GitHubUrlParser::parse_url(url);
        assert!(
            res.is_err(),
            "Reserved GitHub path '{url}' must not be parsed as a repository"
        );
        match res.unwrap_err() {
            GitHubExtractorError::ReservedPath(_) => {}
            other => panic!("Expected ReservedPath error for '{url}', got {other:?}"),
        }
    }

    assert!(is_reserved_github_path("features"));
    assert!(is_reserved_github_path("Pricing"));
    assert!(!is_reserved_github_path("rust-lang"));
}

#[test]
fn test_user_or_org_profile_urls_rejected() {
    let res = GitHubUrlParser::parse_url("https://github.com/torvalds");
    assert!(res.is_err());
}

#[test]
fn test_owner_name_validation() {
    assert!(validate_owner("rust-lang").is_ok());
    assert!(validate_owner("dayashimoga").is_ok());
    assert!(validate_owner("a").is_ok());
    assert!(validate_owner("a-b-c").is_ok());

    // Invalid owner cases
    assert!(validate_owner("").is_err());
    assert!(validate_owner("-starts-with-hyphen").is_err());
    assert!(validate_owner("ends-with-hyphen-").is_err());
    assert!(validate_owner("consecutive--hyphens").is_err());
    assert!(validate_owner("invalid$char").is_err());
    assert!(validate_owner(&"a".repeat(40)).is_err()); // Max 39 chars
}

#[test]
fn test_repo_name_validation() {
    assert!(validate_repo("web-repo-to-ai").is_ok());
    assert!(validate_repo("tokio_util").is_ok());
    assert!(validate_repo("v1.2.3").is_ok());

    // Invalid repo cases
    assert!(validate_repo("").is_err());
    assert!(validate_repo(".").is_err());
    assert!(validate_repo("..").is_err());
    assert!(validate_repo("invalid/slash").is_err());
    assert!(validate_repo(&"a".repeat(101)).is_err()); // Max 100 chars
}

#[test]
fn test_custom_github_enterprise_host() {
    assert!(is_default_host("github.com"));
    assert!(is_default_host("www.github.com"));
    assert!(!is_default_host("github.internal.corp"));

    let parser = GitHubUrlParser::new().with_allowed_host("github.internal.corp");

    let target = parser
        .parse("https://github.internal.corp/enterprise-org/internal-tool/tree/master/docs")
        .expect("must parse enterprise URL with configured host");

    assert_eq!(target.owner(), "enterprise-org");
    assert_eq!(target.repo(), "internal-tool");
    assert_eq!(target.host(), "github.internal.corp");
    assert!(target.is_enterprise);
    assert!(!target.is_default_host());
    assert_eq!(target.api_base_url(), "https://github.internal.corp/api/v3");
    assert_eq!(
        target.api_repo_url(),
        "https://github.internal.corp/api/v3/repos/enterprise-org/internal-tool"
    );
    assert_eq!(
        target.raw_blob_url("master", "docs/index.md"),
        "https://github.internal.corp/raw/enterprise-org/internal-tool/master/docs/index.md"
    );
}

#[test]
fn test_cdm_source_identity_mapping() {
    let target = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates/core",
    )
    .expect("must parse target");

    let identity = target
        .to_source_identity("https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates/core");
    assert_eq!(
        identity.canonical_url,
        "https://github.com/dayashimoga/web-repo-to-ai/tree/main/crates/core"
    );
    assert_eq!(identity.domain, Some("github.com".to_string()));
    assert_eq!(identity.repo_owner, Some("dayashimoga".to_string()));
    assert_eq!(identity.repo_name, Some("web-repo-to-ai".to_string()));
    assert_eq!(identity.git_ref, Some("main".to_string()));
    assert_eq!(identity.subpath, Some("crates/core".to_string()));
}

#[test]
fn test_ref_refinement_for_slashed_branch_names() {
    // If a URL like .../tree/feature/sub/path was initially parsed with ref "feature" and subpath "sub/path",
    // once Phase 5.1.2 discovers remote branch "feature/sub", it can refine the target.
    let initial = GitHubUrlParser::parse_url(
        "https://github.com/dayashimoga/web-repo-to-ai/tree/feature/sub/path",
    )
    .expect("must parse");

    assert_eq!(initial.ref_spec(), Some("feature"));
    assert_eq!(initial.subpath(), Some("sub/path"));

    let refined = initial.with_refined_ref_and_subpath("feature/sub", Some("path".to_string()));
    assert_eq!(refined.ref_spec(), Some("feature/sub"));
    assert_eq!(refined.subpath(), Some("path"));
    assert_eq!(
        refined.canonical_target_url(),
        "https://github.com/dayashimoga/web-repo-to-ai/tree/feature/sub/path"
    );
}
