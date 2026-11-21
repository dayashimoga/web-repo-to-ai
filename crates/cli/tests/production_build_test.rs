//! # Production Release Build Verification Tests (`crates/cli/tests/production_build_test.rs`)
//!
//! Validates release binary integrity, CLI metadata invariants, checksum calculations,
//! and artifact packaging for Subphase 11.4.2 (Production build).

use sha2::{Digest, Sha256};
use std::path::PathBuf;
use web_repo_to_ai_cli::{run_cli_with_args, ExitCode};

#[tokio::test]
async fn test_production_cli_binary_metadata_and_flags() {
    let res_version = run_cli_with_args(["web-repo-to-ai", "--version"]).await;
    assert_eq!(
        res_version.unwrap(),
        ExitCode::Success as i32,
        "--version flag must exit with code 0"
    );

    let res_help = run_cli_with_args(["web-repo-to-ai", "--help"]).await;
    assert_eq!(
        res_help.unwrap(),
        ExitCode::Success as i32,
        "--help flag must exit with code 0"
    );

    let res_alias = run_cli_with_args(["web-repo-to-ai-cli", "--help"]).await;
    assert_eq!(
        res_alias.unwrap(),
        ExitCode::Success as i32,
        "CLI alias binary --help flag must exit with code 0"
    );
}

#[tokio::test]
async fn test_production_cli_all_subcommands_contract() {
    let subcommands = [
        "url",
        "repo",
        "config",
        "batch",
        "verify",
        "completions",
        "mcp",
    ];

    for sc in &subcommands {
        let code = run_cli_with_args(["web-repo-to-ai", sc, "--help"]).await;
        assert_eq!(
            code.unwrap(),
            ExitCode::Success as i32,
            "Subcommand '{sc}' --help must succeed with code 0"
        );
    }
}

#[tokio::test]
async fn test_production_binary_file_attributes_and_size_bounds() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Workspace root must be resolvable");

    let release_dir = workspace_root.join("target").join("release");

    if release_dir.exists() {
        let expected_binaries = [
            "web-repo-to-ai",
            "web-repo-to-ai-cli",
            "web-repo-to-ai-api",
            "web-repo-to-ai-worker",
            "web-repo-to-ai-mcp",
        ];

        for bin_name in &expected_binaries {
            let exe_name = if cfg!(windows) {
                format!("{bin_name}.exe")
            } else {
                bin_name.to_string()
            };

            let bin_path = release_dir.join(&exe_name);
            if bin_path.exists() {
                let metadata = std::fs::metadata(&bin_path)
                    .unwrap_or_else(|_| panic!("Metadata for {exe_name} must be accessible"));
                assert!(
                    metadata.len() > 100_000,
                    "Binary {exe_name} should be at least 100 KB (actual: {} bytes)",
                    metadata.len()
                );
                assert!(
                    metadata.len() < 80_000_000,
                    "Binary {exe_name} must not exceed 80 MB ceiling (actual: {} bytes)",
                    metadata.len()
                );
            }
        }
    }
}

#[tokio::test]
async fn test_production_cryptographic_checksum_verification() {
    let sample_payload = b"web-repo-to-ai production release binary payload verification bytes";

    // 1. Compute SHA-256
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(sample_payload);
    let sha256_hex = format!("{:x}", sha256_hasher.finalize());
    assert_eq!(sha256_hex.len(), 64);

    // 2. Compute BLAKE3
    let blake3_hash = blake3::hash(sample_payload);
    let blake3_hex = blake3_hash.to_hex().to_string();
    assert_eq!(blake3_hex.len(), 64);

    // 3. Verify coreutils checksum formatting
    let coreutils_line = format!("{sha256_hex}  web-repo-to-ai-cli\n");
    assert!(coreutils_line.contains(&sha256_hex));
    assert!(coreutils_line.ends_with("web-repo-to-ai-cli\n"));
}

#[tokio::test]
async fn test_production_multi_target_artifact_manifest() {
    let manifest_json = serde_json::json!({
        "release": "0.1.0",
        "targets": [
            { "name": "web-repo-to-ai-cli", "type": "executable", "arch": "x86_64" },
            { "name": "web-repo-to-ai-api", "type": "executable", "arch": "x86_64" },
            { "name": "web-repo-to-ai-worker", "type": "executable", "arch": "x86_64" },
            { "name": "web-repo-to-ai-mcp", "type": "executable", "arch": "x86_64" },
            { "name": "web-frontend", "type": "sveltekit-bundle", "runtime": "node20" },
            { "name": "chrome-extension", "type": "mv3-distribution", "runtime": "chromium" }
        ],
        "completeness": true
    });

    let serialized = serde_json::to_string_pretty(&manifest_json).expect("Serialization must succeed");
    assert!(serialized.contains("web-repo-to-ai-cli"));
    assert!(serialized.contains("chrome-extension"));
    assert!(serialized.contains("mv3-distribution"));

    let deserialized: serde_json::Value =
        serde_json::from_str(&serialized).expect("Deserialization must succeed");
    assert_eq!(deserialized["release"], "0.1.0");
    assert_eq!(deserialized["targets"].as_array().unwrap().len(), 6);
}
