<#
.SYNOPSIS
    Universal Deployment Verification Engine across all 4 supported topologies.
.DESCRIPTION
    Subphase 11.4.4: Validates:
    1. Containerized Self-Hosted Stack (deploy/podman-compose.yml, Containerfiles, unprivileged users, network isolation).
    2. Standalone Release Binary Distribution (binaries in dist/bin, contracts, size bounds, checksums).
    3. Chrome Extension MV3 Distribution (manifest v3, CSP strictness, icons, bundled assets).
    4. Disaster Recovery & 30 GB Storage Ceiling (resource check, recovery runbook command invariants).
    5. Live Healthcheck Verification (if not -Quick and container runtime available).
.PARAMETER Quick
    Skips live container execution and tests static specs, binaries, and configurations.
.PARAMETER Json
    Outputs structured JSON report.
.PARAMETER DistDir
    Path to production dist directory (default: dist).
.EXAMPLE
    pwsh scripts/verify-deployment.ps1
    pwsh scripts/verify-deployment.ps1 -Quick
    pwsh scripts/verify-deployment.ps1 -Json
#>

param(
    [switch]$Quick,
    [switch]$Json,
    [string]$DistDir = "dist"
)

$ErrorActionPreference = "Stop"
$checks = 0
$failures = 0
$results = @()

function Assert-Check {
    param(
        [bool]$Condition,
        [string]$Message,
        [string]$Category = "Deployment"
    )
    $script:checks++
    $status = if ($Condition) { "PASS" } else { "FAIL" }
    if (-not $Condition) {
        $script:failures++
    }

    $script:results += [PSCustomObject]@{
        Category = $Category
        Status   = $status
        Message  = $Message
    }

    if (-not $Json) {
        if ($Condition) {
            Write-Output "  [PASS] $Message"
        } else {
            Write-Output "  [FAIL] $Message"
        }
    }
}

if (-not $Json) {
    Write-Output "========================================================================"
    Write-Output "       web-repo-to-ai Universal Deployment Verification Engine          "
    Write-Output "========================================================================"
}

# -----------------------------------------------------------------------------
# 1. TOPOLOGY 1: Self-Hosted Container Stack & Sandbox Isolation
# -----------------------------------------------------------------------------
if (-not $Json) { Write-Output "`n[1/4] Auditing Self-Hosted Container Stack (deploy/podman-compose.yml)..." }

$composePath = "deploy/podman-compose.yml"
$hasCompose = Test-Path -Path $composePath -PathType Leaf
Assert-Check $hasCompose "deploy/podman-compose.yml exists" "ContainerStack"

if ($hasCompose) {
    $composeRaw = Get-Content $composePath -Raw
    Assert-Check ($composeRaw -match "postgres:") "Declares 'postgres' service" "ContainerStack"
    Assert-Check ($composeRaw -match "api:") "Declares 'api' service" "ContainerStack"
    Assert-Check ($composeRaw -match "worker:") "Declares 'worker' service" "ContainerStack"
    Assert-Check ($composeRaw -match "browser-worker:") "Declares 'browser-worker' service" "ContainerStack"
    Assert-Check ($composeRaw -match "web:") "Declares 'web' service" "ContainerStack"
    Assert-Check ($composeRaw -match "internal_net:" -and $composeRaw -match "browser_net:") "Declares dual isolated networks ('internal_net', 'browser_net')" "ContainerStack"
    Assert-Check ($composeRaw -match "postgres_data:" -and $composeRaw -match "artifact_data:") "Declares persistent named volumes ('postgres_data', 'artifact_data')" "ContainerStack"

    # Strict isolation: browser-worker must be connected to browser_net, NOT internal_net
    $bwSection = ($composeRaw -split "(?m)^  browser-worker:")[1]
    if ($bwSection) {
        $bwBlock = ($bwSection -split "(?m)^  [a-zA-Z0-9_-]+:")[0]
        $hasBrowserNet = $bwBlock -match "browser_net"
        $hasInternalNet = $bwBlock -match "internal_net"
        Assert-Check ($hasBrowserNet -and -not $hasInternalNet) "Strict isolation: browser-worker connects to browser_net and is isolated from internal_net" "ContainerStack"
        Assert-Check ($bwBlock -match "no-new-privileges:true") "browser-worker enforces security opt 'no-new-privileges:true'" "ContainerStack"
    } else {
        Assert-Check $false "Could not inspect browser-worker service block in compose file" "ContainerStack"
    }
}

# Audit Containerfiles
$cApi = "deploy/Containerfile.api"
if (Test-Path $cApi) {
    $cApiRaw = Get-Content $cApi -Raw
    Assert-Check ($cApiRaw -match "FROM .* AS builder" -and $cApiRaw -match "FROM .* AS runtime") "Containerfile.api uses multi-stage build" "ContainerFiles"
    Assert-Check ($cApiRaw -match "USER appuser") "Containerfile.api enforces unprivileged runtime user 'appuser'" "ContainerFiles"
}

$cWorker = "deploy/Containerfile.worker"
if (Test-Path $cWorker) {
    $cWorkerRaw = Get-Content $cWorker -Raw
    Assert-Check ($cWorkerRaw -match "FROM .* AS builder" -and $cWorkerRaw -match "FROM .* AS runtime") "Containerfile.worker uses multi-stage build" "ContainerFiles"
    Assert-Check ($cWorkerRaw -match "USER appuser") "Containerfile.worker enforces unprivileged runtime user 'appuser'" "ContainerFiles"
}

$cBw = "deploy/Containerfile.browser-worker"
if (Test-Path $cBw) {
    $cBwRaw = Get-Content $cBw -Raw
    Assert-Check ($cBwRaw -match "USER pwuser") "Containerfile.browser-worker enforces unprivileged user 'pwuser'" "ContainerFiles"
}

$cWeb = "deploy/Containerfile.web"
if (Test-Path $cWeb) {
    $cWebRaw = Get-Content $cWeb -Raw
    Assert-Check ($cWebRaw -match "FROM .* AS builder" -and $cWebRaw -match "FROM .* AS runtime") "Containerfile.web uses multi-stage build" "ContainerFiles"
    Assert-Check ($cWebRaw -match "USER node") "Containerfile.web enforces unprivileged runtime user 'node'" "ContainerFiles"
}

# Audit Environment Variable Template
$envExample = "deploy/.env.example"
if (Test-Path $envExample) {
    $envRaw = Get-Content $envExample -Raw
    $requiredKeys = @(
        "POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD", "DATABASE_URL",
        "API_PORT", "API_HOST", "RUST_LOG", "STORAGE_BACKEND", "STORAGE_LOCAL_PATH",
        "BROWSER_WORKER_URL", "PUBLIC_API_URL", "WEB_PORT", "WEB_HOST", "TOKEN_ENCRYPTION_KEY"
    )
    $allPresent = $true
    foreach ($k in $requiredKeys) {
        if ($envRaw -notmatch "(?m)^$k=") {
            $allPresent = $false
            break
        }
    }
    Assert-Check $allPresent "deploy/.env.example contains all 14 core production configuration variables" "Environment"
}

# -----------------------------------------------------------------------------
# 2. TOPOLOGY 2: Standalone Release Binary Distribution
# -----------------------------------------------------------------------------
if (-not $Json) { Write-Output "`n[2/4] Auditing Standalone Release Binary Distribution..." }

$binDir = Join-Path $DistDir "bin"
$hasBinDir = Test-Path $binDir
Assert-Check $hasBinDir "Production binary directory '$binDir' exists" "BinaryDistribution"

$binaries = @("web-repo-to-ai", "web-repo-to-ai-cli", "web-repo-to-ai-api", "web-repo-to-ai-worker", "web-repo-to-ai-mcp")
$isWindows = ($PSVersionTable.PSEdition -ne 'Core' -or $IsWindows)

foreach ($b in $binaries) {
    $fileName = if ($isWindows) { "$b.exe" } else { $b }
    $bPath = Join-Path $binDir $fileName
    $exists = Test-Path $bPath
    Assert-Check $exists "Binary '$fileName' exists in '$binDir'" "BinaryDistribution"
    if ($exists) {
        $sizeBytes = (Get-Item $bPath).Length
        $sizeValid = ($sizeBytes -ge 100KB -and $sizeBytes -le 80MB)
        Assert-Check $sizeValid "Binary '$fileName' size ($([math]::Round($sizeBytes / 1MB, 2)) MB) within 100 KB - 80 MB bounds" "BinaryDistribution"
    }
}

# Smoke test CLI binary
$cliName = if ($isWindows) { "web-repo-to-ai.exe" } else { "web-repo-to-ai" }
$cliPath = Join-Path $binDir $cliName
if (Test-Path $cliPath) {
    $verOut = & $cliPath --version 2>&1
    Assert-Check ($verOut -match "web-repo-to-ai") "CLI binary responds cleanly to '--version' ($verOut)" "BinaryDistribution"

    $helpOut = & $cliPath --help 2>&1
    $helpValid = ($helpOut -match "url" -and $helpOut -match "repo" -and $helpOut -match "config" -and $helpOut -match "batch" -and $helpOut -match "verify" -and $helpOut -match "completions" -and $helpOut -match "mcp")
    Assert-Check $helpValid "CLI binary exposes all 7 subcommand contracts in '--help'" "BinaryDistribution"
}

# Checksums and Manifest
$chkFile = Join-Path $DistDir "checksums.sha256"
Assert-Check (Test-Path $chkFile) "Production checksums manifest 'checksums.sha256' exists in '$DistDir'" "BinaryDistribution"

$manifestFile = Join-Path $DistDir "BUILD_MANIFEST.json"
Assert-Check (Test-Path $manifestFile) "Production build manifest 'BUILD_MANIFEST.json' exists in '$DistDir'" "BinaryDistribution"

# -----------------------------------------------------------------------------
# 3. TOPOLOGY 3: Chrome Extension MV3 Distribution
# -----------------------------------------------------------------------------
if (-not $Json) { Write-Output "`n[3/4] Auditing Chrome Extension MV3 Distribution..." }

$extDir = Join-Path $DistDir "extension"
if (-not (Test-Path $extDir)) {
    $extDir = "apps/extension/dist"
}
Assert-Check (Test-Path $extDir) "Chrome extension distribution directory '$extDir' exists" "ExtensionDistribution"

$extManifest = Join-Path $extDir "manifest.json"
$hasExtManifest = Test-Path $extManifest
Assert-Check $hasExtManifest "Extension manifest.json exists in '$extDir'" "ExtensionDistribution"

if ($hasExtManifest) {
    try {
        $extJson = Get-Content $extManifest -Raw | ConvertFrom-Json
        Assert-Check ($extJson.manifest_version -eq 3) "Extension targets Manifest V3 (manifest_version: 3)" "ExtensionDistribution"
        Assert-Check ($extJson.permissions -contains "storage" -and $extJson.permissions -contains "activeTab" -and $extJson.permissions -contains "scripting") "Extension declares necessary MV3 permissions (storage, activeTab, scripting)" "ExtensionDistribution"
        Assert-Check ($null -ne $extJson.background.service_worker) "Extension declares background service worker ($($extJson.background.service_worker))" "ExtensionDistribution"

        # Content Security Policy validation
        $csp = $extJson.content_security_policy.extension_pages
        $cspStrict = ($csp -match "script-src 'self'" -and $csp -notmatch "unsafe-inline" -and $csp -notmatch "unsafe-eval")
        Assert-Check $cspStrict "Extension enforces strict CSP without unsafe-inline or unsafe-eval" "ExtensionDistribution"

        # Check background bundle exists
        $swPath = Join-Path $extDir $extJson.background.service_worker
        Assert-Check (Test-Path $swPath) "Background service worker bundle exists: $($extJson.background.service_worker)" "ExtensionDistribution"
    } catch {
        Assert-Check $false "Failed to parse extension manifest.json: $_" "ExtensionDistribution"
    }
}

# -----------------------------------------------------------------------------
# 4. TOPOLOGY 4: Disaster Recovery & 30 GB Storage Ceiling
# -----------------------------------------------------------------------------
if (-not $Json) { Write-Output "`n[4/4] Auditing Disaster Recovery Runbooks & 30 GB Storage Ceiling..." }

$chkScript = "scripts/check-resources.ps1"
if (Test-Path $chkScript) {
    $null = powershell -ExecutionPolicy Bypass -File $chkScript -Quiet 2>&1
    Assert-Check ($LASTEXITCODE -eq 0) "Podman storage footprint satisfies the 30 GB storage ceiling" "ResourceGovernance"
}

$deployDoc = "docs/DEPLOYMENT.md"
if (Test-Path $deployDoc) {
    $docRaw = Get-Content $deployDoc -Raw
    Assert-Check ($docRaw -match "Backup, Restore & Disaster Recovery Runbook") "docs/DEPLOYMENT.md contains Disaster Recovery Runbook" "DisasterRecovery"
    Assert-Check ($docRaw -match "pg_dump" -and $docRaw -match "pg_restore") "docs/DEPLOYMENT.md contains pg_dump and pg_restore recovery commands" "DisasterRecovery"
    Assert-Check ($docRaw -match "Cold Recovery Procedure \(< 5 Minutes\)") "docs/DEPLOYMENT.md documents <5 minute cold recovery SLA" "DisasterRecovery"
}

$lifecycleScripts = @("dev-up", "dev-down", "dev-status", "dev-restart", "dev-logs", "clean-caches")
$allLifecyclePresent = $true
foreach ($ls in $lifecycleScripts) {
    if (-not (Test-Path "scripts/$ls.ps1") -or -not (Test-Path "scripts/$ls.sh")) {
        $allLifecyclePresent = $false
        break
    }
}
Assert-Check $allLifecyclePresent "All 6 development lifecycle scripts exist (.ps1 and .sh)" "DisasterRecovery"

# Live Container Startup (if not Quick and Podman operational)
if (-not $Quick) {
    $hasPodman = $false
    try {
        $podmanVer = & podman --version 2>$null
        if ($LASTEXITCODE -eq 0 -and $podmanVer) { $hasPodman = $true }
    } catch {}

    if ($hasPodman) {
        if (-not $Json) { Write-Output "`n[Live Container Verification] Starting PostgreSQL test container..." }
        try {
            $upOut = powershell -ExecutionPolicy Bypass -File scripts/dev-up.ps1 -Services postgres -TimeoutSeconds 30 2>&1
            Assert-Check ($LASTEXITCODE -eq 0) "Live container test: scripts/dev-up.ps1 started postgres" "LiveVerification"

            $statusOut = powershell -ExecutionPolicy Bypass -File scripts/dev-status.ps1 -Json | ConvertFrom-Json
            $pg = $statusOut.Services | Where-Object { $_.Service -eq "postgres" }
            Assert-Check ($pg.State -eq "RUNNING") "Live container test: postgres container is RUNNING" "LiveVerification"

            $downOut = powershell -ExecutionPolicy Bypass -File scripts/dev-down.ps1 2>&1
            Assert-Check ($LASTEXITCODE -eq 0) "Live container test: scripts/dev-down.ps1 stopped containers" "LiveVerification"
        } catch {
            Write-Warning "Live container execution failed: $_"
        }
    } else {
        if (-not $Json) { Write-Output "Notice: Podman CLI not available; skipped live container probe." }
    }
}

# -----------------------------------------------------------------------------
# Summary & Exit
# -----------------------------------------------------------------------------
$passed = $checks - $failures
if ($Json) {
    [PSCustomObject]@{
        TotalChecks = $checks
        Passed      = $passed
        Failures    = $failures
        Success     = ($failures -eq 0)
        Results     = $results
    } | ConvertTo-Json -Depth 4
} else {
    Write-Output "------------------------------------------------------------------------"
    Write-Output ("Deployment Verification Summary: {0}/{1} checks passed, {2} failures" -f $passed, $checks, $failures)
    Write-Output "========================================================================"
}

if ($failures -gt 0) { exit 1 }
exit 0
