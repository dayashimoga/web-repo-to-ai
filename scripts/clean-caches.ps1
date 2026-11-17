<#
.SYNOPSIS
    Safely prunes Podman build caches and dangling layers to respect the 30 GB ceiling.

.DESCRIPTION
    Safely cleans Podman build cache, stopped build containers, and dangling untagged images.
    Mandated by GLOBAL_ENGINEERING_CONTRACT.md and Rule 08 (08-deployment.md).
    NEVER deletes active containers, persistent database volumes, or repository source files by default.

.PARAMETER DryRun
    Simulates cache cleanup and reports reclaimable space without deleting anything.

.PARAMETER BuildCacheOnly
    Restricts pruning solely to persistent build cache (--mount=type=cache).

.PARAMETER IncludeVolumes
    CAUTION: Prunes unused local volumes. Never run by default. Requires interactive confirmation or -Force.

.PARAMETER Force
    Skips interactive confirmation when -IncludeVolumes is requested.

.PARAMETER Quiet
    Suppresses verbose progress output.

.EXAMPLE
    pwsh scripts/clean-caches.ps1 -DryRun
    pwsh scripts/clean-caches.ps1
    pwsh scripts/clean-caches.ps1 -BuildCacheOnly
#>

param(
    [switch]$DryRun,
    [switch]$BuildCacheOnly,
    [switch]$IncludeVolumes,
    [switch]$Force,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

function Format-Bytes {
    param([double]$Bytes)
    if ($Bytes -ge 1GB) {
        return "$([math]::Round($Bytes / 1GB, 2)) GB"
    } elseif ($Bytes -ge 1MB) {
        return "$([math]::Round($Bytes / 1MB, 2)) MB"
    } elseif ($Bytes -ge 1KB) {
        return "$([math]::Round($Bytes / 1KB, 2)) KB"
    } else {
        return "$([math]::Round($Bytes, 0)) B"
    }
}

function Get-PodmanStorageBytes {
    $podmanCmd = Get-Command "podman" -ErrorAction SilentlyContinue
    if (-not $podmanCmd) { return 0 }
    try {
        $dfOut = & podman system df --format json 2>&1
        if ($LASTEXITCODE -eq 0 -and $dfOut) {
            $items = $dfOut | ConvertFrom-Json
            $total = 0
            foreach ($it in $items) {
                if ($null -ne $it.RawSize) { $total += [double]$it.RawSize }
            }
            return $total
        }
    } catch {}
    return 0
}

# Verify Podman availability
$podmanCmd = Get-Command "podman" -ErrorAction SilentlyContinue
if (-not $podmanCmd) {
    Write-Warning "Podman is not installed or not in PATH. Nothing to clean."
    exit 0
}

if (-not $Quiet) {
    Write-Output "========================================================================"
    Write-Output "          web-repo-to-ai Safe Podman Cache Pruning Utility             "
    Write-Output "========================================================================"
}

$initialBytes = Get-PodmanStorageBytes
if (-not $Quiet) {
    Write-Output ("Initial Podman Storage Usage: {0}" -f (Format-Bytes $initialBytes))
}

# Dry Run Inspection
if ($DryRun) {
    Write-Output "`n[DRY RUN] Inspecting reclaimable Podman space without deletion..."
    try {
        & podman system df
    } catch {
        Write-Warning "Failed to run podman system df: $_"
    }
    Write-Output "`n[DRY RUN] Actions that would be performed during normal execution:"
    Write-Output "  1. podman image prune --build-cache -f (prune persistent build cache)"
    if (-not $BuildCacheOnly) {
        Write-Output "  2. podman system prune --build -f       (prune intermediate build containers)"
        Write-Output "  3. podman image prune -f                (prune dangling untagged images)"
    }
    if ($IncludeVolumes) {
        Write-Output "  4. podman volume prune -f               (CAUTION: would prune unused volumes)"
    } else {
        Write-Output "  *  Persistent database volumes (e.g. postgres_data) are PROTECTED and retained."
    }
    Write-Output "`n[DRY RUN] No changes were made."
    exit 0
}

# Guard Volume Pruning
if ($IncludeVolumes) {
    if (-not $Force) {
        Write-Warning "You specified -IncludeVolumes. This will remove unused local container volumes."
        Write-Warning "If your development database volume is not currently in use, its state will be LOST!"
        $confirm = Read-Host "Are you absolutely sure you want to prune volumes? (yes/NO)"
        if ($confirm -ne "yes") {
            Write-Output "Volume pruning aborted by user."
            $IncludeVolumes = $false
        }
    }
}

# Execute Pruning Steps
Write-Output "`nExecuting safe cache pruning..."

# Step 1: Prune persistent build cache
Write-Output "  -> Pruning Podman build cache..."
try {
    $null = & podman image prune --build-cache -f 2>&1
    Write-Output "     [OK] Build cache pruned."
} catch {
    Write-Warning "     [SKIP] Build cache prune failed or not supported: $_"
}

if (-not $BuildCacheOnly) {
    # Step 2: Prune build containers
    Write-Output "  -> Pruning stopped build containers..."
    try {
        $null = & podman system prune --build -f 2>&1
        Write-Output "     [OK] Build containers pruned."
    } catch {
        Write-Warning "     [SKIP] Build container prune failed: $_"
    }

    # Step 3: Prune dangling images (untagged layers)
    Write-Output "  -> Pruning dangling (untagged) image layers..."
    try {
        $null = & podman image prune -f 2>&1
        Write-Output "     [OK] Dangling images pruned."
    } catch {
        Write-Warning "     [SKIP] Dangling image prune failed: $_"
    }
}

# Step 4: Optional volume prune (guarded)
if ($IncludeVolumes) {
    Write-Output "  -> Pruning unused volumes (explicitly requested)..."
    try {
        $null = & podman volume prune -f 2>&1
        Write-Output "     [OK] Unused volumes pruned."
    } catch {
        Write-Warning "     [SKIP] Volume prune failed: $_"
    }
}

# Report Reclaimed Space
$finalBytes = Get-PodmanStorageBytes
$reclaimedBytes = [math]::Max(0, ($initialBytes - $finalBytes))

if (-not $Quiet) {
    Write-Output "------------------------------------------------------------------------"
    Write-Output ("Final Podman Storage Usage  : {0}" -f (Format-Bytes $finalBytes))
    Write-Output ("Total Storage Reclaimed     : {0}" -f (Format-Bytes $reclaimedBytes))
    Write-Output "------------------------------------------------------------------------"
    Write-Output "Running post-cleanup resource ceiling verification:"
    & powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -Quiet
    Write-Output "========================================================================"
} else {
    Write-Output ("Reclaimed: {0}, Final Usage: {1}" -f (Format-Bytes $reclaimedBytes), (Format-Bytes $finalBytes))
}

exit 0
