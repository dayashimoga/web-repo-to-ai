//! # URL Conversion Command Integration Tests
//!
//! Validates the full CLI URL extraction, validation, SSRF security blocking,
//! multi-format export, budget optimization, and JSON envelope output.

use std::fs;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

use web_repo_to_ai_cli::{run_cli_with_args, run_cli_with_args_and_options, ExitCode};
use web_repo_to_ai_url_security::{MockDnsResolver, SecureDnsResolver, UrlSecurityPolicy};
use web_repo_to_ai_web_extractor::http::MockHttpTransport;

fn setup_hermetic_fixtures() -> (Arc<MockHttpTransport>, SecureDnsResolver) {
    let policy = UrlSecurityPolicy::default();
    let mock_dns = Arc::new(MockDnsResolver::new());
    mock_dns.set_host(
        "example.com",
        vec![SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(93, 184, 216, 34)),
            443,
        )],
    );
    let resolver = SecureDnsResolver::with_resolver(policy, mock_dns);

    let mock_transport = Arc::new(MockHttpTransport::new());
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <title>AI Engineering Manual</title>
    <meta name="author" content="Dr. Ada Lovelace">
    <meta name="description" content="A comprehensive manual on modern agent systems.">
</head>
<body>
    <article>
        <h1>AI Engineering Manual</h1>
        <p class="byline">By Dr. Ada Lovelace</p>
        <p>Artificial Intelligence systems require rigorous context grounding, deterministic token budgeting, and zero-loss semantic preservation.</p>
        <h2>Section 1: Architecture</h2>
        <p>Building high-performance converters necessitates memory-safe execution, streaming decompression, and clean multi-format serialization.</p>
    </article>
</body>
</html>"#;
    mock_transport.set_html("https://example.com/docs/ai-manual", html);

    (mock_transport, resolver)
}

#[tokio::test]
async fn test_cli_help_and_version() {
    let code = run_cli_with_args(["web-repo-to-ai", "--help"]).await;
    assert_eq!(code.unwrap(), 0);

    let code_url = run_cli_with_args(["web-repo-to-ai", "url", "--help"]).await;
    assert_eq!(code_url.unwrap(), 0);

    let code_ver = run_cli_with_args(["web-repo-to-ai", "--version"]).await;
    assert_eq!(code_ver.unwrap(), 0);
}

#[tokio::test]
async fn test_invalid_args_and_validation_errors() {
    // Missing URL parameter
    let res = run_cli_with_args(["web-repo-to-ai", "url"]).await;
    assert!(res.is_err());
    let err = res.unwrap_err();
    assert_eq!(err.exit_code(), ExitCode::Validation.code());

    // Invalid format
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "url",
        "https://example.com/doc",
        "--format",
        "invalid_fmt",
    ])
    .await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().exit_code(), ExitCode::Validation.code());

    // Forbidden scheme (ftp://)
    let res = run_cli_with_args(["web-repo-to-ai", "url", "ftp://example.com/file"]).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().exit_code(), ExitCode::Validation.code());

    // Malformed URL syntax
    let res = run_cli_with_args(["web-repo-to-ai", "url", "not_a_valid_url"]).await;
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().exit_code(), ExitCode::Validation.code());
}

#[tokio::test]
async fn test_security_ssrf_blocking_exit_code() {
    // IPv4 Loopback
    let res = run_cli_with_args(["web-repo-to-ai", "url", "http://127.0.0.1/admin"]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );

    // RFC 1918 Private IP
    let res = run_cli_with_args(["web-repo-to-ai", "url", "http://192.168.1.1/router"]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );

    // Cloud Metadata endpoint
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "url",
        "http://169.254.169.254/latest/meta-data",
    ])
    .await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );

    // Blocked network port (SSH port 22)
    let res = run_cli_with_args(["web-repo-to-ai", "url", "http://example.com:22/service"]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err().exit_code(),
        ExitCode::SecurityOrNetwork.code()
    );
}

#[tokio::test]
async fn test_local_url_conversion_stdout_markdown() {
    let (transport, resolver) = setup_hermetic_fixtures();
    let res = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "url",
            "https://example.com/docs/ai-manual",
            "--quiet",
        ],
        Some(transport),
        Some(resolver),
    )
    .await;

    assert!(res.is_ok());
    assert_eq!(res.unwrap(), 0);
}

#[tokio::test]
async fn test_local_url_conversion_multi_format_file_outputs() {
    let (transport, resolver) = setup_hermetic_fixtures();
    let temp_dir =
        std::env::temp_dir().join(format!("web_repo_to_ai_cli_test_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let formats = [
        ("markdown", "manual.md"),
        ("txt", "manual.txt"),
        ("json", "manual.json"),
        ("html", "manual.html"),
        ("jsonl", "manual.jsonl"),
        ("zip", "package.zip"),
        ("manifest", "manifest.json"),
        ("checksums", "checksums.txt"),
    ];

    for (fmt, filename) in formats {
        let output_path = temp_dir.join(filename);
        let path_str = output_path.to_string_lossy().to_string();

        let res = run_cli_with_args_and_options(
            [
                "web-repo-to-ai",
                "url",
                "https://example.com/docs/ai-manual",
                "--format",
                fmt,
                "-o",
                &path_str,
                "--quiet",
            ],
            Some(transport.clone()),
            Some(resolver.clone()),
        )
        .await;

        assert!(res.is_ok(), "Conversion failed for format {fmt}");
        assert_eq!(res.unwrap(), 0);
        assert!(
            output_path.exists(),
            "Output file {path_str} was not created"
        );

        let metadata = fs::metadata(&output_path).unwrap();
        assert!(metadata.len() > 0, "Output file {path_str} is empty");

        // Specific format validations
        match fmt {
            "json" | "manifest" => {
                let content = fs::read_to_string(&output_path).unwrap();
                let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
                assert!(parsed.is_object());
            }
            "jsonl" => {
                let content = fs::read_to_string(&output_path).unwrap();
                for line in content.lines() {
                    let parsed: serde_json::Value = serde_json::from_str(line).unwrap();
                    assert!(parsed.is_object());
                }
            }
            "zip" => {
                let bytes = fs::read(&output_path).unwrap();
                assert!(bytes.len() >= 4);
                // Standard PK zip header: 0x50, 0x4B, 0x03, 0x04
                assert_eq!(&bytes[0..4], b"PK\x03\x04");
            }
            _ => {}
        }
    }

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_budget_optimization_and_no_optimize_flag() {
    let (transport, resolver) = setup_hermetic_fixtures();
    let temp_dir =
        std::env::temp_dir().join(format!("web_repo_to_ai_budget_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();

    let file_optimized = temp_dir.join("opt.md");
    let file_unoptimized = temp_dir.join("unopt.md");

    // 1. Run with tight budget
    let res1 = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "url",
            "https://example.com/docs/ai-manual",
            "--budget",
            "50",
            "-o",
            &file_optimized.to_string_lossy(),
            "--quiet",
        ],
        Some(transport.clone()),
        Some(resolver.clone()),
    )
    .await;
    assert_eq!(res1.unwrap(), 0);

    // 2. Run with --no-optimize
    let res2 = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "url",
            "https://example.com/docs/ai-manual",
            "--no-optimize",
            "-o",
            &file_unoptimized.to_string_lossy(),
            "--quiet",
        ],
        Some(transport),
        Some(resolver),
    )
    .await;
    assert_eq!(res2.unwrap(), 0);

    assert!(file_optimized.exists());
    assert!(file_unoptimized.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_json_execution_envelope() {
    let (transport, resolver) = setup_hermetic_fixtures();
    let temp_dir = std::env::temp_dir().join(format!("web_repo_to_ai_json_{}", std::process::id()));
    fs::create_dir_all(&temp_dir).unwrap();
    let out_file = temp_dir.join("output.md");

    let res = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "url",
            "https://example.com/docs/ai-manual",
            "--json",
            "-o",
            &out_file.to_string_lossy(),
        ],
        Some(transport),
        Some(resolver),
    )
    .await;

    assert_eq!(res.unwrap(), 0);
    assert!(out_file.exists());

    let _ = fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_verbose_and_quiet_flags() {
    let (transport, resolver) = setup_hermetic_fixtures();

    let res_v = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "-v",
            "url",
            "https://example.com/docs/ai-manual",
        ],
        Some(transport.clone()),
        Some(resolver.clone()),
    )
    .await;
    assert_eq!(res_v.unwrap(), 0);

    let res_q = run_cli_with_args_and_options(
        [
            "web-repo-to-ai",
            "-q",
            "url",
            "https://example.com/docs/ai-manual",
        ],
        Some(transport),
        Some(resolver),
    )
    .await;
    assert_eq!(res_q.unwrap(), 0);
}
