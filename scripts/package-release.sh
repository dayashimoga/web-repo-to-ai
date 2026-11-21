#!/usr/bin/env bash
# Release Artifact Packaging & Distribution Engine (`scripts/package-release.sh`)
# Packages binaries, web bundle, Chrome extension, SBOMs, and cryptographic checksums.

set -euo pipefail

TAG="${1:-v0.1.0}"
OUT_DIR="${2:-release-assets}"
DIST_DIR="${3:-dist}"

echo "========================================================================"
echo "       web-repo-to-ai Release Artifact Packaging Engine ($TAG)          "
echo "========================================================================"

# 1. Ensure production build exists in DistDir
if [ ! -d "${DIST_DIR}/bin" ]; then
    echo -e "\n[1/5] Production binaries not found in ${DIST_DIR}. Running build-production.sh..."
    bash scripts/build-production.sh "${DIST_DIR}"
fi

# 2. Prepare Output Directory
echo -e "\n[2/5] Preparing release assets directory: ${OUT_DIR}..."
rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

# 3. Create Distribution Archives
echo -e "\n[3/5] Packaging distribution archives..."

# Linux / Unix binaries archive (.tar.gz)
BIN_TAR="${OUT_DIR}/web-repo-to-ai-${TAG}-linux-x86_64.tar.gz"
tar -czf "${BIN_TAR}" -C "${DIST_DIR}/bin" .
echo "  [ARCHIVE] Binary package: ${BIN_TAR}"

# Frontend web application archive (.tar.gz)
if [ -d "${DIST_DIR}/web" ]; then
    WEB_TAR="${OUT_DIR}/web-repo-to-ai-${TAG}-frontend-web.tar.gz"
    tar -czf "${WEB_TAR}" -C "${DIST_DIR}/web" .
    echo "  [ARCHIVE] Frontend web package: ${WEB_TAR}"
fi

# Chrome extension distribution archive (.zip)
if [ -d "${DIST_DIR}/extension" ]; then
    EXT_ZIP="${OUT_DIR}/web-repo-to-ai-${TAG}-chrome-extension.zip"
    (cd "${DIST_DIR}/extension" && zip -qr "../../${EXT_ZIP}" .)
    echo "  [ARCHIVE] Chrome extension package: ${EXT_ZIP}"
fi

# 4. Generate Standards-Compliant SBOMs
echo -e "\n[4/5] Generating CycloneDX v1.5 and SPDX v2.3 Software Bills of Materials..."
PY_BIN="python3"
if ! command -v python3 > /dev/null 2>&1; then
    PY_BIN="python"
fi
"$PY_BIN" scripts/generate-sbom.py --out-dir "${OUT_DIR}" --tag "${TAG}"

# 5. Cryptographic Checksums & Manifest Generation
echo -e "\n[5/5] Computing SHA-256 checksums for all release assets..."
(
    cd "${OUT_DIR}"
    sha256sum * > checksums.sha256 2>/dev/null || true
)

cat << EOF > "${OUT_DIR}/RELEASE_MANIFEST.json"
{
  "project": "web-repo-to-ai",
  "release_tag": "${TAG}",
  "generated_at": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "sboms": ["sbom-cyclonedx.json", "sbom-spdx.json"],
  "status": "READY_FOR_PUBLICATION"
}
EOF

echo -e "\n========================================================================"
echo "          RELEASE ASSETS GENERATED & PACKAGED SUCCESSFULLY              "
echo "========================================================================"
ls -lh "${OUT_DIR}"
