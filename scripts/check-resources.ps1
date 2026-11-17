<#
.SYNOPSIS
    Monitors system resources and enforces the 30 GB Podman storage ceiling.

.DESCRIPTION
    Inspects host CPU, RAM, disk space, and Podman container/image/volume storage usage.
    Enforces the strict 30 GB container storage ceiling mandated by GLOBAL_ENGINEERING_CONTRACT.md
    and Rule 08 (08-deployment.md). Warns when storage reaches the warning threshold (default 80%)
    and exits with a non-zero code if the 30 GB ceiling is breached.

.PARAMETER CeilingGB
    Maximum allowable Podman storage in GB (default: 30.0).

.PARAMETER WarnThresholdPercent
    Percentage of the ceiling that triggers a warning (default: 80.0%).

.PARAMETER Json
    Emits structured JSON output for automated ingestion and CI runners.

.PARAMETER Quiet
    Suppresses verbose human-readable output, returning only the status code and summary.

.PARAMETER FailOnWarn
    Causes the script to exit with code 1 if the warning threshold is reached.

.EXAMPLE
    pwsh scripts/check-resources.ps1
    pwsh scripts/check-resources.ps1 -CeilingGB 30 -Json
    pwsh scripts/check-resources.ps1 -FailOnWarn
#>

param(
    [double]$CeilingGB = 30.0,
    [double]$WarnThresholdPercent = 80.0,
    [switch]$Json,
    [switch]$Quiet,
    [switch]$FailOnWarn
)

$ErrorActionPreference = "Stop"

# Helper for byte unit conversions
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

# 1. Inspect Host System Resources
$cpuCores = [Environment]::ProcessorCount
$totalMemBytes = 0
$freeMemBytes = 0

if ($IsWindows -or $env:OS -like "*Windows*") {
    try {
        $osInfo = Get-CimInstance -ClassName Win32_OperatingSystem -ErrorAction SilentlyContinue
        if ($osInfo) {
            $totalMemBytes = [double]$osInfo.TotalVisibleMemorySize * 1KB
            $freeMemBytes = [double]$osInfo.FreePhysicalMemory * 1KB
        }
    } catch {
        # Fallback if CimInstance fails
        $totalMemBytes = 16GB
        $freeMemBytes = 8GB
    }
} elseif (Test-Path -Path "/proc/meminfo" -PathType Leaf) {
    try {
        $meminfo = Get-Content "/proc/meminfo" -Raw
        if ($meminfo -match "MemTotal:\s+(\d+)\s+kB") { $totalMemBytes = [double]$matches[1] * 1KB }
        if ($meminfo -match "MemAvailable:\s+(\d+)\s+kB") { $freeMemBytes = [double]$matches[1] * 1KB }
    } catch {}
}

# Inspect Current Drive Free Space
$driveLetter = "N/A"
$driveFreeBytes = 0
$driveTotalBytes = 0
try {
    $currentDrive = (Get-Item (Get-Location) -Force).PSDrive
    if ($currentDrive) {
        $driveLetter = $currentDrive.Name
        $driveFreeBytes = [double]$currentDrive.Free
        $driveTotalBytes = [double]$currentDrive.Used + [double]$currentDrive.Free
    }
} catch {}

# 2. Inspect Podman Storage and Machine
$podmanCmd = Get-Command "podman" -ErrorAction SilentlyContinue
$podmanInstalled = ($null -ne $podmanCmd)
$podmanVersion = "N/A"
$machineName = "N/A"
$machineDiskSizeBytes = 0
$podmanUsedBytes = 0
$podmanReclaimableBytes = 0
$breakdownList = @()

if ($podmanInstalled) {
    try {
        $vOut = & podman --version 2>&1
        if ($vOut -match "version\s+([0-9\.]+)") {
            $podmanVersion = $matches[1]
        } else {
            $podmanVersion = $vOut.ToString().Trim()
        }
    } catch {}

    # Inspect Podman Machine (Windows / macOS)
    try {
        $machineOut = & podman machine list --format json 2>&1
        if ($LASTEXITCODE -eq 0 -and $machineOut) {
            $machines = $machineOut | ConvertFrom-Json
            if ($machines -and $machines.Count -gt 0) {
                $defaultMachine = $machines | Where-Object { $_.Default -eq $true } | Select-Object -First 1
                if (-not $defaultMachine) { $defaultMachine = $machines[0] }
                $machineName = $defaultMachine.Name
                if ($defaultMachine.DiskSize) {
                    $machineDiskSizeBytes = [double]$defaultMachine.DiskSize
                }
            }
        }
    } catch {}

    # Inspect Podman Storage Usage
    try {
        $dfOut = & podman system df --format json 2>&1
        if ($LASTEXITCODE -eq 0 -and $dfOut) {
            $dfItems = $dfOut | ConvertFrom-Json
            foreach ($item in $dfItems) {
                $rawSize = if ($null -ne $item.RawSize) { [double]$item.RawSize } else { 0 }
                $rawReclaim = if ($null -ne $item.RawReclaimable) { [double]$item.RawReclaimable } else { 0 }
                $podmanUsedBytes += $rawSize
                $podmanReclaimableBytes += $rawReclaim
                $breakdownList += [PSCustomObject]@{
                    Type        = $item.Type
                    TotalCount  = $item.TotalCount
                    ActiveCount = $item.Active
                    RawSize     = $rawSize
                    Size        = $item.Size
                    Reclaimable = $item.Reclaimable
                }
            }
        }
    } catch {
        Write-Warning "Failed to query podman system df: $_"
    }
}

# 3. Calculate Thresholds and Status
$ceilingBytes = $CeilingGB * 1GB
$warnThresholdBytes = $ceilingBytes * ($WarnThresholdPercent / 100.0)
$usedPercent = if ($ceilingBytes -gt 0) { [math]::Round(($podmanUsedBytes / $ceilingBytes) * 100.0, 2) } else { 0 }

$status = "HEALTHY"
$exitCode = 0
$statusMessage = ""

if (-not $podmanInstalled) {
    $status = "PODMAN_NOT_INSTALLED"
    $statusMessage = "Podman is not installed or not in PATH."
    $exitCode = 0
} elseif ($podmanUsedBytes -gt $ceilingBytes) {
    $status = "CRITICAL_CEILING_BREACH"
    $statusMessage = "Podman storage footprint ($(Format-Bytes $podmanUsedBytes)) EXCEEDS the $CeilingGB GB ceiling ($([math]::Round($usedPercent, 1))%)! Run scripts/clean-caches.ps1 immediately."
    $exitCode = 1
} elseif ($podmanUsedBytes -ge $warnThresholdBytes) {
    $status = "WARNING_STORAGE_HIGH"
    $statusMessage = "Podman storage footprint ($(Format-Bytes $podmanUsedBytes)) exceeds $WarnThresholdPercent% of $CeilingGB GB ceiling ($([math]::Round($usedPercent, 1))%). Run scripts/clean-caches.ps1 to prune caches."
    if ($FailOnWarn) { $exitCode = 1 } else { $exitCode = 0 }
} else {
    $status = "HEALTHY_WITHIN_BUDGET"
    $statusMessage = "Podman storage footprint ($(Format-Bytes $podmanUsedBytes) / $CeilingGB GB, $([math]::Round($usedPercent, 1))%) is within budget."
    $exitCode = 0
}

# 4. Output Generation
if ($Json) {
    $result = [PSCustomObject]@{
        Timestamp       = (Get-Date).ToUniversalTime().ToString("o")
        Status          = $status
        ExitCode        = $exitCode
        StatusMessage   = $statusMessage
        CeilingGB       = $CeilingGB
        CeilingBytes    = $ceilingBytes
        UsedBytes       = $podmanUsedBytes
        UsedPercent     = $usedPercent
        ReclaimableBytes= $podmanReclaimableBytes
        Host            = [PSCustomObject]@{
            CpuCores       = $cpuCores
            TotalMemBytes  = $totalMemBytes
            TotalMemFormatted = Format-Bytes $totalMemBytes
            FreeMemBytes   = $freeMemBytes
            FreeMemFormatted = Format-Bytes $freeMemBytes
            DriveLetter    = $driveLetter
            DriveFreeBytes = $driveFreeBytes
            DriveFreeFormatted = Format-Bytes $driveFreeBytes
            DriveTotalBytes= $driveTotalBytes
            DriveTotalFormatted = Format-Bytes $driveTotalBytes
        }
        Podman          = [PSCustomObject]@{
            Installed           = $podmanInstalled
            Version             = $podmanVersion
            MachineName         = $machineName
            MachineDiskSizeBytes= $machineDiskSizeBytes
            MachineDiskFormatted= Format-Bytes $machineDiskSizeBytes
            Breakdown           = $breakdownList
        }
    }
    $result | ConvertTo-Json -Depth 5
    exit $exitCode
}

if (-not $Quiet) {
    Write-Output "========================================================================"
    Write-Output "  web-repo-to-ai Resource Monitor & 30 GB Podman Storage Ceiling Check  "
    Write-Output "========================================================================"
    Write-Output ("Host CPU Cores     : {0}" -f $cpuCores)
    Write-Output ("Host RAM (Total/Free): {0} / {1}" -f (Format-Bytes $totalMemBytes), (Format-Bytes $freeMemBytes))
    if ($driveLetter -ne "N/A") {
        Write-Output ("Host Disk Space ({0}): {1} free / {2} total" -f $driveLetter, (Format-Bytes $driveFreeBytes), (Format-Bytes $driveTotalBytes))
    }
    Write-Output ("Podman Installed   : {0} ({1})" -f $podmanInstalled, $podmanVersion)
    if ($machineName -ne "N/A") {
        Write-Output ("Podman Machine     : {0} (Disk Cap: {1})" -f $machineName, (Format-Bytes $machineDiskSizeBytes))
    }
    Write-Output "------------------------------------------------------------------------"
    Write-Output "Podman Storage Usage Breakdown:"
    if ($breakdownList.Count -gt 0) {
        foreach ($b in $breakdownList) {
            Write-Output ("  - {0,-14} : {1,-10} (Count: {2}, Reclaimable: {3})" -f $b.Type, $b.Size, $b.TotalCount, $b.Reclaimable)
        }
    } else {
        Write-Output "  (No Podman storage items found or Podman not initialized)"
    }
    Write-Output "------------------------------------------------------------------------"
    Write-Output ("Total Podman Usage : {0} / {1} GB ({2}%)" -f (Format-Bytes $podmanUsedBytes), $CeilingGB, [math]::Round($usedPercent, 1))
    Write-Output ("Reclaimable Space  : {0}" -f (Format-Bytes $podmanReclaimableBytes))
    Write-Output ("Status             : [{0}]" -f $status)
    Write-Output ("Message            : {0}" -f $statusMessage)
    Write-Output "========================================================================"
} else {
    Write-Output ("[{0}] {1}" -f $status, $statusMessage)
}

exit $exitCode
