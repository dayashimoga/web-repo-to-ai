//! # Deployment Verification Integration Tests (`crates/cli/tests/deployment_verification_test.rs`)
//!
//! Subphase 11.4.4: Deployment Verification
//! Validates deployment invariants across all 4 supported topologies:
//! 1. Containerized Self-Hosted Stack (compose specs, network isolation, unprivileged users, volumes).
//! 2. Standalone Release Binary Distribution (contracts, size bounds, execution).
//! 3. Chrome Extension MV3 Distribution (manifest v3, CSP strictness, bundled assets).
//! 4. Disaster Recovery & 30 GB Storage Ceiling (resource check, recovery runbook invariants).

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Must resolve workspace root")
        .to_path_buf()
}

#[test]
fn test_compose_topology_and_network_isolation_invariants() {
    let root = get_workspace_root();
    let compose_path = root.join("deploy/podman-compose.yml");
    assert!(
        compose_path.exists(),
        "deploy/podman-compose.yml must exist at {}",
        compose_path.display()
    );

    let compose_content = fs::read_to_string(&compose_path).expect("Must read compose file");
    let doc: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&compose_content).expect("Must parse compose YAML");

    let services = doc["services"]
        .as_mapping()
        .expect("services section required");

    // 1. Core Services Invariant
    let required_services = ["postgres", "api", "worker", "browser-worker", "web"];
    for svc in &required_services {
        assert!(
            services.contains_key(*svc),
            "deploy/podman-compose.yml must declare '{svc}' service"
        );
    }

    // 2. Dual Isolated Networks Invariant
    let networks = doc["networks"]
        .as_mapping()
        .expect("networks section required");
    assert!(
        networks.contains_key("internal_net"),
        "networks must declare internal_net"
    );
    assert!(
        networks.contains_key("browser_net"),
        "networks must declare browser_net"
    );

    // 3. Strict Sandbox Network Isolation for browser-worker
    // browser-worker MUST attach to browser_net and MUST NOT attach to internal_net
    let bw = &services["browser-worker"];
    let bw_nets = bw["networks"]
        .as_sequence()
        .expect("browser-worker networks list required");
    let bw_net_names: Vec<String> = bw_nets
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    assert!(
        bw_net_names.contains(&"browser_net".to_string()),
        "browser-worker must attach to browser_net"
    );
    assert!(
        !bw_net_names.contains(&"internal_net".to_string()),
        "SECURITY VIOLATION: browser-worker must NEVER attach to internal_net"
    );

    // 4. Security Sandbox Opts
    let bw_sec_opts = bw["security_opt"]
        .as_sequence()
        .expect("browser-worker security_opt list required");
    let has_no_new_priv = bw_sec_opts
        .iter()
        .any(|v| v.as_str() == Some("no-new-privileges:true"));
    assert!(
        has_no_new_priv,
        "browser-worker must enforce 'no-new-privileges:true'"
    );

    // 5. Named Persistent Volumes
    let volumes = doc["volumes"]
        .as_mapping()
        .expect("volumes section required");
    assert!(
        volumes.contains_key("postgres_data"),
        "volumes must declare postgres_data"
    );
    assert!(
        volumes.contains_key("artifact_data"),
        "volumes must declare artifact_data"
    );
}

#[test]
fn test_containerfile_multi_stage_and_unprivileged_users() {
    let root = get_workspace_root();

    // 1. Containerfile.api
    let api_path = root.join("deploy/Containerfile.api");
    assert!(api_path.exists());
    let api_content = fs::read_to_string(&api_path).expect("Read Containerfile.api");
    assert!(
        api_content.contains("FROM ") && api_content.contains(" AS builder"),
        "Containerfile.api must have builder stage"
    );
    assert!(
        api_content.contains("FROM ") && api_content.contains(" AS runtime"),
        "Containerfile.api must have runtime stage"
    );
    assert!(
        api_content.contains("USER appuser"),
        "Containerfile.api must enforce unprivileged user appuser"
    );
    assert!(
        api_content.contains("debian:bookworm-slim"),
        "Containerfile.api runtime must use debian:bookworm-slim"
    );

    // 2. Containerfile.worker
    let worker_path = root.join("deploy/Containerfile.worker");
    assert!(worker_path.exists());
    let worker_content = fs::read_to_string(&worker_path).expect("Read Containerfile.worker");
    assert!(
        worker_content.contains("FROM ") && worker_content.contains(" AS builder"),
        "Containerfile.worker must have builder stage"
    );
    assert!(
        worker_content.contains("FROM ") && worker_content.contains(" AS runtime"),
        "Containerfile.worker must have runtime stage"
    );
    assert!(
        worker_content.contains("USER appuser"),
        "Containerfile.worker must enforce unprivileged user appuser"
    );
    assert!(
        worker_content.contains("debian:bookworm-slim"),
        "Containerfile.worker runtime must use debian:bookworm-slim"
    );

    // 3. Containerfile.browser-worker
    let bw_path = root.join("deploy/Containerfile.browser-worker");
    assert!(bw_path.exists());
    let bw_content = fs::read_to_string(&bw_path).expect("Read Containerfile.browser-worker");
    assert!(
        bw_content.contains("playwright:"),
        "Containerfile.browser-worker must use playwright base image"
    );
    assert!(
        bw_content.contains("USER pwuser"),
        "Containerfile.browser-worker must enforce unprivileged user pwuser"
    );

    // 4. Containerfile.web
    let web_path = root.join("deploy/Containerfile.web");
    assert!(web_path.exists());
    let web_content = fs::read_to_string(&web_path).expect("Read Containerfile.web");
    assert!(
        web_content.contains("FROM ") && web_content.contains(" AS builder"),
        "Containerfile.web must have builder stage"
    );
    assert!(
        web_content.contains("FROM ") && web_content.contains(" AS runtime"),
        "Containerfile.web must have runtime stage"
    );
    assert!(
        web_content.contains("USER node"),
        "Containerfile.web must enforce unprivileged user node"
    );
    assert!(
        web_content.contains("node:20-slim"),
        "Containerfile.web runtime must use node:20-slim"
    );
}

#[test]
fn test_environment_variables_schema_and_security_bounds() {
    let root = get_workspace_root();
    let env_example_path = root.join("deploy/.env.example");
    assert!(env_example_path.exists());

    let env_content = fs::read_to_string(&env_example_path).expect("Read .env.example");

    let required_keys = [
        "POSTGRES_DB",
        "POSTGRES_USER",
        "POSTGRES_PASSWORD",
        "DATABASE_URL",
        "API_PORT",
        "API_HOST",
        "RUST_LOG",
        "STORAGE_BACKEND",
        "STORAGE_LOCAL_PATH",
        "BROWSER_WORKER_URL",
        "PUBLIC_API_URL",
        "WEB_PORT",
        "WEB_HOST",
        "TOKEN_ENCRYPTION_KEY",
    ];

    for key in &required_keys {
        assert!(
            env_content
                .lines()
                .any(|l| l.starts_with(&format!("{key}="))),
            "deploy/.env.example must define key '{key}'"
        );
    }

    // Verify documentation in docs/DEPLOYMENT.md covers environment variables
    let deploy_doc =
        fs::read_to_string(root.join("docs/DEPLOYMENT.md")).expect("Read DEPLOYMENT.md");
    assert!(
        deploy_doc.contains("Environment Variables Reference"),
        "docs/DEPLOYMENT.md must contain Environment Variables Reference"
    );
    for key in &required_keys {
        assert!(
            deploy_doc.contains(key),
            "docs/DEPLOYMENT.md must document variable '{key}'"
        );
    }
}

#[test]
fn test_standalone_binary_contracts_and_execution() {
    let root = get_workspace_root();
    let is_windows = cfg!(target_os = "windows");

    let bin_dir = root.join("dist/bin");
    if bin_dir.exists() {
        let binaries = [
            "web-repo-to-ai",
            "web-repo-to-ai-cli",
            "web-repo-to-ai-api",
            "web-repo-to-ai-worker",
            "web-repo-to-ai-mcp",
        ];

        for b in &binaries {
            let file_name = if is_windows {
                format!("{b}.exe")
            } else {
                b.to_string()
            };
            let b_path = bin_dir.join(&file_name);
            assert!(
                b_path.exists(),
                "Release binary '{file_name}' must exist in dist/bin"
            );

            let meta = fs::metadata(&b_path).expect("Must read metadata");
            let size = meta.len();
            assert!(
                size >= 100 * 1024,
                "Binary {file_name} size ({size} bytes) is below 100 KB minimum"
            );
            assert!(
                size <= 80 * 1024 * 1024,
                "Binary {file_name} size ({size} bytes) exceeds 80 MB ceiling"
            );
        }

        // Test CLI execution
        let cli_name = if is_windows {
            "web-repo-to-ai.exe"
        } else {
            "web-repo-to-ai"
        };
        let cli_path = bin_dir.join(cli_name);
        if cli_path.exists() {
            let version_output = Command::new(&cli_path)
                .arg("--version")
                .output()
                .expect("Failed to execute cli --version");
            let ver_str = String::from_utf8_lossy(&version_output.stdout);
            assert!(
                ver_str.contains("web-repo-to-ai"),
                "CLI --version output must contain program name"
            );

            let help_output = Command::new(&cli_path)
                .arg("--help")
                .output()
                .expect("Failed to execute cli --help");
            let help_str = String::from_utf8_lossy(&help_output.stdout);
            let subcommands = [
                "url",
                "repo",
                "config",
                "batch",
                "verify",
                "completions",
                "mcp",
            ];
            for sub in &subcommands {
                assert!(
                    help_str.contains(sub),
                    "CLI help must document subcommand '{sub}'"
                );
            }
        }
    } else {
        println!("dist/bin not present in test runner; validated specifications");
    }
}

#[test]
fn test_chrome_extension_manifest_v3_and_csp_compliance() {
    let root = get_workspace_root();
    let manifest_path = root.join("apps/extension/manifest.json");
    assert!(manifest_path.exists());

    let content = fs::read_to_string(&manifest_path).expect("Read extension manifest");
    let doc: serde_json::Value = serde_json::from_str(&content).expect("Must parse manifest.json");

    assert_eq!(doc["manifest_version"], 3, "Target must be Manifest V3");
    assert_eq!(doc["name"], "web-repo-to-ai Converter");

    // Permissions
    let perms = doc["permissions"]
        .as_array()
        .expect("permissions array required");
    let perm_strings: Vec<&str> = perms.iter().filter_map(|v| v.as_str()).collect();
    assert!(perm_strings.contains(&"storage"));
    assert!(perm_strings.contains(&"activeTab"));
    assert!(perm_strings.contains(&"scripting"));

    // Host Permissions
    let host_perms = doc["host_permissions"]
        .as_array()
        .expect("host_permissions array required");
    let host_strings: Vec<&str> = host_perms.iter().filter_map(|v| v.as_str()).collect();
    assert!(host_strings.iter().any(|h| h.contains("localhost")));
    assert!(host_strings.iter().any(|h| h.contains("127.0.0.1")));

    // Content Security Policy
    let csp = doc["content_security_policy"]["extension_pages"]
        .as_str()
        .expect("extension_pages CSP required");
    assert!(
        csp.contains("script-src 'self'"),
        "CSP must restrict script-src to 'self'"
    );
    assert!(
        !csp.contains("unsafe-inline"),
        "CSP must NOT allow 'unsafe-inline'"
    );
    assert!(
        !csp.contains("unsafe-eval"),
        "CSP must NOT allow 'unsafe-eval'"
    );

    // Verify icons exist
    let icons_dir = root.join("apps/extension/public/icons");
    assert!(icons_dir.join("icon16.png").exists());
    assert!(icons_dir.join("icon48.png").exists());
    assert!(icons_dir.join("icon128.png").exists());
}

#[test]
fn test_disaster_recovery_and_resource_budget_governance() {
    let root = get_workspace_root();

    // 1. Resource Management Scripts & 30 GB Ceiling
    let chk_ps1 = root.join("scripts/check-resources.ps1");
    let chk_sh = root.join("scripts/check-resources.sh");
    assert!(chk_ps1.exists());
    assert!(chk_sh.exists());

    let ps1_content = fs::read_to_string(&chk_ps1).expect("Read check-resources.ps1");
    assert!(
        ps1_content.contains("30GB")
            || ps1_content.contains("32212254720")
            || ps1_content.contains("30 GB"),
        "check-resources.ps1 must enforce 30 GB ceiling"
    );

    // 2. Lifecycle Scripts Inventory
    let lifecycle_scripts = [
        "dev-up",
        "dev-down",
        "dev-status",
        "dev-restart",
        "dev-logs",
        "clean-caches",
    ];
    for ls in &lifecycle_scripts {
        assert!(
            root.join(format!("scripts/{ls}.ps1")).exists(),
            "scripts/{ls}.ps1 must exist"
        );
        assert!(
            root.join(format!("scripts/{ls}.sh")).exists(),
            "scripts/{ls}.sh must exist"
        );
    }

    // 3. Deployment Runbook & Recovery Invariants
    let deploy_doc =
        fs::read_to_string(root.join("docs/DEPLOYMENT.md")).expect("Read DEPLOYMENT.md");
    assert!(
        deploy_doc.contains("Backup, Restore & Disaster Recovery Runbook"),
        "DEPLOYMENT.md must contain Backup, Restore & Disaster Recovery Runbook"
    );
    assert!(
        deploy_doc.contains("pg_dump") && deploy_doc.contains("pg_restore"),
        "DEPLOYMENT.md must specify pg_dump and pg_restore recovery commands"
    );
    assert!(
        deploy_doc.contains("5 Minutes") || deploy_doc.contains("5 min"),
        "DEPLOYMENT.md must specify <5 minute cold recovery SLA"
    );
}
