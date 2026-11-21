#!/usr/bin/env bash
# Software Bill of Materials (SBOM) Generation Script (`scripts/generate-sbom.sh`)
# Wraps scripts/generate-sbom.py to generate CycloneDX v1.5 and SPDX v2.3 SBOMs.

set -euo pipefail

OUT_DIR="${1:-dist}"
TAG="${2:-v0.1.0}"

if ! command -v python3 > /dev/null 2>&1 && ! command -v python > /dev/null 2>&1; then
    echo "Error: Python 3 is required to generate SBOMs." >&2
    exit 1
fi

PY_BIN="python3"
if ! command -v python3 > /dev/null 2>&1; then
    PY_BIN="python"
fi

"$PY_BIN" scripts/generate-sbom.py --out-dir "$OUT_DIR" --tag "$TAG"
