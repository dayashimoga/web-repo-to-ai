<#
.SYNOPSIS
    Validates reproducible container builds, multi-stage architectures, and the 30 GB storage ceiling.

.DESCRIPTION
    Audits deploy/podman-compose.yml and all Containerfiles for reproducible build patterns,
    unprivileged users, multi-stage layer isolation, and enforces the 30 GB container storage ceiling.

.PARAMETER Quick
    Skips live container startup verification and performs only static specification and budget audits.

.EXAMPLE
    pwsh scripts/verify-container-builds.ps1
    pwsh scripts/verify-container-builds.ps1 -Quick
#>

param(
    [switch]$Quick
)

$ErrorActionPreference = "Stop"
$checks = 0
$failures = 0

function Assert-Test {
    param(
        [bool]$Condition,
        [string]$Message
    )
    $script:checks++
    if ($Condition) {
        Write-Output "  [PASS] $Message"
    } else {
        Write-Output "  [FAIL] $Message"
        $script:failures++
    }
}

Write-Output "========================================================================"
Write-Output "       web-repo-to-ai Container Build & Architecture Audit              "
Write-Output "========================================================================"

# 1. Audit Compose Configuration
Write-Output "`n[1/4] Auditing deploy/podman-compose.yml..."
$composeFile = "deploy/podman-compose.yml"
$composeExists = Test-Path -Path $composeFile -PathType Leaf
Assert-Test $composeExists "deploy/podman-compose.yml exists"

if ($composeExists) {
    $composeContent = Get-Content $composeFile -Raw
    Assert-Test ($composeContent -match "postgres:") "Declares 'postgres' service"
    Assert-Test ($composeContent -match "api:") "Declares 'api' service"
    Assert-Test ($composeContent -match "worker:") "Declares 'worker' service"
    Assert-Test ($composeContent -match "browser-worker:") "Declares 'browser-worker' service"
    Assert-Test ($composeContent -match "web:") "Declares 'web' service"
    Assert-Test ($composeContent -match "internal_net:" -and $composeContent -match "browser_net:") "Declares dual isolated networks ('internal_net', 'browser_net')"
    Assert-Test ($composeContent -match "postgres_data:" -and $composeContent -match "artifact_data:") "Declares persistent named volumes ('postgres_data', 'artifact_data')"
    Assert-Test ($composeContent -match "no-new-privileges:true") "Enforces 'no-new-privileges:true' security sandbox on browser-worker"
}

# 2. Audit Containerfiles Architecture & Unprivileged Users
Write-Output "`n[2/4] Auditing Containerfiles for multi-stage builds and unprivileged users..."

# Containerfile.api
$apiFile = "deploy/Containerfile.api"
if (Test-Path -Path $apiFile -PathType Leaf) {
    $apiContent = Get-Content $apiFile -Raw
    Assert-Test ($apiContent -match "FROM .* AS builder" -and $apiContent -match "FROM .* AS runtime") "Containerfile.api uses multi-stage build"
    Assert-Test ($apiContent -match "rust:.*slim" -and $apiContent -match "debian:.*slim") "Containerfile.api uses minimal slim base images"
    Assert-Test ($apiContent -match "USER appuser") "Containerfile.api enforces unprivileged runtime user 'appuser'"
}

# Containerfile.worker
$workerFile = "deploy/Containerfile.worker"
if (Test-Path -Path $workerFile -PathType Leaf) {
    $workerContent = Get-Content $workerFile -Raw
    Assert-Test ($workerContent -match "FROM .* AS builder" -and $workerContent -match "FROM .* AS runtime") "Containerfile.worker uses multi-stage build"
    Assert-Test ($workerContent -match "rust:.*slim" -and $workerContent -match "debian:.*slim") "Containerfile.worker uses minimal slim base images"
    Assert-Test ($workerContent -match "USER appuser") "Containerfile.worker enforces unprivileged runtime user 'appuser'"
}

# Containerfile.browser-worker
$bwFile = "deploy/Containerfile.browser-worker"
if (Test-Path -Path $bwFile -PathType Leaf) {
    $bwContent = Get-Content $bwFile -Raw
    Assert-Test ($bwContent -match "playwright:") "Containerfile.browser-worker uses pinned Playwright base image"
    Assert-Test ($bwContent -match "USER pwuser") "Containerfile.browser-worker enforces unprivileged user 'pwuser'"
}

# Containerfile.web
$webFile = "deploy/Containerfile.web"
if (Test-Path -Path $webFile -PathType Leaf) {
    $webContent = Get-Content $webFile -Raw
    Assert-Test ($webContent -match "FROM .* AS builder" -and $webContent -match "FROM .* AS runtime") "Containerfile.web uses multi-stage build"
    Assert-Test ($webContent -match "node:.*slim") "Containerfile.web uses minimal Node slim base image"
    Assert-Test ($webContent -match "USER node") "Containerfile.web enforces unprivileged runtime user 'node'"
}

# 3. Verify Storage Budget & 30 GB Ceiling
Write-Output "`n[3/4] Enforcing 30 GB Podman storage budget..."
$chkScript = "scripts/check-resources.ps1"
if (Test-Path -Path $chkScript -PathType Leaf) {
    powershell -ExecutionPolicy Bypass -File $chkScript -Quiet
    Assert-Test ($LASTEXITCODE -eq 0) "Podman storage footprint is strictly within the 30 GB ceiling"
}

# 4. Live Container Startup & Healthcheck (if not Quick)
if (-not $Quick) {
    Write-Output "`n[4/4] Validating live container startup & healthcheck..."
    try {
        powershell -ExecutionPolicy Bypass -File scripts/dev-up.ps1 -Services postgres -TimeoutSeconds 30
        Assert-Test ($LASTEXITCODE -eq 0) "scripts/dev-up.ps1 successfully started PostgreSQL and verified healthcheck"

        # Verify running state via dev-status
        $statusOut = powershell -ExecutionPolicy Bypass -File scripts/dev-status.ps1 -Json | ConvertFrom-Json
        $pgSvc = $statusOut.Services | Where-Object { $_.Service -eq "postgres" }
        Assert-Test ($pgSvc.State -eq "RUNNING" -and $pgSvc.Health -eq "healthy") "web-repo-postgres reports RUNNING and HEALTHY in live inspection"

        # Verify teardown retains volumes
        powershell -ExecutionPolicy Bypass -File scripts/dev-down.ps1
        Assert-Test ($LASTEXITCODE -eq 0) "scripts/dev-down.ps1 successfully stopped containers"

        $vols = & podman volume ls --format "{{.Name}}" 2>&1
        $hasPostgresVol = [bool]($vols -match "postgres_data")
        Assert-Test $hasPostgresVol "Persistent volume 'deploy_postgres_data' retained across teardown"
    } catch {
        Write-Warning "Live container test failed: $_"
        $script:failures++
    }
} else {
    Write-Output "`n[4/4] Live container startup skipped (-Quick mode)."
}

Write-Output "------------------------------------------------------------------------"
Write-Output ("Container Verification Summary: {0}/{1} checks passed, {2} failures" -f ($checks - $failures), $checks, $failures)
Write-Output "========================================================================"

if ($failures -gt 0) { exit 1 }
exit 0
