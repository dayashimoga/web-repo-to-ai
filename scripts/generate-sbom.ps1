# Software Bill of Materials (SBOM) Generation Script (`scripts/generate-sbom.ps1`)
# Wraps scripts/generate-sbom.py to generate CycloneDX v1.5 and SPDX v2.3 SBOMs.

[CmdletBinding()]
param(
    [string]$OutDir = "dist",
    [string]$Tag = "v0.1.0"
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
    throw "Python 3 is required to generate SBOMs."
}

python scripts/generate-sbom.py --out-dir $OutDir --tag $Tag
if ($LASTEXITCODE -ne 0) {
    throw "SBOM generation failed with exit code $LASTEXITCODE"
}
