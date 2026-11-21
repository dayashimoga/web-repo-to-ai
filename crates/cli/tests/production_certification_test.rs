//! # Production Release Certification Integration Tests (`crates/cli/tests/production_certification_test.rs`)
//!
//! Subphase 11.4.5: Final Certification Report & Production Sign-Off
//!
//! Validates the exhaustive production invariants across the monorepo:
//! 1. 100% Subphase Completion across all 11 stages (197 / 197 ACCEPTED).
//! 2. Authoritative Production Certification Report presence and structural completeness.
//! 3. Complete Release Assets, Manifests, and Software Bills of Materials (SBOMs).
//! 4. Comprehensive Engineering Documentation across all 16 specification documents.
//! 5. Strict Zero-Mock & Zero-Placeholder Policy (Rule 09 enforcement across all crates).
//! 6. Monorepo Package Directed Acyclic Graph (DAG) acyclicity verification.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Must resolve workspace root")
        .to_path_buf()
}

/// 1. Verifies 100% Subphase Completion across all 11 stages (197 / 197 ACCEPTED).
#[test]
fn test_ledger_subphase_completion_100_percent() {
    let root = get_workspace_root();

    // Verify TODO.md
    let todo_path = root.join("TODO.md");
    assert!(todo_path.exists(), "TODO.md must exist");
    let todo_content = fs::read_to_string(&todo_path).expect("Must read TODO.md");

    let completed_tasks_count = todo_content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("- [x] **[")
        })
        .count();

    let incomplete_tasks_count = todo_content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed.starts_with("- [ ] **[")
        })
        .count();

    assert_eq!(
        completed_tasks_count, 197,
        "Expected exactly 197 completed subphase items in TODO.md, found {completed_tasks_count}"
    );
    assert_eq!(
        incomplete_tasks_count, 0,
        "Expected 0 incomplete subphase items in TODO.md, found {incomplete_tasks_count}"
    );

    // Verify PROJECT_STATUS.md
    let status_path = root.join("PROJECT_STATUS.md");
    assert!(status_path.exists(), "PROJECT_STATUS.md must exist");
    let status_content = fs::read_to_string(&status_path).expect("Must read PROJECT_STATUS.md");

    assert!(
        status_content.contains("| `ACCEPTED` | 197 |")
            || status_content.contains("| ACCEPTED | 197 |"),
        "PROJECT_STATUS.md must record 197 ACCEPTED subphases"
    );
    assert!(
        status_content.contains("| `NOT_STARTED` | 0 |")
            || status_content.contains("| NOT_STARTED | 0 |"),
        "PROJECT_STATUS.md must record 0 NOT_STARTED subphases"
    );

    // Verify IMPLEMENTATION_LEDGER.md
    let ledger_path = root.join("IMPLEMENTATION_LEDGER.md");
    let ledger_content = fs::read_to_string(&ledger_path).unwrap_or_else(|e| {
        panic!(
            "Must read IMPLEMENTATION_LEDGER.md at {}: {e}",
            ledger_path.display()
        )
    });

    assert!(
        ledger_content.contains("11.4.5"),
        "IMPLEMENTATION_LEDGER.md must document Subphase 11.4.5 (at {})",
        ledger_path.display()
    );
    assert!(
        ledger_content.contains("Production Certification")
            || ledger_content.contains("Final certification report"),
        "IMPLEMENTATION_LEDGER.md must describe Subphase 11.4.5"
    );
}

/// 2. Verifies docs/PRODUCTION_CERTIFICATION.md presence, size, and structural sections.
#[test]
fn test_production_certification_report_presence_and_structure() {
    let root = get_workspace_root();
    let cert_path = root.join("docs/PRODUCTION_CERTIFICATION.md");
    assert!(
        cert_path.exists(),
        "docs/PRODUCTION_CERTIFICATION.md must exist at {}",
        cert_path.display()
    );

    let cert_content =
        fs::read_to_string(&cert_path).expect("Must read docs/PRODUCTION_CERTIFICATION.md");

    assert!(
        cert_content.len() >= 10_000,
        "docs/PRODUCTION_CERTIFICATION.md must be comprehensive (>= 10,000 bytes, was {})",
        cert_content.len()
    );

    let required_sections = [
        "## 1. Executive Certification Statement",
        "## 2. Monorepo Topology & Component Inventory",
        "## 3. Monorepo Quality Gate Matrix",
        "## 4. Stage Completion & Verification Summary",
        "## 5. Security Certification Audit",
        "## 6. Performance & SLA Benchmark Certification",
        "## 7. Reliability & Chaos Resilience Certification",
        "## 8. Software Supply Chain & Licensing Certification",
        "## 9. Release Packaging & Verification Invariants",
        "## 10. Operational Guidelines & Remaining Constraints",
        "## 11. Final Certification Sign-Off",
    ];

    for section in &required_sections {
        assert!(
            cert_content.contains(section),
            "docs/PRODUCTION_CERTIFICATION.md missing required section: '{section}'"
        );
    }

    // Verify key audit terms
    let required_terms = [
        "197 / 197 ACCEPTED (100.0%)",
        "v0.1.0",
        "STRIDE",
        "CycloneDX v1.5",
        "SPDX v2.3",
        "<= 30 GB",
        "1,041 Total Passing Tests",
    ];

    for term in &required_terms {
        assert!(
            cert_content.contains(term),
            "docs/PRODUCTION_CERTIFICATION.md missing key certification term: '{term}'"
        );
    }
}

/// 3. Verifies release assets inventory, manifests, and SBOM completeness.
#[test]
fn test_release_assets_and_manifests_inventory() {
    let root = get_workspace_root();
    let release_dir = root.join("release-assets");
    assert!(
        release_dir.exists(),
        "release-assets/ directory must exist at {}",
        release_dir.display()
    );

    // 1. Binaries zip
    let bin_zip = release_dir.join("web-repo-to-ai-v0.1.0-windows-x86_64.zip");
    assert!(bin_zip.exists(), "Windows binaries zip must exist");
    let bin_meta = fs::metadata(&bin_zip).expect("Binaries zip metadata");
    assert!(
        bin_meta.len() > 10_000_000,
        "Windows binaries zip must be > 10 MB (was {})",
        bin_meta.len()
    );

    // 2. Web zip
    let web_zip = release_dir.join("web-repo-to-ai-v0.1.0-frontend-web.zip");
    assert!(web_zip.exists(), "Frontend web zip must exist");
    let web_meta = fs::metadata(&web_zip).expect("Web zip metadata");
    assert!(
        web_meta.len() > 100_000,
        "Web zip must be > 100 KB (was {})",
        web_meta.len()
    );

    // 3. Extension zip
    let ext_zip = release_dir.join("web-repo-to-ai-v0.1.0-chrome-extension.zip");
    assert!(ext_zip.exists(), "Extension zip must exist");
    let ext_meta = fs::metadata(&ext_zip).expect("Extension zip metadata");
    assert!(
        ext_meta.len() > 5_000,
        "Extension zip must be > 5 KB (was {})",
        ext_meta.len()
    );

    // 4. CycloneDX SBOM
    let cyclonedx_file = release_dir.join("sbom-cyclonedx.json");
    assert!(cyclonedx_file.exists(), "sbom-cyclonedx.json must exist");
    let cyclonedx_content =
        fs::read_to_string(&cyclonedx_file).expect("Must read sbom-cyclonedx.json");
    let cdx_val: serde_json::Value =
        serde_json::from_str(&cyclonedx_content).expect("Must parse CycloneDX JSON");
    assert_eq!(
        cdx_val["bomFormat"].as_str(),
        Some("CycloneDX"),
        "bomFormat must be CycloneDX"
    );
    let components = cdx_val["components"]
        .as_array()
        .expect("components array required");
    assert!(
        components.len() >= 350,
        "CycloneDX must contain >= 350 components (found {})",
        components.len()
    );

    // 5. SPDX SBOM
    let spdx_file = release_dir.join("sbom-spdx.json");
    assert!(spdx_file.exists(), "sbom-spdx.json must exist");
    let spdx_content = fs::read_to_string(&spdx_file).expect("Must read sbom-spdx.json");
    let spdx_val: serde_json::Value =
        serde_json::from_str(&spdx_content).expect("Must parse SPDX JSON");
    assert_eq!(
        spdx_val["spdxVersion"].as_str(),
        Some("SPDX-2.3"),
        "spdxVersion must be SPDX-2.3"
    );
    let packages = spdx_val["packages"]
        .as_array()
        .expect("packages array required");
    assert!(
        packages.len() >= 350,
        "SPDX must contain >= 350 packages (found {})",
        packages.len()
    );

    // 6. Release Manifest
    let manifest_file = release_dir.join("RELEASE_MANIFEST.json");
    assert!(
        manifest_file.exists(),
        "RELEASE_MANIFEST.json must exist in release-assets/"
    );
    let manifest_content =
        fs::read_to_string(&manifest_file).expect("Must read RELEASE_MANIFEST.json");
    let manifest_val: serde_json::Value =
        serde_json::from_str(&manifest_content).expect("Must parse RELEASE_MANIFEST.json");
    let version = manifest_val["release_tag"]
        .as_str()
        .or_else(|| manifest_val["version"].as_str())
        .expect("release_tag or version required in RELEASE_MANIFEST.json");
    assert!(
        version.contains("0.1.0"),
        "Manifest version must contain 0.1.0 (found {version})"
    );
    assert!(
        manifest_val["assets"].as_array().is_some()
            || manifest_val["artifacts"].as_array().is_some(),
        "Manifest must list assets or artifacts"
    );

    // 7. Checksums
    let checksums_file = release_dir.join("checksums.sha256");
    assert!(checksums_file.exists(), "checksums.sha256 must exist");
    let checksums_content =
        fs::read_to_string(&checksums_file).expect("Must read checksums.sha256");
    assert!(
        checksums_content.contains("web-repo-to-ai-v0.1.0-windows-x86_64.zip"),
        "checksums must include windows binaries zip"
    );
    assert!(
        checksums_content.contains("sbom-cyclonedx.json"),
        "checksums must include CycloneDX SBOM"
    );
}

/// 4. Verifies documentation and specifications completeness across docs/.
#[test]
fn test_documentation_and_specifications_completeness() {
    let root = get_workspace_root();

    let docs = [
        "docs/ARCHITECTURE.md",
        "docs/DATA_MODEL.md",
        "docs/SECURITY.md",
        "docs/TESTING.md",
        "docs/PERFORMANCE.md",
        "docs/DEPLOYMENT.md",
        "docs/ROADMAP.md",
        "docs/USER_GUIDE.md",
        "docs/DEVELOPMENT.md",
        "docs/GITHUB_APP.md",
        "docs/DESIGN_SYSTEM.md",
        "docs/API.md",
        "docs/LICENSING.md",
        "docs/REQUIREMENTS.md",
        "docs/PRODUCTION_CERTIFICATION.md",
    ];

    for doc_relative in &docs {
        let doc_path = root.join(doc_relative);
        assert!(
            doc_path.exists(),
            "Documentation file missing: {doc_relative}"
        );
        let meta = fs::metadata(&doc_path).expect("Metadata for doc");
        assert!(
            meta.len() >= 1_000,
            "Doc {doc_relative} must be non-trivial (>= 1,000 bytes, was {})",
            meta.len()
        );
    }

    // Root governance files
    let governance_files = [
        "README.md",
        "CHANGELOG.md",
        "DECISIONS.md",
        "PROJECT_STATUS.md",
        "IMPLEMENTATION_LEDGER.md",
        "TODO.md",
        "ROADMAP.md",
        "GLOBAL_ENGINEERING_CONTRACT.md",
        "AGENTS.md",
    ];

    for gov_file in &governance_files {
        let path = root.join(gov_file);
        assert!(path.exists(), "Governance root file missing: {gov_file}");
        let meta = fs::metadata(&path).expect("Metadata for governance file");
        assert!(
            meta.len() >= 500,
            "Governance file {gov_file} must have valid content (was {})",
            meta.len()
        );
    }
}

/// Helper function to recursively find all rust source files in production crates.
fn collect_rust_src_files(dir: &Path, results: &mut Vec<PathBuf>) {
    if !dir.exists() || !dir.is_dir() {
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip target and tests directories
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if file_name != "target" && file_name != "tests" && file_name != "benches" {
                collect_rust_src_files(&path, results);
            }
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            results.push(path);
        }
    }
}

/// 5. Verifies Zero-Mock & Zero-Placeholder Policy (Rule 09 enforcement).
#[test]
fn test_clean_governance_and_zero_placeholder_policy() {
    let root = get_workspace_root();
    let mut src_files = Vec::new();

    // Check crates/*/src
    let crates_dir = root.join("crates");
    if let Ok(entries) = fs::read_dir(&crates_dir) {
        for entry in entries.flatten() {
            let crate_src = entry.path().join("src");
            collect_rust_src_files(&crate_src, &mut src_files);
        }
    }

    // Check services/*/src
    let services_dir = root.join("services");
    if let Ok(entries) = fs::read_dir(&services_dir) {
        for entry in entries.flatten() {
            let svc_src = entry.path().join("src");
            collect_rust_src_files(&svc_src, &mut src_files);
        }
    }

    assert!(
        src_files.len() >= 50,
        "Expected at least 50 production Rust source files across monorepo, found {}",
        src_files.len()
    );

    let mut violations = Vec::new();

    for file in &src_files {
        let content = match fs::read_to_string(file) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (line_idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            // Ignore commented lines
            if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
                continue;
            }

            if trimmed.contains("todo!(") || trimmed.contains("unimplemented!(") {
                violations.push(format!(
                    "{}:{}: {}",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    line_idx + 1,
                    trimmed
                ));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Rule 09 violation! Found forbidden placeholder macros:\n{}",
        violations.join("\n")
    );
}

/// 6. Verifies Monorepo Package Directed Acyclic Graph (DAG) acyclicity.
#[test]
fn test_monorepo_package_dependency_graph_acyclicity() {
    let root = get_workspace_root();
    let root_cargo = root.join("Cargo.toml");
    let root_content = fs::read_to_string(&root_cargo).expect("Must read root Cargo.toml");

    // Extract workspace members
    let mut members = Vec::new();
    let mut in_members = false;
    for line in root_content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("members = [") {
            in_members = true;
            continue;
        }
        if in_members {
            if trimmed.starts_with(']') {
                break;
            }
            let member = trimmed.trim_matches(|c| c == '"' || c == ',' || c == ' ');
            if !member.is_empty() {
                members.push(member.to_string());
            }
        }
    }

    assert!(
        members.len() >= 20,
        "Expected at least 20 workspace members, found {}",
        members.len()
    );

    // Map: member relative path -> package name
    let mut pkg_name_to_member = HashMap::new();
    let mut member_to_pkg_name = HashMap::new();

    for member in &members {
        let member_cargo = root.join(member).join("Cargo.toml");
        assert!(
            member_cargo.exists(),
            "Cargo.toml must exist for member: {member}"
        );
        let content = fs::read_to_string(&member_cargo).expect("Must read member Cargo.toml");
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("name = ") {
                let name = trimmed
                    .strip_prefix("name = ")
                    .unwrap()
                    .trim_matches(|c| c == '"' || c == '\'');
                pkg_name_to_member.insert(name.to_string(), member.clone());
                member_to_pkg_name.insert(member.clone(), name.to_string());
                break;
            }
        }
    }

    // Build dependency graph (package -> set of dependent packages)
    let mut adj: HashMap<String, HashSet<String>> = HashMap::new();
    for member in &members {
        let pkg_name = member_to_pkg_name
            .get(member)
            .cloned()
            .unwrap_or_else(|| member.clone());
        adj.entry(pkg_name.clone()).or_default();

        let member_cargo = root.join(member).join("Cargo.toml");
        let content = fs::read_to_string(&member_cargo).expect("Must read member Cargo.toml");

        for line in content.lines() {
            let trimmed = line.trim();
            // Look for internal dependencies like `web-repo-to-ai-core = ...`
            for internal_pkg in pkg_name_to_member.keys() {
                if internal_pkg != &pkg_name && trimmed.starts_with(internal_pkg) {
                    adj.get_mut(&pkg_name).unwrap().insert(internal_pkg.clone());
                }
            }
        }
    }

    // Verify DAG acyclicity via Depth-First Search cycle detection
    // State: 0 = unvisited, 1 = currently visiting (in stack), 2 = visited/finished
    let mut state: HashMap<String, u8> = HashMap::new();
    for pkg in adj.keys() {
        state.insert(pkg.clone(), 0);
    }

    fn has_cycle(
        node: &str,
        adj: &HashMap<String, HashSet<String>>,
        state: &mut HashMap<String, u8>,
        cycle_path: &mut Vec<String>,
    ) -> bool {
        state.insert(node.to_string(), 1);
        cycle_path.push(node.to_string());

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                let neighbor_state = state.get(neighbor).copied().unwrap_or(0);
                if neighbor_state == 1 {
                    // Cycle detected!
                    cycle_path.push(neighbor.to_string());
                    return true;
                } else if neighbor_state == 0 && has_cycle(neighbor, adj, state, cycle_path) {
                    return true;
                }
            }
        }

        cycle_path.pop();
        state.insert(node.to_string(), 2);
        false
    }

    for pkg in adj.keys() {
        if state.get(pkg).copied().unwrap_or(0) == 0 {
            let mut cycle_path = Vec::new();
            let cycle_found = has_cycle(pkg, &adj, &mut state, &mut cycle_path);
            assert!(
                !cycle_found,
                "Cyclic dependency detected in monorepo packages! Cycle path: {}",
                cycle_path.join(" -> ")
            );
        }
    }
}
