<#
.SYNOPSIS
    Universal Automation Script to start web-repo-to-ai via Direct (native), Podman, or Hybrid modes.

.DESCRIPTION
    Provides a fully automated one-command lifecycle manager for web-repo-to-ai:
    - Direct Mode: Runs native Rust API server and SvelteKit Web UI directly on host (fastest, < 2s startup).
    - Podman Mode: Runs the complete containerized stack (Postgres, Redis, API, Worker, Web) via Podman Compose.
    - Hybrid Mode: Runs PostgreSQL container in Podman while running API and Web natively for rapid dev with real persistence.
    - Status: Inspects running processes, ports, and healthcheck endpoints.
    - Stop: Gracefully terminates all host processes and/or container services.

.PARAMETER Mode
    Execution mode: 'Direct' (default), 'Podman', or 'Hybrid'.

.PARAMETER OpenBrowser
    Automatically opens http://localhost:5173 in the default web browser once healthy.

.PARAMETER Stop
    Gracefully stops all running services across native processes and Podman containers.

.PARAMETER Status
    Displays live status and health for all components without modifying state.

.PARAMETER Build
    (Podman mode only) Forces rebuilding of container images.

.PARAMETER ApiPort
    Port for the Rust API server (default: 3000 in Direct/Hybrid, 8080 in Podman).

.PARAMETER WebPort
    Port for the SvelteKit frontend (default: 5173).

.EXAMPLE
    # Start everything natively (fastest development mode)
    powershell -ExecutionPolicy Bypass -File scripts/start-all.ps1

    # Start everything using Podman containers
    powershell -ExecutionPolicy Bypass -File scripts/start-all.ps1 -Mode Podman

    # Start PostgreSQL in Podman and API/Web natively
    powershell -ExecutionPolicy Bypass -File scripts/start-all.ps1 -Mode Hybrid

    # Check status of all services
    powershell -ExecutionPolicy Bypass -File scripts/start-all.ps1 -Status

    # Stop all running services
    powershell -ExecutionPolicy Bypass -File scripts/start-all.ps1 -Stop
#>

[CmdletBinding(DefaultParameterSetName = "Start")]
param(
    [Parameter(ParameterSetName = "Start")]
    [ValidateSet("Direct", "Podman", "Hybrid")]
    [string]$Mode = "Direct",

    [Parameter(ParameterSetName = "Start")]
    [switch]$NoBrowser,

    [Parameter(ParameterSetName = "Start")]
    [switch]$Detach,

    [Parameter(ParameterSetName = "Start")]
    [switch]$Build,

    [Parameter(ParameterSetName = "Stop", Mandatory = $true)]
    [switch]$Stop,

    [Parameter(ParameterSetName = "Status", Mandatory = $true)]
    [switch]$Status,

    [Parameter(ParameterSetName = "Start")]
    [int]$ApiPort = 0,

    [Parameter(ParameterSetName = "Start")]
    [int]$WebPort = 5173
)

$ErrorActionPreference = "Stop"

# Resolve root workspace path
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$rootDir = (Resolve-Path (Join-Path $scriptDir "..")).Path
Set-Location $rootDir

# Color styling helpers
function Write-Header([string]$text) {
    Write-Host "`n========================================================================" -ForegroundColor Cyan
    Write-Host "  $text" -ForegroundColor White
    Write-Host "========================================================================" -ForegroundColor Cyan
}

function Write-Step([string]$step, [string]$detail) {
    Write-Host "[$step] " -ForegroundColor Yellow -NoNewline
    Write-Host $detail -ForegroundColor White
}

function Write-Success([string]$msg) {
    Write-Host " [OK] " -ForegroundColor Green -NoNewline
    Write-Host $msg -ForegroundColor White
}

function Write-Fail([string]$msg) {
    Write-Host " [FAIL] " -ForegroundColor Red -NoNewline
    Write-Host $msg -ForegroundColor Yellow
}

# Process and Port Helpers
function Find-ProcessesByPort([int]$port) {
    try {
        $conns = Get-NetTCPConnection -LocalPort $port -State Listen -ErrorAction SilentlyContinue
        if ($conns) {
            $pids = $conns | Select-Object -ExpandProperty OwningProcess -Unique
            return Get-Process -Id $pids -ErrorAction SilentlyContinue
        }
    } catch {}
    return @()
}

function Stop-AllServices() {
    Write-Header "Stopping all web-repo-to-ai processes and containers"

    # 1. Stop host processes listening on default ports
    foreach ($p in @(5173, 5174, 5175, 5176, 5177, 5178, 3000, 8080)) {
        $procs = Find-ProcessesByPort $p
        foreach ($proc in $procs) {
            Write-Step "KILL" "Stopping process $($proc.ProcessName) (PID: $($proc.Id)) on port $p"
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }

    # 2. Stop node/vite/cargo background processes by name
    $namedProcs = Get-Process -ErrorAction SilentlyContinue | Where-Object {
        $_.ProcessName -match "web-repo-to-ai-api" -or
        ($_.ProcessName -eq "node" -and $_.CommandLine -like "*vite*")
    }
    foreach ($proc in $namedProcs) {
        Write-Step "KILL" "Stopping lingering process $($proc.ProcessName) (PID: $($proc.Id))"
        Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
    }

    # 3. Stop Podman Compose services if running
    if (Get-Command "podman-compose" -ErrorAction SilentlyContinue) {
        if (Test-Path "deploy/podman-compose.yml") {
            Write-Step "PODMAN" "Stopping podman-compose containers..."
            try {
                & podman-compose -f deploy/podman-compose.yml down 2>&1 | Out-Null
                Write-Success "Podman containers stopped."
            } catch {
                Write-Host "Podman down notice: $($_.Exception.Message)" -ForegroundColor DarkGray
            }
        }
    }

    Write-Success "All web-repo-to-ai services stopped cleanly."
}

function Show-Status() {
    Write-Header "web-repo-to-ai System Status"

    # Check Direct Mode Ports
    $checks = @(
        @{ Name = "Frontend UI (Direct/Podman)"; Port = 5173; Url = "http://localhost:5173" },
        @{ Name = "Rust API Server (Direct)"; Port = 3000; Url = "http://127.0.0.1:3000/api/v1/health" },
        @{ Name = "Rust API Server (Podman)"; Port = 8080; Url = "http://127.0.0.1:8080/api/v1/health" },
        @{ Name = "PostgreSQL (Podman/Direct)"; Port = 5432; Url = $null }
    )

    foreach ($chk in $checks) {
        $procs = Find-ProcessesByPort $chk.Port
        $statusStr = "STOPPED"
        $color = "DarkGray"

        if ($procs.Count -gt 0) {
            $statusStr = "LISTENING (PID: $(($procs.Id) -join ', '))"
            $color = "Green"

            if ($chk.Url) {
                try {
                    $res = Invoke-WebRequest -Uri $chk.Url -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
                    if ($res.StatusCode -eq 200) {
                        $statusStr += " - HEALTHY (HTTP 200)"
                    }
                } catch {
                    $statusStr += " - CONNECTED (HTTP $($_.Exception.Response.StatusCode.value__))"
                }
            }
        }

        Write-Host ("{0,-32} [Port {1,5}] : " -f $chk.Name, $chk.Port) -NoNewline
        Write-Host $statusStr -ForegroundColor $color
    }

    # Podman Containers
    if (Get-Command "podman" -ErrorAction SilentlyContinue) {
        Write-Host "`nPodman Containers:" -ForegroundColor Cyan
        try {
            & podman ps --filter "name=web-repo" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
        } catch {
            Write-Host "No Podman daemon reachable or no containers running." -ForegroundColor DarkGray
        }
    }
}

# --- Action Dispatch ---

if ($Stop) {
    Stop-AllServices
    exit 0
}

if ($Status) {
    Show-Status
    exit 0
}

# --- Start Action ---

if ($ApiPort -eq 0) {
    $ApiPort = if ($Mode -eq "Podman") { 8080 } else { 3000 }
}

Write-Header "Launching web-repo-to-ai [$Mode Mode]"

switch ($Mode) {
    "Direct" {
        # 1. Clean up stale port occupants if any
        foreach ($p in @($ApiPort, $WebPort)) {
            $procs = Find-ProcessesByPort $p
            if ($procs.Count -gt 0) {
                Write-Step "CLEANUP" "Freeing occupied port $p (stopping PID: $(($procs.Id) -join ', '))..."
                $procs | Stop-Process -Force -ErrorAction SilentlyContinue
                Start-Sleep -Milliseconds 500
            }
        }

        # 2. Start Rust API Server
        Write-Step "API" "Starting Rust backend API server (port $ApiPort)..."
        $apiCmd = "Set-Location '$rootDir'; cargo run -p web-repo-to-ai-api"
        $apiProc = Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "`$host.ui.RawUI.WindowTitle = 'web-repo-to-ai API Server'; $apiCmd" -PassThru

        # Wait for API health check
        Write-Step "PROBE" "Waiting for API server to become ready on http://127.0.0.1:$ApiPort/api/v1/health..."
        $apiReady = $false
        for ($i = 0; $i -lt 40; $i++) {
            Start-Sleep -Milliseconds 800
            try {
                $res = Invoke-RestMethod -Uri "http://127.0.0.1:$ApiPort/api/v1/health" -TimeoutSec 2 -ErrorAction Stop
                if ($res.status -eq "pass") {
                    $apiReady = $true
                    break
                }
            } catch {}
        }

        if (-not $apiReady) {
            Write-Fail "API server did not respond in time. Please check the 'web-repo-to-ai API Server' PowerShell window for errors."
            exit 1
        }
        Write-Success "API Server is healthy and listening on http://127.0.0.1:$ApiPort (PID: $($apiProc.Id))"

        # 3. Start SvelteKit Web UI
        Write-Step "WEB" "Starting SvelteKit frontend (port $WebPort)..."
        $webDir = Join-Path $rootDir "apps/web"
        $webCmd = "Set-Location '$webDir'; npm run dev -- --port $WebPort"
        $webProc = Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "`$host.ui.RawUI.WindowTitle = 'web-repo-to-ai Web UI'; $webCmd" -PassThru

        # Wait for Web health check
        Write-Step "PROBE" "Waiting for Web UI to become ready on http://localhost:$WebPort/..."
        $webReady = $false
        for ($i = 0; $i -lt 25; $i++) {
            Start-Sleep -Milliseconds 800
            try {
                $res = Invoke-WebRequest -Uri "http://localhost:$WebPort/" -UseBasicParsing -TimeoutSec 2 -ErrorAction Stop
                if ($res.StatusCode -eq 200) {
                    $webReady = $true
                    break
                }
            } catch {}
        }

        if (-not $webReady) {
            Write-Fail "Web UI did not respond in time. Please check the 'web-repo-to-ai Web UI' window for errors."
            exit 1
        }
        Write-Success "Web UI is ready on http://localhost:$WebPort/ (PID: $($webProc.Id))"
    }

    "Podman" {
        # Check podman machine & compose
        if (-not (Get-Command "podman-compose" -ErrorAction SilentlyContinue)) {
            Write-Fail "podman-compose is not installed or not in PATH."
            exit 1
        }

        # Ensure deploy/.env exists
        if (-not (Test-Path "deploy/.env")) {
            Copy-Item "deploy/.env.example" "deploy/.env"
            Write-Step "ENV" "Created deploy/.env from template."
        }

        Write-Step "PODMAN" "Bootstrapping full containerized stack via podman-compose..."
        $composeArgs = @("-f", "deploy/podman-compose.yml", "up", "-d")
        if ($Build) {
            $composeArgs += "--build"
        }

        & podman-compose @composeArgs
        if ($LASTEXITCODE -ne 0) {
            Write-Fail "podman-compose up returned error exit code $LASTEXITCODE"
            exit $LASTEXITCODE
        }

        Write-Step "PROBE" "Waiting for services to become healthy..."
        Start-Sleep -Seconds 5
        Write-Success "Podman container stack launched."
    }

    "Hybrid" {
        # Start Postgres in container, API & Web natively
        Write-Step "HYBRID" "Starting PostgreSQL in Podman..."
        & podman-compose -f deploy/podman-compose.yml up -d postgres
        Start-Sleep -Seconds 3

        Write-Step "API" "Starting API Server pointing to PostgreSQL..."
        $apiCmd = "Set-Location '$rootDir'; `$env:DATABASE_URL='postgresql://webrepouser:devpassword123@127.0.0.1:5432/webrepotodev'; cargo run -p web-repo-to-ai-api"
        Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "`$host.ui.RawUI.WindowTitle = 'web-repo-to-ai API (Hybrid)'; $apiCmd"

        Write-Step "WEB" "Starting SvelteKit Web UI..."
        $webDir = Join-Path $rootDir "apps/web"
        $webCmd = "Set-Location '$webDir'; npm run dev -- --port $WebPort"
        Start-Process -FilePath "powershell.exe" -ArgumentList "-NoExit", "-Command", "`$host.ui.RawUI.WindowTitle = 'web-repo-to-ai Web UI'; $webCmd"

        Start-Sleep -Seconds 4
        Write-Success "Hybrid stack launched."
    }
}

# Open Browser unless -NoBrowser was specified
if (-not $NoBrowser) {
    Write-Step "BROWSER" "Opening http://localhost:$WebPort/ in default browser..."
    Start-Process "http://localhost:$WebPort/"
}

Write-Header "web-repo-to-ai is RUNNING & READY!"
Write-Host "  * Interactive Web UI  : " -NoNewline -ForegroundColor White
Write-Host "http://localhost:$WebPort/" -ForegroundColor Green
Write-Host "  * Backend REST API    : " -NoNewline -ForegroundColor White
Write-Host "http://127.0.0.1:$ApiPort/api/v1" -ForegroundColor Green
Write-Host "  * Health Check URL    : " -NoNewline -ForegroundColor White
Write-Host "http://127.0.0.1:$ApiPort/api/v1/health" -ForegroundColor Green
Write-Host "  * Interactive Docs    : " -NoNewline -ForegroundColor White
Write-Host "http://127.0.0.1:$ApiPort/api/v1/docs" -ForegroundColor Green
Write-Host "------------------------------------------------------------------------" -ForegroundColor Cyan
Write-Host "Useful Control Commands:" -ForegroundColor Yellow
Write-Host "  powershell scripts/start-all.ps1 -Status   # Check health and running ports" -ForegroundColor Gray
Write-Host "  powershell scripts/start-all.ps1 -Stop     # Stop all servers and containers" -ForegroundColor Gray
Write-Host "========================================================================`n" -ForegroundColor Cyan

# If not detached and not Podman mode, monitor in foreground until Ctrl+C
if (-not $Detach -and $Mode -ne "Podman") {
    Write-Host "Running in foreground mode. Press [Ctrl + C] in this window to stop all services." -ForegroundColor Yellow
    try {
        while ($true) {
            Start-Sleep -Seconds 2
            if ($apiProc -and $apiProc.HasExited) {
                Write-Fail "API Server process (PID: $($apiProc.Id)) exited."
                break
            }
            if ($webProc -and $webProc.HasExited) {
                Write-Fail "Web UI process (PID: $($webProc.Id)) exited."
                break
            }
        }
    } finally {
        Write-Host "`nStopping services..." -ForegroundColor Yellow
        Stop-AllServices
    }
}
