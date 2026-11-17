<#
.SYNOPSIS
    Starts the containerized development environment for web-repo-to-ai.

.DESCRIPTION
    Bootstraps and starts development services defined in deploy/podman-compose.yml.
    Verifies that the 30 GB storage ceiling is satisfied before launch, ensures deploy/.env exists,
    and reports live endpoint URLs.

.PARAMETER Services
    Optional subset of services to start (e.g., 'postgres', 'api', 'web', 'browser-worker', 'worker').
    Defaults to all services.

.PARAMETER Build
    Forces rebuilding of container images before starting services.

.PARAMETER NoDetach
    Runs compose in the foreground instead of detached (-d) background mode.

.PARAMETER SkipResourceCheck
    Bypasses the pre-flight 30 GB storage ceiling check.

.PARAMETER DryRun
    Prints the execution plan and compose command without starting containers.

.PARAMETER TimeoutSeconds
    Maximum seconds to wait for database healthcheck (default: 60).

.EXAMPLE
    pwsh scripts/dev-up.ps1
    pwsh scripts/dev-up.ps1 -Services postgres
    pwsh scripts/dev-up.ps1 -Build
    pwsh scripts/dev-up.ps1 -DryRun
#>

param(
    [string[]]$Services = @(),
    [switch]$Build,
    [switch]$NoDetach,
    [switch]$SkipResourceCheck,
    [switch]$DryRun,
    [int]$TimeoutSeconds = 60
)

$ErrorActionPreference = "Stop"

# Helper to execute Compose command
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
Write-Output "         web-repo-to-ai Development Environment Bootstrap              "
Write-Output "========================================================================"

# 1. Ensure deploy/.env exists
$envFile = "deploy/.env"
$envExample = "deploy/.env.example"
if (-not (Test-Path -Path $envFile -PathType Leaf)) {
    if (Test-Path -Path $envExample -PathType Leaf) {
        Write-Output "Notice: $envFile not found. Initializing from $envExample..."
        Copy-Item -Path $envExample -Destination $envFile
        Write-Output "Created $envFile with default development settings."
    } else {
        Write-Warning "$envExample not found. Containers may use built-in defaults."
    }
}

# 2. Check 30 GB Storage Ceiling
if (-not $SkipResourceCheck) {
    Write-Output "Verifying resource budget and 30 GB container storage ceiling..."
    $checkScript = "scripts/check-resources.ps1"
    if (Test-Path -Path $checkScript -PathType Leaf) {
        powershell -ExecutionPolicy Bypass -File $checkScript -Quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Error "Storage ceiling check failed! Run scripts/clean-caches.ps1 or inspect with scripts/check-resources.ps1."
            exit $LASTEXITCODE
        }
        Write-Output "Storage budget verified (< 30 GB)."
    }
}

# 3. Resolve Compose File
$composeFile = "deploy/podman-compose.yml"

if (-not (Test-Path -Path $composeFile -PathType Leaf)) {
    throw "Compose configuration file not found at $composeFile"
}

# 4. Assemble Arguments
$cmdArgs = @("-f", $composeFile)
if ($DryRun) {
    $cmdArgs += "--dry-run"
}
$cmdArgs += "up"
if (-not $NoDetach) {
    $cmdArgs += "-d"
}
if ($Build) {
    $cmdArgs += "--build"
}
if ($Services.Count -gt 0) {
    $cmdArgs += $Services
}

Write-Output ("Executing compose: {0}" -f ($cmdArgs -join " "))

if ($DryRun) {
    Write-Output "`n[DRY RUN] Command formulated successfully. No containers started."
    exit 0
}

# 5. Launch Services
Invoke-Compose $cmdArgs
if ($LASTEXITCODE -ne 0) {
    throw "Compose up failed with exit code $LASTEXITCODE"
}

# 6. Wait for PostgreSQL Healthcheck if starting detached
$shouldCheckPostgres = ($Services.Count -eq 0) -or ($Services -contains "postgres")
if ($shouldCheckPostgres -and (-not $NoDetach)) {
    Write-Output "Waiting for PostgreSQL service to become healthy..."
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

Write-Output "------------------------------------------------------------------------"
Write-Output "Development Services Endpoints:"
Write-Output "  - Web Frontend   : http://localhost:5173"
Write-Output "  - API Service    : http://localhost:8080 (Health: http://localhost:8080/health)"
Write-Output "  - Browser Worker : http://localhost:3000"
Write-Output "  - PostgreSQL     : localhost:5432 (DB: webrepotodev, User: webrepouser)"
Write-Output "------------------------------------------------------------------------"
Write-Output "Useful lifecycle commands:"
Write-Output "  - Stream logs   : pwsh scripts/dev-logs.ps1 -Follow"
Write-Output "  - Check status  : pwsh scripts/dev-status.ps1"
Write-Output "  - Stop services : pwsh scripts/dev-down.ps1"
Write-Output "========================================================================"

exit 0
