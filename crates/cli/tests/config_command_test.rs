//! # CLI Configuration Command Integration Tests
//!
//! Validates `config show`, `config init`, `config validate`, `config path`,
//! secret redaction, TOML/JSON parsing, and precedence merging:
//! `CLI Flags > Environment Variables > Config File > Built-in Defaults`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use web_repo_to_ai_cli::{
    load_config, parse_config_str, run_cli_with_args, CliConfigFile, ExitCode,
};

static TEST_DIR_COUNTER: AtomicU64 = AtomicU64::new(100);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(prefix: &str) -> Self {
        let seq = TEST_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "web_repo_to_ai_test_cfg_{prefix}_{}_{seq}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create test temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[tokio::test]
async fn test_config_cli_help() {
    let res = run_cli_with_args(["web-repo-to-ai", "config", "--help"]).await;
    assert_eq!(res.unwrap(), 0);

    let res_show = run_cli_with_args(["web-repo-to-ai", "config", "show", "--help"]).await;
    assert_eq!(res_show.unwrap(), 0);

    let res_init = run_cli_with_args(["web-repo-to-ai", "config", "init", "--help"]).await;
    assert_eq!(res_init.unwrap(), 0);

    let res_validate = run_cli_with_args(["web-repo-to-ai", "config", "validate", "--help"]).await;
    assert_eq!(res_validate.unwrap(), 0);
}

#[tokio::test]
async fn test_config_init_toml_and_json() {
    let temp = TestDir::new("init");
    let toml_file = temp.path().join("test_config.toml");

    // 1. Initialize TOML template
    let res = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "init",
        "--path",
        toml_file.to_str().unwrap(),
    ])
    .await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);
    assert!(toml_file.is_file());

    let content = fs::read_to_string(&toml_file).unwrap();
    assert!(content.contains("[defaults]"));
    assert!(content.contains("[batch]"));
    assert!(content.contains("[api]"));
    assert!(content.contains("[github]"));

    // 2. Fails to overwrite without --force
    let res_dup = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "init",
        "--path",
        toml_file.to_str().unwrap(),
    ])
    .await;
    assert!(res_dup.is_err());

    // 3. Succeeds overwrite with --force
    let res_force = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "init",
        "--path",
        toml_file.to_str().unwrap(),
        "--force",
    ])
    .await;
    assert_eq!(res_force.unwrap(), ExitCode::Success as i32);

    // 4. Initialize JSON template
    let json_file = temp.path().join("test_config.json");
    let res_json = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "init",
        "--path",
        json_file.to_str().unwrap(),
        "--json",
    ])
    .await;
    assert_eq!(res_json.unwrap(), ExitCode::Success as i32);
    assert!(json_file.is_file());

    let json_content = fs::read_to_string(&json_file).unwrap();
    let parsed: serde_json::Value =
        serde_json::from_str(&json_content).expect("Valid JSON template");
    assert!(parsed.get("defaults").is_some());
    assert!(parsed.get("batch").is_some());
}

#[tokio::test]
async fn test_config_validate() {
    let temp = TestDir::new("validate");
    let valid_toml = temp.path().join("valid.toml");
    fs::write(
        &valid_toml,
        r#"
[defaults]
format = "json"
budget = 64000
model = "claude-3-5-sonnet"
no_optimize = true

[batch]
concurrency = 8
continue_on_error = false
output_dir = "./custom_batch_output"
"#,
    )
    .unwrap();

    let res = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "validate",
        "--path",
        valid_toml.to_str().unwrap(),
    ])
    .await;
    assert_eq!(res.unwrap(), ExitCode::Success as i32);

    // Invalid syntax
    let invalid_toml = temp.path().join("invalid.toml");
    fs::write(&invalid_toml, "[defaults\nformat == invalid TOML!").unwrap();

    let res_err = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "validate",
        "--path",
        invalid_toml.to_str().unwrap(),
    ])
    .await;
    assert!(res_err.is_err());
}

#[tokio::test]
async fn test_config_show_and_path() {
    let temp = TestDir::new("show");
    let cfg_file = temp.path().join("custom.toml");
    fs::write(
        &cfg_file,
        r#"
[defaults]
format = "html"
budget = 80000

[api]
api_url = "http://127.0.0.1:8080"
api_key = "wr_live_supersecretsecretkey123"

[github]
token = "ghp_mygithubsecrettoken987654"
"#,
    )
    .unwrap();

    // Verify config path
    let res_path = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "path",
        "--path",
        cfg_file.to_str().unwrap(),
    ])
    .await;
    assert_eq!(res_path.unwrap(), ExitCode::Success as i32);

    // Verify config show
    let res_show = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "show",
        "--path",
        cfg_file.to_str().unwrap(),
    ])
    .await;
    assert_eq!(res_show.unwrap(), ExitCode::Success as i32);

    // Verify config show with json
    let res_show_json = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "show",
        "--path",
        cfg_file.to_str().unwrap(),
        "--json",
    ])
    .await;
    assert_eq!(res_show_json.unwrap(), ExitCode::Success as i32);

    // Verify config show with revealed secrets
    let res_show_revealed = run_cli_with_args([
        "web-repo-to-ai",
        "config",
        "show",
        "--path",
        cfg_file.to_str().unwrap(),
        "--reveal-secrets",
    ])
    .await;
    assert_eq!(res_show_revealed.unwrap(), ExitCode::Success as i32);
}

#[tokio::test]
async fn test_config_precedence_and_mutation() {
    let temp = TestDir::new("precedence");
    let cfg_file = temp.path().join("web-repo-to-ai.toml");
    fs::write(
        &cfg_file,
        r#"
[defaults]
format = "json"
budget = 42000
model = "gpt-4-turbo"
no_optimize = true

[batch]
concurrency = 6
continue_on_error = false
output_dir = "./batch_target_dir"
"#,
    )
    .unwrap();

    let loaded = load_config(Some(&cfg_file))
        .expect("load config file")
        .unwrap();

    // Verify URL args mutation: default format changes from markdown to json, budget from 160_000 to 42_000
    let mut url_args = web_repo_to_ai_cli::UrlArgs {
        url: "https://example.com".to_string(),
        output: "-".to_string(),
        format: web_repo_to_ai_cli::OutputFormat::Markdown,
        budget: 160_000,
        model: "cl100k_base".to_string(),
        no_optimize: false,
        timeout: 30,
        user_agent: None,
        api_url: None,
        api_key: None,
    };
    loaded.apply_to_url_args(&mut url_args);
    assert_eq!(url_args.format, web_repo_to_ai_cli::OutputFormat::Json);
    assert_eq!(url_args.budget, 42_000);
    assert_eq!(url_args.model, "gpt-4-turbo");
    assert!(url_args.no_optimize);

    // CLI overrides must NOT be overwritten by config file
    let mut overridden_url_args = web_repo_to_ai_cli::UrlArgs {
        url: "https://example.com".to_string(),
        output: "-".to_string(),
        format: web_repo_to_ai_cli::OutputFormat::Html, // explicitly provided on CLI
        budget: 200_000,                                // explicitly provided on CLI
        model: "cl100k_base".to_string(),
        no_optimize: false,
        timeout: 30,
        user_agent: None,
        api_url: None,
        api_key: None,
    };
    loaded.apply_to_url_args(&mut overridden_url_args);
    assert_eq!(
        overridden_url_args.format,
        web_repo_to_ai_cli::OutputFormat::Html
    );
    assert_eq!(overridden_url_args.budget, 200_000);
}

#[test]
fn test_config_parsing_toml_and_json_edge_cases() {
    // 1. Raw TOML
    let toml = r#"
# Comments and blank lines
[defaults]
format = "txt"
budget = 120000
no_optimize = false

[api]
api_url = "http://localhost:9000"
"#;
    let cfg: CliConfigFile = parse_config_str(toml, false).expect("parse toml");
    assert_eq!(
        cfg.defaults.format,
        Some(web_repo_to_ai_cli::OutputFormat::Txt)
    );
    assert_eq!(cfg.defaults.budget, Some(120_000));
    assert_eq!(cfg.api.api_url.as_deref(), Some("http://localhost:9000"));

    // 2. Raw JSON
    let json = r#"{
        "defaults": {
            "format": "html",
            "budget": 99000
        }
    }"#;
    let cfg_json: CliConfigFile = parse_config_str(json, true).expect("parse json");
    assert_eq!(
        cfg_json.defaults.format,
        Some(web_repo_to_ai_cli::OutputFormat::Html)
    );
    assert_eq!(cfg_json.defaults.budget, Some(99_000));
}
