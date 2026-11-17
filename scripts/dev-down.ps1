<#
.SYNOPSIS
    Stops and tears down the containerized development environment for web-repo-to-ai.

.DESCRIPTION
    Stops running development services and networks defined in deploy/podman-compose.yml.
    Protects persistent database volumes by default. Supports optional volume removal
    with explicit confirmation and optional build cache pruning.

.PARAMETER Volumes
    Removes named persistent volumes (postgres_data, artifact_data). Prompts for confirmation unless -Force is passed.

.PARAMETER Force
    Skips interactive confirmation when -Volumes is specified.

.PARAMETER PruneCaches
    Runs scripts/clean-caches.ps1 after teardown to reclaim storage and respect the 30 GB ceiling.

.PARAMETER DryRun
    Prints the compose down command without stopping containers.

.EXAMPLE
    pwsh scripts/dev-down.ps1
    pwsh scripts/dev-down.ps1 -Volumes
    pwsh scripts/dev-down.ps1 -PruneCaches
    pwsh scripts/dev-down.ps1 -DryRun
#>

param(
    [switch]$Volumes,
    [switch]$Force,
    [switch]$PruneCaches,
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
Write-Output "           web-repo-to-ai Development Teardown Utility                  "
Write-Output "========================================================================"

# Volume confirmation guard
if ($Volumes) {
    if (-not $Force) {
        Write-Warning "CAUTION: You specified -Volumes."
        Write-Warning "This will permanently delete the local PostgreSQL database (postgres_data) and cached artifacts (artifact_data)!"
        $confirm = Read-Host "Are you absolutely sure you want to delete persistent volumes? (yes/NO)"
        if ($confirm -ne "yes") {
            Write-Output "Volume deletion aborted by user. Retaining persistent data volumes."
            $Volumes = $false
        }
    }
}

$composeFile = "deploy/podman-compose.yml"

if (-not (Test-Path -Path $composeFile -PathType Leaf)) {
    throw "Compose configuration file not found at $composeFile"
}

$cmdArgs = @("-f", $composeFile)
if ($DryRun) {
    $cmdArgs += "--dry-run"
}
$cmdArgs += "down"
if ($Volumes) {
    $cmdArgs += "-v"
}

Write-Output ("Executing compose: {0}" -f ($cmdArgs -join " "))

if ($DryRun) {
    Write-Output "`n[DRY RUN] Command formulated successfully. No containers stopped."
    if ($PruneCaches) {
        Write-Output "[DRY RUN] Would invoke scripts/clean-caches.ps1 after teardown."
    }
    exit 0
}

# Execute Compose Down
Invoke-Compose $cmdArgs
if ($LASTEXITCODE -ne 0) {
    throw "Compose down failed with exit code $LASTEXITCODE"
}

Write-Output "Development services stopped successfully."

# Optional Cache Pruning
if ($PruneCaches) {
    Write-Output "`nExecuting post-teardown cache pruning..."
    $cleanScript = "scripts/clean-caches.ps1"
    if (Test-Path -Path $cleanScript -PathType Leaf) {
        powershell -ExecutionPolicy Bypass -File $cleanScript
    }
}

Write-Output "========================================================================"
exit 0
