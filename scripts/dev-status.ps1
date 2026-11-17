<#
.SYNOPSIS
    Reports the status of containerized development services for web-repo-to-ai.

.DESCRIPTION
    Inspects container runtime status, healthcheck state, exposed network ports,
    and storage budget compliance across all services in deploy/podman-compose.yml.

.PARAMETER Json
    Outputs structured JSON for automated pipelines and monitoring.

.PARAMETER Quiet
    Outputs compact single-line status summaries.

.EXAMPLE
    pwsh scripts/dev-status.ps1
    pwsh scripts/dev-status.ps1 -Json
    pwsh scripts/dev-status.ps1 -Quiet
#>

param(
    [switch]$Json,
    [switch]$Quiet
)

$ErrorActionPreference = "Stop"

$expectedServices = @(
    @{ Name = "postgres"; Container = "web-repo-postgres"; Port = "5432"; Desc = "PostgreSQL 16 Database" },
    @{ Name = "api"; Container = "web-repo-api"; Port = "8080"; Desc = "Rust Axum API Service" },
    @{ Name = "worker"; Container = "web-repo-worker"; Port = "Internal"; Desc = "Rust Processing Worker" },
    @{ Name = "browser-worker"; Container = "web-repo-browser-worker"; Port = "3000"; Desc = "Playwright Browser Worker" },
    @{ Name = "web"; Container = "web-repo-web"; Port = "5173"; Desc = "SvelteKit Web Frontend" }
)

$statusList = @()
$runningCount = 0

foreach ($svc in $expectedServices) {
    $cName = $svc.Container
    $state = "NOT_CREATED"
    $health = "N/A"
    $uptime = "N/A"
    $ports = $svc.Port

    try {
        $jsonOut = & podman inspect --format "{{json .State}}" $cName 2>$null
        if ($LASTEXITCODE -eq 0 -and $jsonOut) {
            $stateObj = $jsonOut | ConvertFrom-Json
            if ($stateObj.Running) {
                $state = "RUNNING"
                $runningCount++
                $uptime = $stateObj.StartedAt
            } else {
                $state = "STOPPED ($($stateObj.Status))"
            }

            if ($null -ne $stateObj.Health) {
                $health = $stateObj.Health.Status
            }
        }
    } catch {}

    $statusList += [PSCustomObject]@{
        Service   = $svc.Name
        Container = $svc.Container
        State     = $state
        Health    = $health
        Port      = $ports
        Description = $svc.Desc
    }
}

# Check Storage Ceiling
$storageInfo = "N/A"
$storageStatus = "UNKNOWN"
try {
    $resScript = "scripts/check-resources.ps1"
    if (Test-Path -Path $resScript -PathType Leaf) {
        $resJson = powershell -ExecutionPolicy Bypass -File $resScript -Json | ConvertFrom-Json
        if ($resJson) {
            $storageInfo = "$([math]::Round($resJson.UsedBytes / 1GB, 2)) GB / $($resJson.CeilingGB) GB ($($resJson.UsedPercent)%)"
            $storageStatus = $resJson.Status
        }
    }
} catch {}

if ($Json) {
    $result = [PSCustomObject]@{
        Timestamp     = (Get-Date).ToUniversalTime().ToString("o")
        TotalServices = $expectedServices.Count
        RunningCount  = $runningCount
        StorageBudget = [PSCustomObject]@{
            Usage  = $storageInfo
            Status = $storageStatus
        }
        Services      = $statusList
    }
    $result | ConvertTo-Json -Depth 4
    exit 0
}

if (-not $Quiet) {
    Write-Output "========================================================================"
    Write-Output "       web-repo-to-ai Development Services Status & Health             "
    Write-Output "========================================================================"
    Write-Output ("Running Services : {0} / {1}" -f $runningCount, $expectedServices.Count)
    Write-Output ("Storage Footprint: {0} [{1}]" -f $storageInfo, $storageStatus)
    Write-Output "------------------------------------------------------------------------"
    Write-Output ("{0,-16} {1,-26} {2,-16} {3,-10} {4}" -f "SERVICE", "CONTAINER", "STATE", "HEALTH", "PORT")
    Write-Output ("{0,-16} {1,-26} {2,-16} {3,-10} {4}" -f "-------", "---------", "-----", "------", "----")
    foreach ($s in $statusList) {
        Write-Output ("{0,-16} {1,-26} {2,-16} {3,-10} {4}" -f $s.Service, $s.Container, $s.State, $s.Health, $s.Port)
    }
    Write-Output "========================================================================"
} else {
    Write-Output ("Running: {0}/{1}, Storage: {2} [{3}]" -f $runningCount, $expectedServices.Count, $storageInfo, $storageStatus)
}

exit 0
