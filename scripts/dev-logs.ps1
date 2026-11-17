<#
.SYNOPSIS
    Displays and follows logs from containerized development services for web-repo-to-ai.

.DESCRIPTION
    Streams logs from services running via deploy/podman-compose.yml.
    Supports filtering by service, tailing lines, and real-time streaming.

.PARAMETER Service
    Optional service name to filter logs (e.g. 'postgres', 'api', 'worker', 'browser-worker', 'web').
    Defaults to all services.

.PARAMETER Follow
    Follows log output in real-time.

.PARAMETER Tail
    Number of lines to show from the end of the logs (default: 50).

.PARAMETER Timestamps
    Shows timestamps in the log output.

.EXAMPLE
    pwsh scripts/dev-logs.ps1
    pwsh scripts/dev-logs.ps1 -Service api
    pwsh scripts/dev-logs.ps1 -Follow
    pwsh scripts/dev-logs.ps1 -Service postgres -Tail 100
#>

param(
    [string]$Service,
    [switch]$Follow,
    [int]$Tail = 50,
    [switch]$Timestamps
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

$composeFile = "deploy/podman-compose.yml"
if (-not (Test-Path -Path $composeFile -PathType Leaf)) {
    throw "Compose configuration file not found at $composeFile"
}

$cmdArgs = @("-f", $composeFile, "logs", "--tail", $Tail)
if ($Follow) {
    $cmdArgs += "-f"
}
if ($Timestamps) {
    $cmdArgs += "-t"
}
if ($Service) {
    $cmdArgs += $Service
}

Write-Output ("Streaming logs via compose: {0}" -f ($cmdArgs -join " "))
Invoke-Compose $cmdArgs
exit $LASTEXITCODE
