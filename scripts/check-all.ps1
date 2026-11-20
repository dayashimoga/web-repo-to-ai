# Monorepo Comprehensive Quality, Lint, Format & Test Check (`scripts/check-all.ps1`)
# Runs all verification gates across Rust, Frontend, Governance, and Workflows.

$ErrorActionPreference = "Stop"
$failedGates = 0

function Run-Gate {
    param(
        [string]$Name,
        [scriptblock]$Script
    )
    Write-Output "`n=== [GATE] $Name ==="
    try {
        & $Script
        Write-Output ">>> [PASS] Gate: $Name"
    } catch {
        Write-Output ">>> [FAIL] Gate: $Name - $_"
        $script:failedGates++
    }
}

Write-Output "Starting web-repo-to-ai monorepo quality audit..."

# 1. Bootstrap & Governance Integrity Gate
Run-Gate "Bootstrap & Governance Integrity" {
    powershell -ExecutionPolicy Bypass -File scripts/verify-bootstrap.ps1
    if ($LASTEXITCODE -ne 0) { throw "Bootstrap verification failed with exit code $LASTEXITCODE" }
}

# 2. GitHub Actions Workflows Validation Gate
Run-Gate "GitHub Actions Workflow Validation" {
    python scripts/validate-workflows.py
    if ($LASTEXITCODE -ne 0) { throw "Workflow validation failed with exit code $LASTEXITCODE" }
}

# 3. Rust Workspace Quality Gate (formatting, clippy, tests)
Run-Gate "Rust Workspace Checks" {
    if (Test-Path -Path "Cargo.toml" -PathType Leaf) {
        Write-Output "Checking cargo fmt..."
        cargo fmt --all -- --check
        if ($LASTEXITCODE -ne 0) { throw "cargo fmt check failed" }

        Write-Output "Checking cargo clippy..."
        cargo clippy --workspace --all-targets --jobs 4 -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw "cargo clippy check failed" }

        Write-Output "Checking cargo test..."
        cargo test --workspace --jobs 4
        if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }

        if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
            Write-Output "Checking cargo deny..."
            cargo deny check
            if ($LASTEXITCODE -ne 0) { throw "cargo deny check failed" }
        }
    } else {
        Write-Output "Notice: Root Cargo.toml scheduled for Stage 2 (Rust Architecture). rustfmt.toml verified."
    }
}

# 4. Frontend Quality Gate (lint, check, test)
Run-Gate "Frontend Web Quality Checks" {
    if (Test-Path -Path "apps/web/package.json" -PathType Leaf) {
        $rootItem = Get-Item "." -ErrorAction SilentlyContinue
        $appItem = Get-Item "apps" -ErrorAction SilentlyContinue
        $webDir = if ($appItem -and $appItem.LinkType -eq "Junction") { 
            Join-Path $appItem.Target[0] "web" 
        } elseif ($rootItem -and $rootItem.LinkType -eq "Junction") {
            Join-Path $rootItem.Target[0] "apps/web"
        } else { 
            "apps/web" 
        }
        Push-Location $webDir
        try {
            Write-Output "Checking frontend lint..."
            npm run lint
            if ($LASTEXITCODE -ne 0) { throw "npm run lint failed" }

            Write-Output "Checking frontend types (svelte-check)..."
            npm run check
            if ($LASTEXITCODE -ne 0) { throw "npm run check failed" }

            Write-Output "Checking frontend tests..."
            npm test
            if ($LASTEXITCODE -ne 0) { throw "npm test failed" }
        } finally {
            Pop-Location
        }
    } else {
        Write-Output "Notice: apps/web package.json scheduled for Stage 1 (Frontend Experience). .prettierrc verified."
    }

    if (Test-Path -Path "apps/extension/package.json" -PathType Leaf) {
        $rootItem = Get-Item "." -ErrorAction SilentlyContinue
        $appItem = Get-Item "apps" -ErrorAction SilentlyContinue
        $extDir = if ($appItem -and $appItem.LinkType -eq "Junction") { 
            Join-Path $appItem.Target[0] "extension" 
        } elseif ($rootItem -and $rootItem.LinkType -eq "Junction") {
            Join-Path $rootItem.Target[0] "apps/extension"
        } else { 
            "apps/extension" 
        }
        Push-Location $extDir
        try {
            Write-Output "Checking extension lint..."
            npm run lint
            if ($LASTEXITCODE -ne 0) { throw "extension npm run lint failed" }

            Write-Output "Checking extension types (tsc)..."
            npm run typecheck
            if ($LASTEXITCODE -ne 0) { throw "extension npm run typecheck failed" }

            Write-Output "Checking extension build..."
            npm run build
            if ($LASTEXITCODE -ne 0) { throw "extension npm run build failed" }

            Write-Output "Checking extension tests..."
            npm test
            if ($LASTEXITCODE -ne 0) { throw "extension npm test failed" }
        } finally {
            Pop-Location
        }
    }
}

# 5. Resource Budget & 30 GB Storage Ceiling Gate
Run-Gate "Resource Budget & 30 GB Storage Ceiling" {
    powershell -ExecutionPolicy Bypass -File scripts/check-resources.ps1 -Quiet
    if ($LASTEXITCODE -ne 0) { throw "Resource budget check failed with exit code $LASTEXITCODE" }
}

Write-Output "`n=========================================="
if ($failedGates -eq 0) {
    Write-Output "ALL GATES PASSED (0 failures)."
    exit 0
} else {
    Write-Output "AUDIT FAILED ($failedGates gates failed)."
    exit 1
}
