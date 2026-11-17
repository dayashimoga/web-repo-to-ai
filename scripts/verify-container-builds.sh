#!/usr/bin/env bash
# web-repo-to-ai Container Build & Architecture Audit (scripts/verify-container-builds.sh)
# Audits Containerfiles, compose configuration, and the 30 GB storage ceiling for Linux / macOS / CI.

set -euo pipefail

QUICK=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --quick|-q)
      QUICK=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/verify-container-builds.sh [options]"
      echo "Options:"
      echo "  --quick, -q   Skip live container startup test and perform only static checks"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

checks=0
failures=0

assert_test() {
  local condition="$1"
  local message="$2"
  checks=$((checks + 1))
  if eval "$condition"; then
    echo "  [PASS] $message"
  else
    echo "  [FAIL] $message"
    failures=$((failures + 1))
  fi
}

echo "========================================================================"
echo "       web-repo-to-ai Container Build & Architecture Audit              "
echo "========================================================================"

# 1. Audit Compose Configuration
echo ""
echo "[1/4] Auditing deploy/podman-compose.yml..."
assert_test '[ -f "deploy/podman-compose.yml" ]' "deploy/podman-compose.yml exists"
assert_test 'grep -q "postgres:" deploy/podman-compose.yml' "Declares 'postgres' service"
assert_test 'grep -q "api:" deploy/podman-compose.yml' "Declares 'api' service"
assert_test 'grep -q "worker:" deploy/podman-compose.yml' "Declares 'worker' service"
assert_test 'grep -q "browser-worker:" deploy/podman-compose.yml' "Declares 'browser-worker' service"
assert_test 'grep -q "web:" deploy/podman-compose.yml' "Declares 'web' service"
assert_test 'grep -q "internal_net:" deploy/podman-compose.yml && grep -q "browser_net:" deploy/podman-compose.yml' "Declares dual isolated networks ('internal_net', 'browser_net')"
assert_test 'grep -q "postgres_data:" deploy/podman-compose.yml && grep -q "artifact_data:" deploy/podman-compose.yml' "Declares persistent named volumes ('postgres_data', 'artifact_data')"
assert_test 'grep -q "no-new-privileges:true" deploy/podman-compose.yml' "Enforces 'no-new-privileges:true' security sandbox on browser-worker"

# 2. Audit Containerfiles
echo ""
echo "[2/4] Auditing Containerfiles for multi-stage builds and unprivileged users..."
assert_test 'grep -q "FROM .* AS builder" deploy/Containerfile.api && grep -q "FROM .* AS runtime" deploy/Containerfile.api' "Containerfile.api uses multi-stage build"
assert_test 'grep -q "rust:.*slim" deploy/Containerfile.api && grep -q "debian:.*slim" deploy/Containerfile.api' "Containerfile.api uses minimal slim base images"
assert_test 'grep -q "USER appuser" deploy/Containerfile.api' "Containerfile.api enforces unprivileged runtime user 'appuser'"

assert_test 'grep -q "FROM .* AS builder" deploy/Containerfile.worker && grep -q "FROM .* AS runtime" deploy/Containerfile.worker' "Containerfile.worker uses multi-stage build"
assert_test 'grep -q "rust:.*slim" deploy/Containerfile.worker && grep -q "debian:.*slim" deploy/Containerfile.worker' "Containerfile.worker uses minimal slim base images"
assert_test 'grep -q "USER appuser" deploy/Containerfile.worker' "Containerfile.worker enforces unprivileged runtime user 'appuser'"

assert_test 'grep -q "playwright:" deploy/Containerfile.browser-worker' "Containerfile.browser-worker uses pinned Playwright base image"
assert_test 'grep -q "USER pwuser" deploy/Containerfile.browser-worker' "Containerfile.browser-worker enforces unprivileged user 'pwuser'"

assert_test 'grep -q "FROM .* AS builder" deploy/Containerfile.web && grep -q "FROM .* AS runtime" deploy/Containerfile.web' "Containerfile.web uses multi-stage build"
assert_test 'grep -q "node:.*slim" deploy/Containerfile.web' "Containerfile.web uses minimal Node slim base image"
assert_test 'grep -q "USER node" deploy/Containerfile.web' "Containerfile.web enforces unprivileged runtime user 'node'"

# 3. Storage Ceiling
echo ""
echo "[3/4] Enforcing 30 GB Podman storage budget..."
assert_test 'bash scripts/check-resources.sh --quiet' "Podman storage footprint is strictly within the 30 GB ceiling"

# 4. Live Container Startup & Healthcheck
if [ "$QUICK" = false ]; then
  echo ""
  echo "[4/4] Validating live container startup & healthcheck..."
  assert_test 'bash scripts/dev-up.sh -s postgres --timeout 30' "scripts/dev-up.sh successfully started PostgreSQL and verified healthcheck"
  assert_test 'bash scripts/dev-status.sh --json | grep -q "\"state\": \"RUNNING\""' "web-repo-postgres reports RUNNING in live inspection"
  assert_test 'bash scripts/dev-down.sh' "scripts/dev-down.sh successfully stopped containers"
  assert_test 'podman volume ls --format "{{.Name}}" | grep -q "postgres_data"' "Persistent volume 'deploy_postgres_data' retained across teardown"
else
  echo ""
  echo "[4/4] Live container startup skipped (--quick mode)."
fi

echo "------------------------------------------------------------------------"
echo "Container Verification Summary: $((checks - failures))/$checks checks passed, $failures failures"
echo "========================================================================"

if [ "$failures" -gt 0 ]; then
  exit 1
fi
exit 0
