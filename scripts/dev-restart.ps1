<#
.SYNOPSIS
    Restarts containerized development services for web-repo-to-ai.

.DESCRIPTION
    Restarts services defined in deploy/podman-compose.yml.
    Supports restarting individual services (e.g. postgres, api, worker),
    optional image rebuilding before restart, and waiting for database healthcheck.

.PARAMETER Services
    Optional subset of services to restart (e.g. 'api', 'postgres', 'web').
    Defaults to all services.

.PARAMETER Build
    Rebuilds container images prior to restarting.

.PARAMETER TimeoutSeconds
    Maximum seconds to wait for database healthcheck if postgres was restarted (default: 60).

.PARAMETER DryRun
    Prints the restart command without restarting containers.

.EXAMPLE
    pwsh scripts/dev-restart.ps1
    pwsh scripts/dev-restart.ps1 -Services api
    pwsh scripts/dev-restart.ps1 -Build
    pwsh scripts/dev-restart.ps1 -DryRun
#>

param(
    [string[]]$Services = @(),
    [switch]$Build,
    [int]$TimeoutSeconds = 60,
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

function Invoke-Compose {
    param([string[]]$ComposeArgs)
    if (Get-Command "podman-compose" -ErrorAction SilentlyContinue) {
        & podman-compose @ComposeArgs
    } elseif (Get-Command "podman" -ErrorAction SilentlyContinue) {
        & podman compose @ComposeArgs
    } elseif (Get-Command "docker-compose" -ErrorAction SilentlyContinue) {
        & docker-compose @ComposeArgs
    } else {
        throw "Neither podman-compose, podman compose, nor docker-compose was found in PATH."
    }
}

Write-Output "========================================================================"
Write-Output "           web-repo-to-ai Service Restart Utility                       "
Write-Output "========================================================================"

$composeFile = "deploy/podman-compose.yml"
if (-not (Test-Path -Path $composeFile -PathType Leaf)) {
    throw "Compose configuration file not found at $composeFile"
}

# 1. Optional Rebuild
if ($Build) {
    $buildArgs = @("-f", $composeFile)
    if ($DryRun) { $buildArgs += "--dry-run" }
    $buildArgs += "build"
    if ($Services.Count -gt 0) { $buildArgs += $Services }
    Write-Output ("Executing build: {0}" -f ($buildArgs -join " "))
    if (-not $DryRun) {
        Invoke-Compose $buildArgs
        if ($LASTEXITCODE -ne 0) { throw "Build failed with exit code $LASTEXITCODE" }
    }
}

# 2. Restart Services
$cmdArgs = @("-f", $composeFile)
if ($DryRun) { $cmdArgs += "--dry-run" }
$cmdArgs += "restart"
if ($Services.Count -gt 0) { $cmdArgs += $Services }

Write-Output ("Executing restart: {0}" -f ($cmdArgs -join " "))

if ($DryRun) {
    Write-Output "`n[DRY RUN] Command formulated successfully. No services restarted."
    exit 0
}

Invoke-Compose $cmdArgs
if ($LASTEXITCODE -ne 0) {
    throw "Restart failed with exit code $LASTEXITCODE"
}

# 3. Wait for PostgreSQL Healthcheck if restarted
$shouldCheckPostgres = ($Services.Count -eq 0) -or ($Services -contains "postgres")
if ($shouldCheckPostgres) {
    Write-Output "Waiting for PostgreSQL service healthcheck..."
    $elapsed = 0
    $isHealthy = $false
    while ($elapsed -lt $TimeoutSeconds) {
        try {
            $inspectOut = & podman inspect --format "{{.State.Health.Status}}" web-repo-postgres 2>&1
            if ($inspectOut -match "healthy") {
                $isHealthy = $true
                break
            }
        } catch {}
        Start-Sleep -Seconds 2
        $elapsed += 2
    }
    if ($isHealthy) {
        Write-Output "PostgreSQL is healthy and ready for connections."
    } else {
        Write-Warning "PostgreSQL healthcheck did not report 'healthy' within $TimeoutSeconds seconds."
    }
}

Write-Output "Services restarted successfully."
Write-Output "========================================================================"
exit 0
