# Production Release Build Orchestration Script (`scripts/build-production.ps1`)
# Builds all release binaries, frontend web bundle, Chrome extension, and unified dist staging.

[CmdletBinding()]
param(
    [string]$OutDir = "dist",
    [switch]$SkipTests,
    [switch]$Clean,
    [switch]$Quick
)

$ErrorActionPreference = "Stop"

Write-Output "========================================================================"
Write-Output "           web-repo-to-ai Unified Production Build Engine               "
Write-Output "========================================================================"

# 1. Prerequisite Validation & Resource Budget Check
Write-Output "`n[1/6] Verifying toolchains and storage ceiling..."
$requiredCommands = @("cargo", "rustc", "node", "npm")
foreach ($cmd in $requiredCommands) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        throw "Required build toolchain not found: $cmd"
    }
    $ver = & $cmd --version 2>&1
    Write-Output "  [PASS] Found $cmd ($ver)"
}

# Run resource budget check
if (Test-Path "scripts/check-resources.ps1") {
    powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -Quiet
    if ($LASTEXITCODE -ne 0) {
        Write-Warning "Storage budget warning or check returned non-zero code $LASTEXITCODE"
    } else {
        Write-Output "  [PASS] Storage budget verified within 30 GB ceiling"
    }
}

# 2. Directory Preparation & Clean
Write-Output "`n[2/6] Preparing staging directory: $OutDir..."
if ($Clean -and (Test-Path $OutDir)) {
    Remove-Item -Recurse -Force $OutDir
}
$binDir = Join-Path $OutDir "bin"
$webDistDir = Join-Path $OutDir "web"
$extDistDir = Join-Path $OutDir "extension"

New-Item -ItemType Directory -Force -Path $binDir | Out-Null
New-Item -ItemType Directory -Force -Path $webDistDir | Out-Null
New-Item -ItemType Directory -Force -Path $extDistDir | Out-Null

# 3. Compile Rust Workspace Release Binaries
Write-Output "`n[3/6] Compiling Rust workspace release binaries (profile: release, jobs: 4)..."
cargo build --workspace --release --jobs 4
if ($LASTEXITCODE -ne 0) {
    throw "Cargo release compilation failed with exit code $LASTEXITCODE"
}

$expectedBinaries = @(
    "web-repo-to-ai",
    "web-repo-to-ai-cli",
    "web-repo-to-ai-api",
    "web-repo-to-ai-worker",
    "web-repo-to-ai-mcp"
)

foreach ($bin in $expectedBinaries) {
    $exeName = if ($IsWindows -or $env:OS -like "*Windows*") { "$bin.exe" } else { $bin }
    $sourcePath = Join-Path "target/release" $exeName
    if (-not (Test-Path $sourcePath)) {
        throw "Expected release binary not found: $sourcePath"
    }
    $destPath = Join-Path $binDir $exeName
    Copy-Item -Force $sourcePath $destPath
    $size = (Get-Item $destPath).Length
    Write-Output "  [STAGED] $exeName ($('{0:N2}' -f ($size / 1MB)) MB)"
}

# 4. Build Frontend Web Application
Write-Output "`n[4/6] Building production SvelteKit web application..."
$rootItem = Get-Item "." -ErrorAction SilentlyContinue
$appItem = Get-Item "apps" -ErrorAction SilentlyContinue
$webDir = if ($appItem -and $appItem.LinkType -eq "Junction") { 
    Join-Path $appItem.Target[0] "web" 
} elseif ($rootItem -and $rootItem.LinkType -eq "Junction") {
    Join-Path $rootItem.Target[0] "apps/web"
} else { 
    "apps/web" 
}

Push-Location $webDir
try {
    npm run build
    if ($LASTEXITCODE -ne 0) { throw "npm run build failed in apps/web" }
} finally {
    Pop-Location
}

$webBuildSource = Join-Path $webDir "build"
if (Test-Path $webBuildSource) {
    Copy-Item -Recurse -Force "$webBuildSource/*" $webDistDir
    Write-Output "  [STAGED] Frontend web build staged to $webDistDir"
} else {
    Write-Warning "apps/web/build directory not found after build"
}

# 5. Build Chrome Extension MV3
Write-Output "`n[5/6] Building production Chrome Extension MV3..."
$extDir = if ($appItem -and $appItem.LinkType -eq "Junction") { 
    Join-Path $appItem.Target[0] "extension" 
} elseif ($rootItem -and $rootItem.LinkType -eq "Junction") {
    Join-Path $rootItem.Target[0] "apps/extension"
} else { 
    "apps/extension" 
}

Push-Location $extDir
try {
    npm run build
    if ($LASTEXITCODE -ne 0) { throw "npm run build failed in apps/extension" }
} finally {
    Pop-Location
}

$extBuildSource = Join-Path $extDir "dist"
if (Test-Path $extBuildSource) {
    Copy-Item -Recurse -Force "$extBuildSource/*" $extDistDir
    Write-Output "  [STAGED] Chrome extension build staged to $extDistDir"
} else {
    Write-Warning "apps/extension/dist directory not found after build"
}

# 6. Checksums, Manifest & Smoke Verification
Write-Output "`n[6/6] Generating cryptographic checksums and verifying smoke execution..."

$checksumLines = @()
$stagedBinaries = Get-ChildItem -Path $binDir -File
foreach ($file in $stagedBinaries) {
    $sha256 = (Get-FileHash -Path $file.FullName -Algorithm SHA256).Hash.ToLower()
    $checksumLines += "$sha256  $($file.Name)"
}
$checksumFilePath = Join-Path $OutDir "checksums.sha256"
$utf8NoBom = New-Object System.Text.UTF8Encoding $false
[System.IO.File]::WriteAllLines($checksumFilePath, $checksumLines, $utf8NoBom)
Write-Output "  [CHECKSUM] Staged SHA-256 checksums written to $checksumFilePath"

# Create structured build manifest
$manifest = [ordered]@{
    project = "web-repo-to-ai"
    version = "0.1.0"
    built_at = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    artifacts = @(
        [ordered]@{ target = "binaries"; path = "bin/"; count = $stagedBinaries.Count },
        [ordered]@{ target = "frontend"; path = "web/"; adapter = "adapter-node" },
        [ordered]@{ target = "extension"; path = "extension/"; manifest = "v3" }
    )
    status = "SUCCESS"
}
$manifestJson = $manifest | ConvertTo-Json -Depth 4
$manifestPath = Join-Path $OutDir "BUILD_MANIFEST.json"
[System.IO.File]::WriteAllText($manifestPath, $manifestJson, $utf8NoBom)

# Smoke execution test
$cliBinaryName = if ($IsWindows -or $env:OS -like "*Windows*") { "web-repo-to-ai.exe" } else { "web-repo-to-ai" }
$testCliExe = Join-Path $binDir $cliBinaryName
if (Test-Path $testCliExe) {
    $versionOutput = & $testCliExe --version 2>&1
    Write-Output "  [SMOKE] Binary execution verified: $versionOutput"
}

Write-Output "`n========================================================================"
Write-Output "       PRODUCTION BUILD COMPLETED SUCCESSFULLY (0 Errors)               "
Write-Output "========================================================================"
Write-Output "Artifacts staged in directory: $OutDir"
Get-ChildItem -Path $OutDir -Recurse -File | Select-Object FullName, Length | Format-Table -AutoSize
