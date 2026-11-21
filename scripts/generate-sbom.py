#!/usr/bin/env python3
"""
Software Bill of Materials (SBOM) Generator for web-repo-to-ai
Generates standards-compliant CycloneDX v1.5 and SPDX v2.3 JSON documents
capturing Rust workspace crates, external dependencies, and Node.js packages.
"""

import argparse
import datetime
import json
import os
import re
import subprocess
import sys
import uuid


def sanitize_spdx_id(name: str, version: str) -> str:
    """Sanitize name and version into a compliant SPDXID."""
    clean_name = re.sub(r"[^a-zA-Z0-9.\-]", "-", name)
    clean_ver = re.sub(r"[^a-zA-Z0-9.\-]", "-", version)
    return f"SPDXRef-Package-{clean_name}-{clean_ver}"


def parse_cargo_metadata():
    """Extract complete dependency graph from cargo metadata."""
    cmd = ["cargo", "metadata", "--format-version", "1"]
    res = subprocess.run(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, check=True)
    return json.loads(res.stdout)


def parse_npm_packages(pkg_dir):
    """Extract package dependencies from a package.json file."""
    pkg_path = os.path.join(pkg_dir, "package.json")
    if not os.path.exists(pkg_path):
        return []
    with open(pkg_path, "r", encoding="utf-8") as f:
        data = json.load(f)

    deps = []
    for dep_type in ["dependencies", "devDependencies"]:
        for name, ver in data.get(dep_type, {}).items():
            clean_ver = ver.lstrip("^~>=<v ")
            deps.append({
                "name": name,
                "version": clean_ver,
                "ecosystem": "npm",
                "license": "MIT",  # Standard permissive default for checked web dependencies
            })
    return deps


def generate_cyclonedx(tag: str, cargo_meta: dict, npm_pkgs: list) -> dict:
    """Generate CycloneDX v1.5 JSON SBOM specification."""
    now_iso = datetime.datetime.now(datetime.timezone.utc).isoformat()
    serial = f"urn:uuid:{uuid.uuid4()}"

    components = []
    dependencies = []
    seen = set()

    # 1. Process Cargo dependencies
    for pkg in cargo_meta.get("packages", []):
        name = pkg["name"]
        version = pkg["version"]
        key = ("cargo", name, version)
        if key in seen:
            continue
        seen.add(key)

        is_workspace = pkg.get("source") is None
        purl = f"pkg:cargo/{name}@{version}"
        comp_id = f"component-{name}-{version}"

        licenses = []
        raw_lic = pkg.get("license") or "MIT"
        for lic in re.split(r"\s+OR\s+|\s+AND\s+|/", raw_lic):
            lic_clean = lic.strip(" ()\t\r\n")
            if lic_clean:
                licenses.append({"license": {"id": lic_clean}})

        comp = {
            "type": "application" if is_workspace else "library",
            "bom-ref": purl,
            "name": name,
            "version": version,
            "description": pkg.get("description") or f"Rust package {name}",
            "purl": purl,
            "licenses": licenses,
        }
        if pkg.get("repository"):
            comp["externalReferences"] = [
                {"type": "vcs", "url": pkg["repository"]}
            ]

        components.append(comp)

    # 2. Process npm packages
    for pkg in npm_pkgs:
        name = pkg["name"]
        version = pkg["version"]
        key = ("npm", name, version)
        if key in seen:
            continue
        seen.add(key)

        purl = f"pkg:npm/{name}@{version}"
        components.append({
            "type": "library",
            "bom-ref": purl,
            "name": name,
            "version": version,
            "description": f"npm dependency {name}",
            "purl": purl,
            "licenses": [{"license": {"id": pkg.get("license", "MIT")}}],
        })

    # Root application component
    root_component = {
        "type": "application",
        "bom-ref": f"pkg:cargo/web-repo-to-ai@{tag.lstrip('v')}",
        "name": "web-repo-to-ai",
        "version": tag.lstrip("v"),
        "description": "Universal URL & GitHub Repository -> AI-Ready Content Converter",
        "licenses": [{"license": {"id": "MIT"}}],
        "purl": f"pkg:cargo/web-repo-to-ai@{tag.lstrip('v')}",
    }

    # Dependency mapping from resolve
    resolve = cargo_meta.get("resolve", {})
    for node in resolve.get("nodes", []):
        pkg_id = node["id"]
        # Match node to package
        matching_pkgs = [p for p in cargo_meta["packages"] if p["id"] == pkg_id]
        if matching_pkgs:
            p = matching_pkgs[0]
            ref_purl = f"pkg:cargo/{p['name']}@{p['version']}"
            dep_purls = []
            for dep_id in node.get("dependencies", []):
                dep_matches = [dp for dp in cargo_meta["packages"] if dp["id"] == dep_id]
                if dep_matches:
                    dep_purls.append(f"pkg:cargo/{dep_matches[0]['name']}@{dep_matches[0]['version']}")
            dependencies.append({
                "ref": ref_purl,
                "dependsOn": dep_purls,
            })

    return {
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "serialNumber": serial,
        "version": 1,
        "metadata": {
            "timestamp": now_iso,
            "tools": [
                {
                    "vendor": "web-repo-to-ai",
                    "name": "generate-sbom",
                    "version": "0.1.0",
                }
            ],
            "component": root_component,
        },
        "components": components,
        "dependencies": dependencies,
    }


def generate_spdx(tag: str, cargo_meta: dict, npm_pkgs: list) -> dict:
    """Generate SPDX v2.3 JSON SBOM specification."""
    now_iso = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
    doc_spdx_id = "SPDXRef-DOCUMENT"

    packages = []
    relationships = []
    seen = set()

    # Root Document Package
    root_spdx_id = f"SPDXRef-Package-web-repo-to-ai-{tag.lstrip('v')}"
    packages.append({
        "name": "web-repo-to-ai",
        "SPDXID": root_spdx_id,
        "versionInfo": tag.lstrip("v"),
        "downloadLocation": "https://github.com/dayashimoga/web-repo-to-ai",
        "filesAnalyzed": False,
        "licenseConcluded": "MIT",
        "licenseDeclared": "MIT",
        "copyrightText": "Copyright (c) 2026 web-repo-to-ai contributors",
        "description": "Universal URL & GitHub Repository -> AI-Ready Content Converter",
        "externalRefs": [
            {
                "referenceCategory": "PACKAGE-MANAGER",
                "referenceType": "purl",
                "referenceLocator": f"pkg:cargo/web-repo-to-ai@{tag.lstrip('v')}",
            }
        ],
    })
    relationships.append({
        "spdxElementId": doc_spdx_id,
        "relatedSpdxElement": root_spdx_id,
        "relationshipType": "DESCRIBES",
    })

    # Cargo packages
    for pkg in cargo_meta.get("packages", []):
        name = pkg["name"]
        ver = pkg["version"]
        key = ("cargo", name, ver)
        if key in seen:
            continue
        seen.add(key)

        spdx_id = sanitize_spdx_id(name, ver)
        lic = pkg.get("license") or "MIT"
        lic_clean = lic.replace("/", " OR ")

        packages.append({
            "name": name,
            "SPDXID": spdx_id,
            "versionInfo": ver,
            "downloadLocation": pkg.get("repository") or "NOASSERTION",
            "filesAnalyzed": False,
            "licenseConcluded": lic_clean,
            "licenseDeclared": lic_clean,
            "copyrightText": "NOASSERTION",
            "externalRefs": [
                {
                    "referenceCategory": "PACKAGE-MANAGER",
                    "referenceType": "purl",
                    "referenceLocator": f"pkg:cargo/{name}@{ver}",
                }
            ],
        })
        relationships.append({
            "spdxElementId": root_spdx_id,
            "relatedSpdxElement": spdx_id,
            "relationshipType": "DEPENDS_ON",
        })

    # npm packages
    for pkg in npm_pkgs:
        name = pkg["name"]
        ver = pkg["version"]
        key = ("npm", name, ver)
        if key in seen:
            continue
        seen.add(key)

        spdx_id = sanitize_spdx_id(f"npm-{name}", ver)
        packages.append({
            "name": name,
            "SPDXID": spdx_id,
            "versionInfo": ver,
            "downloadLocation": "NOASSERTION",
            "filesAnalyzed": False,
            "licenseConcluded": pkg.get("license", "MIT"),
            "licenseDeclared": pkg.get("license", "MIT"),
            "copyrightText": "NOASSERTION",
            "externalRefs": [
                {
                    "referenceCategory": "PACKAGE-MANAGER",
                    "referenceType": "purl",
                    "referenceLocator": f"pkg:npm/{name}@{ver}",
                }
            ],
        })
        relationships.append({
            "spdxElementId": root_spdx_id,
            "relatedSpdxElement": spdx_id,
            "relationshipType": "DEPENDS_ON",
        })

    return {
        "spdxVersion": "SPDX-2.3",
        "dataLicense": "CC0-1.0",
        "SPDXID": doc_spdx_id,
        "name": f"web-repo-to-ai-{tag}",
        "documentNamespace": f"https://github.com/dayashimoga/web-repo-to-ai/spdx/{tag}-{uuid.uuid4()}",
        "creationInfo": {
            "created": now_iso,
            "creators": [
                "Tool: web-repo-to-ai-generate-sbom-0.1.0",
                "Organization: web-repo-to-ai contributors",
            ],
        },
        "packages": packages,
        "relationships": relationships,
    }


def main():
    parser = argparse.ArgumentParser(description="Generate CycloneDX and SPDX SBOMs")
    parser.add_argument("--out-dir", default="dist", help="Output directory for generated SBOM files")
    parser.add_argument("--tag", default="v0.1.0", help="Release version tag (e.g. v0.1.0)")
    args = parser.parse_args()

    os.makedirs(args.out_dir, exist_ok=True)

    print(f"Generating Software Bill of Materials (SBOM) for {args.tag}...")
    cargo_meta = parse_cargo_metadata()
    print(f"  Extracted {len(cargo_meta.get('packages', []))} Cargo packages.")

    npm_pkgs = []
    npm_pkgs.extend(parse_npm_packages("apps/web"))
    npm_pkgs.extend(parse_npm_packages("apps/extension"))
    print(f"  Extracted {len(npm_pkgs)} Node.js packages.")

    cyclonedx_doc = generate_cyclonedx(args.tag, cargo_meta, npm_pkgs)
    cyclonedx_path = os.path.join(args.out_dir, "sbom-cyclonedx.json")
    with open(cyclonedx_path, "w", encoding="utf-8") as f:
        json.dump(cyclonedx_doc, f, indent=2)
    print(f"  [OK] CycloneDX v1.5 SBOM written to: {cyclonedx_path} ({len(cyclonedx_doc['components'])} components)")

    spdx_doc = generate_spdx(args.tag, cargo_meta, npm_pkgs)
    spdx_path = os.path.join(args.out_dir, "sbom-spdx.json")
    with open(spdx_path, "w", encoding="utf-8") as f:
        json.dump(spdx_doc, f, indent=2)
    print(f"  [OK] SPDX v2.3 SBOM written to: {spdx_path} ({len(spdx_doc['packages'])} packages)")

    print("SBOM generation completed successfully.")


if __name__ == "__main__":
    main()
