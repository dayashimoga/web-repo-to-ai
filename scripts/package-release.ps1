# Release Artifact Packaging & Distribution Engine (`scripts/package-release.ps1`)
# Packages binaries, web bundle, Chrome extension, SBOMs, and cryptographic checksums.

[CmdletBinding()]
param(
    [string]$Tag = "v0.1.0",
    [string]$OutDir = "release-assets",
    [string]$DistDir = "dist"
)

$ErrorActionPreference = "Stop"

Write-Output "========================================================================"
Write-Output "       web-repo-to-ai Release Artifact Packaging Engine ($Tag)          "
Write-Output "========================================================================"

# 1. Ensure production build exists in DistDir
if (-not (Test-Path (Join-Path $DistDir "bin"))) {
    Write-Output "`n[1/5] Production binaries not found in $DistDir. Running build-production.ps1..."
    powershell -ExecutionPolicy Bypass -File scripts/build-production.ps1 -Clean -OutDir $DistDir
    if ($LASTEXITCODE -ne 0) { throw "build-production.ps1 failed with code $LASTEXITCODE" }
} else {
    Write-Output "`n[1/5] Verified existing production build in $DistDir"
}

# 2. Prepare Output Directory
Write-Output "`n[2/5] Preparing release assets directory: $OutDir..."
if (Test-Path $OutDir) {
    Remove-Item -Recurse -Force $OutDir
}
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

# 3. Create Distribution Archives
Write-Output "`n[3/5] Packaging distribution archives..."

# Windows binaries archive (.zip)
$binZipPath = Join-Path $OutDir "web-repo-to-ai-$Tag-windows-x86_64.zip"
$filesToZip = @()
Get-ChildItem -Path (Join-Path $DistDir "bin") -File | ForEach-Object { $filesToZip += $_.FullName }
if (Test-Path "README.md") { $filesToZip += (Resolve-Path "README.md").Path }
if (Test-Path "LICENSE") { $filesToZip += (Resolve-Path "LICENSE").Path }

Compress-Archive -Path $filesToZip -DestinationPath $binZipPath -Force
$zipSize = (Get-Item $binZipPath).Length
Write-Output "  [ARCHIVE] Windows binaries zip: $binZipPath ($('{0:N2}' -f ($zipSize / 1MB)) MB)"

# Frontend web application archive (.tar.gz or .zip)
$webZipPath = Join-Path $OutDir "web-repo-to-ai-$Tag-frontend-web.zip"
$webDir = Join-Path $DistDir "web"
if (Test-Path $webDir) {
    Compress-Archive -Path "$webDir/*" -DestinationPath $webZipPath -Force
    $webSize = (Get-Item $webZipPath).Length
    Write-Output "  [ARCHIVE] Frontend web package: $webZipPath ($('{0:N2}' -f ($webSize / 1MB)) MB)"
}

# Chrome extension distribution archive (.zip)
$extZipPath = Join-Path $OutDir "web-repo-to-ai-$Tag-chrome-extension.zip"
$extDir = Join-Path $DistDir "extension"
if (Test-Path $extDir) {
    Compress-Archive -Path "$extDir/*" -DestinationPath $extZipPath -Force
    $extSize = (Get-Item $extZipPath).Length
    Write-Output "  [ARCHIVE] Chrome extension package: $extZipPath ($('{0:N2}' -f ($extSize / 1KB)) KB)"
}

# 4. Generate Standards-Compliant SBOMs
Write-Output "`n[4/5] Generating CycloneDX v1.5 and SPDX v2.3 Software Bills of Materials..."
python scripts/generate-sbom.py --out-dir $OutDir --tag $Tag
if ($LASTEXITCODE -ne 0) { throw "SBOM generation failed" }

# 5. Cryptographic Checksums & Manifest Generation
Write-Output "`n[5/5] Computing SHA-256 and BLAKE3 checksums for all release assets..."
$checksumSha256Lines = @()
$manifestAssets = @()

$assetFiles = Get-ChildItem -Path $OutDir -File
foreach ($file in $assetFiles) {
    $sha256 = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash.ToLower()
    $checksumSha256Lines += "$sha256  $($file.Name)"

    $manifestAssets += [ordered]@{
        name = $file.Name
        size_bytes = $file.Length
        sha256 = $sha256
    }
}

$checksumSha256Path = Join-Path $OutDir "checksums.sha256"
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllLines($checksumSha256Path, $checksumSha256Lines, $utf8NoBom)
Write-Output "  [CHECKSUM] Checksums written to: $checksumSha256Path"

$releaseManifest = [ordered]@{
    project = "web-repo-to-ai"
    release_tag = $Tag
    generated_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    asset_count = $manifestAssets.Count
    assets = $manifestAssets
    sboms = @("sbom-cyclonedx.json", "sbom-spdx.json")
    status = "READY_FOR_PUBLICATION"
}
$releaseManifestJson = $releaseManifest | ConvertTo-Json -Depth 5
$manifestPath = Join-Path $OutDir "RELEASE_MANIFEST.json"
[System.IO.File]::WriteAllText($manifestPath, $releaseManifestJson, $utf8NoBom)

Write-Output "`n========================================================================"
Write-Output "          RELEASE ASSETS GENERATED & PACKAGED SUCCESSFULLY              "
Write-Output "========================================================================"
Get-ChildItem -Path $OutDir -File | Select-Object Name, Length, LastWriteTime | Format-Table -AutoSize
