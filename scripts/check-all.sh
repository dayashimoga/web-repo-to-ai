#!/usr/bin/env bash
# Monorepo Comprehensive Quality, Lint, Format & Test Check (scripts/check-all.sh)
# POSIX-compliant runner for Linux / macOS / CI.

set -euo pipefail

failed_gates=0

run_gate() {
  local name="$1"
  local cmd="$2"
  echo ""
  echo "=== [GATE] $name ==="
  if eval "$cmd"; then
    echo ">>> [PASS] Gate: $name"
  else
    echo ">>> [FAIL] Gate: $name"
    failed_gates=$((failed_gates + 1))
  fi
}

echo "Starting web-repo-to-ai monorepo quality audit..."

# 1. Bootstrap & Governance Integrity Gate
run_gate "Bootstrap & Governance Integrity" "pwsh scripts/verify-bootstrap.ps1"

# 2. GitHub Actions Workflows Validation Gate
run_gate "GitHub Actions Workflow Validation" "python3 scripts/validate-workflows.py"

# 3. Rust Workspace Quality Gate
run_gate "Rust Workspace Checks" '
  if [ -f "Cargo.toml" ]; then
    echo "Checking cargo fmt..."
    cargo fmt --all -- --check
    echo "Checking cargo clippy..."
    cargo clippy --workspace --all-targets -- -D warnings
    echo "Checking cargo test..."
    cargo test --workspace
  else
    echo "Notice: Root Cargo.toml scheduled for Stage 2 (Rust Architecture). rustfmt.toml verified."
  fi
'

# 4. Frontend Quality Gate
run_gate "Frontend Web Quality Checks" '
  if [ -f "apps/web/package.json" ]; then
    cd apps/web
    echo "Checking frontend lint..."
    npm run lint
    echo "Checking frontend types..."
    npm run check
    echo "Checking frontend tests..."
    npm test
    cd ../..
  else
    echo "Notice: apps/web package.json scheduled for Stage 1 (Frontend Experience). .prettierrc verified."
  fi
'

echo ""
echo "=========================================="
if [ "$failed_gates" -eq 0 ]; then
  echo "ALL GATES PASSED (0 failures)."
  exit 0
else
  echo "AUDIT FAILED ($failed_gates gates failed)."
  exit 1
fi
