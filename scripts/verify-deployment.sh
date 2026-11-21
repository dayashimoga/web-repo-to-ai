#!/usr/bin/env bash
# ==============================================================================
# web-repo-to-ai Universal Deployment Verification Engine (POSIX)
# ==============================================================================
set -euo pipefail

DIST_DIR="${1:-dist}"
CHECKS=0
FAILURES=0

assert_check() {
    local condition="$1"
    local message="$2"
    CHECKS=$((CHECKS + 1))
    if [ "$condition" -eq 0 ]; then
        echo "  [PASS] $message"
    else
        echo "  [FAIL] $message"
        FAILURES=$((FAILURES + 1))
    fi
}

echo "========================================================================"
echo "       web-repo-to-ai Universal Deployment Verification Engine (POSIX)   "
echo "========================================================================"

echo ""
echo "[1/4] Auditing Self-Hosted Container Stack (deploy/podman-compose.yml)..."
COMPOSE_FILE="deploy/podman-compose.yml"
if [ -f "$COMPOSE_FILE" ]; then
    assert_check 0 "deploy/podman-compose.yml exists"
    grep -q "postgres:" "$COMPOSE_FILE" && assert_check 0 "Declares 'postgres' service" || assert_check 1 "Declares 'postgres' service"
    grep -q "api:" "$COMPOSE_FILE" && assert_check 0 "Declares 'api' service" || assert_check 1 "Declares 'api' service"
    grep -q "worker:" "$COMPOSE_FILE" && assert_check 0 "Declares 'worker' service" || assert_check 1 "Declares 'worker' service"
    grep -q "browser-worker:" "$COMPOSE_FILE" && assert_check 0 "Declares 'browser-worker' service" || assert_check 1 "Declares 'browser-worker' service"
    grep -q "web:" "$COMPOSE_FILE" && assert_check 0 "Declares 'web' service" || assert_check 1 "Declares 'web' service"
    grep -q "internal_net:" "$COMPOSE_FILE" && grep -q "browser_net:" "$COMPOSE_FILE" && assert_check 0 "Declares dual isolated networks" || assert_check 1 "Declares dual isolated networks"
    grep -q "postgres_data:" "$COMPOSE_FILE" && grep -q "artifact_data:" "$COMPOSE_FILE" && assert_check 0 "Declares persistent named volumes" || assert_check 1 "Declares persistent named volumes"
    grep -q "no-new-privileges:true" "$COMPOSE_FILE" && assert_check 0 "Enforces no-new-privileges:true" || assert_check 1 "Enforces no-new-privileges:true"
else
    assert_check 1 "deploy/podman-compose.yml exists"
fi

# Audit Containerfiles
[ -f "deploy/Containerfile.api" ] && grep -q "FROM .* AS builder" "deploy/Containerfile.api" && assert_check 0 "Containerfile.api uses multi-stage build" || assert_check 1 "Containerfile.api uses multi-stage build"
[ -f "deploy/Containerfile.worker" ] && grep -q "FROM .* AS builder" "deploy/Containerfile.worker" && assert_check 0 "Containerfile.worker uses multi-stage build" || assert_check 1 "Containerfile.worker uses multi-stage build"
[ -f "deploy/Containerfile.browser-worker" ] && grep -q "USER pwuser" "deploy/Containerfile.browser-worker" && assert_check 0 "Containerfile.browser-worker enforces unprivileged user pwuser" || assert_check 1 "Containerfile.browser-worker enforces unprivileged user pwuser"
[ -f "deploy/Containerfile.web" ] && grep -q "FROM .* AS builder" "deploy/Containerfile.web" && assert_check 0 "Containerfile.web uses multi-stage build" || assert_check 1 "Containerfile.web uses multi-stage build"

echo ""
echo "[2/4] Auditing Standalone Release Binary Distribution..."
BIN_DIR="$DIST_DIR/bin"
if [ -d "$BIN_DIR" ]; then
    assert_check 0 "Production binary directory '$BIN_DIR' exists"
    for b in web-repo-to-ai web-repo-to-ai-cli web-repo-to-ai-api web-repo-to-ai-worker web-repo-to-ai-mcp; do
        if [ -f "$BIN_DIR/$b" ] || [ -f "$BIN_DIR/$b.exe" ]; then
            assert_check 0 "Binary '$b' exists in '$BIN_DIR'"
        else
            assert_check 1 "Binary '$b' exists in '$BIN_DIR'"
        fi
    done
else
    assert_check 1 "Production binary directory '$BIN_DIR' exists"
fi

[ -f "$DIST_DIR/checksums.sha256" ] && assert_check 0 "Production checksums manifest exists" || assert_check 1 "Production checksums manifest exists"
[ -f "$DIST_DIR/BUILD_MANIFEST.json" ] && assert_check 0 "Production build manifest exists" || assert_check 1 "Production build manifest exists"

echo ""
echo "[3/4] Auditing Chrome Extension MV3 Distribution..."
EXT_DIR="$DIST_DIR/extension"
if [ ! -d "$EXT_DIR" ]; then
    EXT_DIR="apps/extension/dist"
fi
if [ -f "$EXT_DIR/manifest.json" ]; then
    assert_check 0 "Extension manifest.json exists in '$EXT_DIR'"
    grep -q '"manifest_version": 3' "$EXT_DIR/manifest.json" && assert_check 0 "Extension targets Manifest V3" || assert_check 1 "Extension targets Manifest V3"
    grep -q 'script-src' "$EXT_DIR/manifest.json" && assert_check 0 "Extension enforces Content Security Policy" || assert_check 1 "Extension enforces Content Security Policy"
else
    assert_check 1 "Extension manifest.json exists"
fi

echo ""
echo "[4/4] Auditing Disaster Recovery Runbooks & Storage Ceiling..."
if [ -f "scripts/check-resources.sh" ]; then
    bash scripts/check-resources.sh -Quiet >/dev/null 2>&1 && assert_check 0 "Storage footprint satisfies 30 GB ceiling" || assert_check 1 "Storage footprint satisfies 30 GB ceiling"
fi

if [ -f "docs/DEPLOYMENT.md" ]; then
    grep -q "Backup, Restore & Disaster Recovery Runbook" "docs/DEPLOYMENT.md" && assert_check 0 "docs/DEPLOYMENT.md contains Disaster Recovery Runbook" || assert_check 1 "docs/DEPLOYMENT.md contains Disaster Recovery Runbook"
    grep -q "pg_dump" "docs/DEPLOYMENT.md" && assert_check 0 "docs/DEPLOYMENT.md contains pg_dump commands" || assert_check 1 "docs/DEPLOYMENT.md contains pg_dump commands"
fi

PASSED=$((CHECKS - FAILURES))
echo "------------------------------------------------------------------------"
echo "Deployment Verification Summary: $PASSED/$CHECKS checks passed, $FAILURES failures"
echo "========================================================================"

if [ "$FAILURES" -gt 0 ]; then
    exit 1
fi
exit 0
