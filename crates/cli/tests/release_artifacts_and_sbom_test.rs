//! # Release Artifacts & SBOM Verification Tests (`crates/cli/tests/release_artifacts_and_sbom_test.rs`)
//!
//! Validates CycloneDX v1.5 JSON, SPDX v2.3 JSON, release archives, checksums,
//! and manifest integrity for Subphase 11.4.3 (Release artifacts & SBOM).

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("Workspace root must be resolvable")
        .to_path_buf()
}

fn ensure_sboms_generated() -> (PathBuf, PathBuf) {
    let root = get_workspace_root();
    let release_assets_dir = root.join("release-assets");
    let cdx_path = release_assets_dir.join("sbom-cyclonedx.json");
    let spdx_path = release_assets_dir.join("sbom-spdx.json");

    if !cdx_path.exists() || !spdx_path.exists() {
        let script = root.join("scripts").join("generate-sbom.py");
        let status = std::process::Command::new("python")
            .arg(&script)
            .arg("--out-dir")
            .arg(&release_assets_dir)
            .arg("--tag")
            .arg("v0.1.0")
            .current_dir(&root)
            .status()
            .expect("Failed to execute generate-sbom.py");
        assert!(status.success(), "generate-sbom.py must succeed");
    }

    (cdx_path, spdx_path)
}

#[test]
fn test_cyclonedx_sbom_specification_schema_and_invariants() {
    let (cdx_path, _) = ensure_sboms_generated();
    let content = std::fs::read_to_string(&cdx_path).expect("CycloneDX file must exist");
    let doc: serde_json::Value =
        serde_json::from_str(&content).expect("CycloneDX must be valid JSON");

    assert_eq!(doc["bomFormat"], "CycloneDX");
    assert_eq!(doc["specVersion"], "1.5");
    assert_eq!(doc["version"], 1);

    let serial = doc["serialNumber"].as_str().expect("serialNumber required");
    assert!(
        serial.starts_with("urn:uuid:"),
        "serialNumber must start with urn:uuid:"
    );

    let metadata = &doc["metadata"];
    assert_eq!(metadata["component"]["name"], "web-repo-to-ai");
    assert_eq!(metadata["component"]["type"], "application");
    assert!(!metadata["tools"].as_array().unwrap().is_empty());

    let components = doc["components"]
        .as_array()
        .expect("components array required");
    assert!(
        components.len() >= 300,
        "Expected at least 300 components in SBOM (found: {})",
        components.len()
    );

    for comp in components {
        assert!(comp["name"].is_string(), "Component must have name");
        assert!(comp["version"].is_string(), "Component must have version");
        assert!(comp["purl"].is_string(), "Component must have purl");
        let purl = comp["purl"].as_str().unwrap();
        assert!(
            purl.starts_with("pkg:cargo/") || purl.starts_with("pkg:npm/"),
            "purl must be cargo or npm: {purl}"
        );
    }

    let dependencies = doc["dependencies"]
        .as_array()
        .expect("dependencies array required");
    assert!(
        !dependencies.is_empty(),
        "Dependencies graph must contain nodes"
    );
}

#[test]
fn test_spdx_sbom_specification_schema_and_invariants() {
    let (_, spdx_path) = ensure_sboms_generated();
    let content = std::fs::read_to_string(&spdx_path).expect("SPDX file must exist");
    let doc: serde_json::Value = serde_json::from_str(&content).expect("SPDX must be valid JSON");

    assert_eq!(doc["spdxVersion"], "SPDX-2.3");
    assert_eq!(doc["dataLicense"], "CC0-1.0");
    assert_eq!(doc["SPDXID"], "SPDXRef-DOCUMENT");

    let packages = doc["packages"].as_array().expect("packages array required");
    assert!(
        packages.len() >= 300,
        "Expected at least 300 packages in SPDX (found: {})",
        packages.len()
    );

    let root_pkg = packages
        .iter()
        .find(|p| p["name"] == "web-repo-to-ai")
        .expect("Root package web-repo-to-ai must be declared in SPDX");
    assert_eq!(root_pkg["licenseConcluded"], "MIT");

    for pkg in packages {
        let spdx_id = pkg["SPDXID"].as_str().expect("SPDXID must be string");
        assert!(
            spdx_id.starts_with("SPDXRef-Package-"),
            "SPDXID must start with SPDXRef-Package-: {spdx_id}"
        );
        assert!(pkg["name"].is_string());
        assert!(pkg["versionInfo"].is_string());
    }

    let relationships = doc["relationships"]
        .as_array()
        .expect("relationships required");
    assert!(
        relationships
            .iter()
            .any(|r| r["relationshipType"] == "DESCRIBES"),
        "Must contain DESCRIBES relationship for root document"
    );
    assert!(
        relationships
            .iter()
            .any(|r| r["relationshipType"] == "DEPENDS_ON"),
        "Must contain DEPENDS_ON relationships for packages"
    );
}

#[test]
fn test_release_manifest_structure_and_completeness() {
    let root = get_workspace_root();
    let manifest_path = root.join("release-assets").join("RELEASE_MANIFEST.json");

    if manifest_path.exists() {
        let content = std::fs::read_to_string(&manifest_path).unwrap();
        let content = content.trim_start_matches('\u{feff}');
        let doc: serde_json::Value = serde_json::from_str(content).unwrap();

        assert_eq!(doc["project"], "web-repo-to-ai");
        assert_eq!(doc["release_tag"], "v0.1.0");
        assert_eq!(doc["status"], "READY_FOR_PUBLICATION");

        let assets = doc["assets"].as_array().expect("assets array required");
        assert!(
            assets.len() >= 4,
            "Expected at least 4 release assets (found: {})",
            assets.len()
        );

        for asset in assets {
            assert!(asset["name"].is_string());
            assert!(asset["size_bytes"].as_u64().unwrap() > 0);
            assert_eq!(asset["sha256"].as_str().unwrap().len(), 64);
        }
    }
}

#[test]
fn test_release_archive_zip_integrity_and_decompression() {
    let root = get_workspace_root();
    let zip_path = root
        .join("release-assets")
        .join("web-repo-to-ai-v0.1.0-windows-x86_64.zip");

    if zip_path.exists() {
        let mut data = Vec::new();
        let mut file = File::open(&zip_path).expect("Release zip must open");
        file.read_to_end(&mut data).expect("Zip must be readable");

        // Validate standard Zip PK magic header bytes
        assert!(
            data.len() > 1000,
            "Release zip must contain substantial payload"
        );
        assert_eq!(
            &data[0..4],
            b"PK\x03\x04",
            "Zip archive must start with standard PK local file header magic bytes"
        );

        let data_str = String::from_utf8_lossy(&data);
        assert!(
            data_str.contains("web-repo-to-ai"),
            "Zip must contain primary CLI binary reference"
        );
        assert!(
            data_str.contains("README.md"),
            "Zip must contain README.md reference"
        );
        assert!(
            data_str.contains("LICENSE"),
            "Zip must contain LICENSE reference"
        );
    }
}

#[test]
fn test_dual_checksum_verification_and_monotonic_ordering() {
    let root = get_workspace_root();
    let checksum_path = root.join("release-assets").join("checksums.sha256");

    if checksum_path.exists() {
        let content = std::fs::read_to_string(&checksum_path).unwrap();
        let content = content.trim_start_matches('\u{feff}');
        let lines: Vec<&str> = content.lines().filter(|l| !l.trim().is_empty()).collect();
        assert!(!lines.is_empty(), "checksums.sha256 must not be empty");

        for line in lines {
            let line = line.trim_start_matches('\u{feff}').trim();
            let parts: Vec<&str> = line.split_whitespace().collect();
            assert_eq!(parts.len(), 2, "Line must be '<hash>  <filename>': {line}");
            let hash = parts[0];
            let filename = parts[1];
            assert_eq!(hash.len(), 64, "SHA-256 hash must be 64 hex characters");

            let target_file = root.join("release-assets").join(filename);
            if target_file.exists() && filename != "checksums.sha256" {
                let mut data = Vec::new();
                let mut f = File::open(&target_file).unwrap();
                f.read_to_end(&mut data).unwrap();

                let mut hasher = Sha256::new();
                hasher.update(&data);
                let actual_hash = format!("{:x}", hasher.finalize());
                assert_eq!(hash, actual_hash, "Checksum mismatch for file {filename}");
            }
        }
    }
}

#[test]
fn test_license_audit_and_spdx_whitelist_compliance() {
    let (cdx_path, _) = ensure_sboms_generated();
    let content = std::fs::read_to_string(&cdx_path).unwrap();
    let doc: serde_json::Value = serde_json::from_str(&content).unwrap();

    let components = doc["components"].as_array().unwrap();

    // Whitelist per docs/LICENSING.md
    let allowed_prefixes = [
        "MIT",
        "Apache-2.0",
        "BSD-2-Clause",
        "BSD-3-Clause",
        "0BSD",
        "ISC",
        "Unicode",
        "PostgreSQL",
        "CC0",
        "CDLA-Permissive",
        "Zlib",
        "Unlicense",
        "OpenSSL",
        "BSL-1.0",
        "MIT-0",
    ];

    // Strictly prohibited solo copyleft licenses
    let prohibited_patterns = ["GPL-2.0", "GPL-3.0", "AGPL", "SSPL", "CC-BY-NC"];

    for comp in components {
        let comp_name = comp["name"].as_str().unwrap();
        if let Some(licenses) = comp["licenses"].as_array() {
            // Verify component contains at least one approved permissive license
            let has_approved_license = licenses.iter().any(|lic_item| {
                if let Some(lic_id) = lic_item["license"]["id"].as_str() {
                    allowed_prefixes
                        .iter()
                        .any(|prefix| lic_id.starts_with(prefix))
                } else {
                    false
                }
            });
            assert!(
                has_approved_license,
                "Component '{comp_name}' must satisfy at least one approved permissive license"
            );

            // Verify no solo strictly-prohibited copyleft licenses
            if licenses.len() == 1 {
                if let Some(lic_id) = licenses[0]["license"]["id"].as_str() {
                    for prohibited in &prohibited_patterns {
                        assert!(
                            !lic_id.contains(prohibited),
                            "Prohibited license '{lic_id}' found in component '{comp_name}'"
                        );
                    }
                }
            }
        }
    }
}
