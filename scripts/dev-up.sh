#!/usr/bin/env bash
# web-repo-to-ai Development Environment Bootstrap (scripts/dev-up.sh)
# Starts containerized services for Linux / macOS / CI.

set -euo pipefail

SERVICES=()
BUILD=false
NO_DETACH=false
SKIP_RESOURCE_CHECK=false
DRY_RUN=false
TIMEOUT_SECONDS=60

while [[ $# -gt 0 ]]; do
  case "$1" in
    --services|-s)
      IFS=',' read -r -a SERVICES <<< "$2"
      shift 2
      ;;
    --build)
      BUILD=true
      shift
      ;;
    --no-detach)
      NO_DETACH=true
      shift
      ;;
    --skip-resource-check)
      SKIP_RESOURCE_CHECK=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --timeout)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    -h|--help)
      echo "Usage: scripts/dev-up.sh [options]"
      echo "Options:"
      echo "  --services, -s <svc,...>  Comma-separated list of services (postgres, api, worker, web, browser-worker)"
      echo "  --build                   Rebuild container images before starting"
      echo "  --no-detach               Run in foreground"
      echo "  --skip-resource-check     Bypass the 30 GB storage ceiling pre-flight check"
      echo "  --dry-run                 Simulate commands without starting containers"
      echo "  --timeout <sec>           Seconds to wait for healthchecks (default: 60)"
      exit 0
      ;;
    *)
      SERVICES+=("$1")
      shift
      ;;
  esac
done

echo "========================================================================"
echo "         web-repo-to-ai Development Environment Bootstrap              "
echo "========================================================================"

# 1. Ensure deploy/.env exists
if [ ! -f "deploy/.env" ]; then
  if [ -f "deploy/.env.example" ]; then
    echo "Notice: deploy/.env not found. Initializing from deploy/.env.example..."
    cp deploy/.env.example deploy/.env
    echo "Created deploy/.env with default development settings."
  fi
fi

# 2. Check 30 GB storage ceiling
if [ "$SKIP_RESOURCE_CHECK" = false ]; then
  echo "Verifying resource budget and 30 GB container storage ceiling..."
  if [ -f "scripts/check-resources.sh" ]; then
    if ! bash scripts/check-resources.sh --quiet; then
      echo "ERROR: Storage ceiling check failed! Run scripts/clean-caches.sh or inspect with scripts/check-resources.sh." >&2
      exit 1
    fi
    echo "Storage budget verified (< 30 GB)."
  fi
fi

# 3. Resolve Compose Binary
COMPOSE_CMD=()
if command -v podman-compose &>/dev/null; then
  COMPOSE_CMD=("podman-compose")
elif command -v podman-compose.exe &>/dev/null; then
  COMPOSE_CMD=("podman-compose.exe")
elif command -v podman &>/dev/null && podman compose version &>/dev/null; then
  COMPOSE_CMD=("podman" "compose")
elif command -v podman.exe &>/dev/null; then
  COMPOSE_CMD=("podman.exe" "compose")
elif command -v docker-compose &>/dev/null; then
  COMPOSE_CMD=("docker-compose")
elif command -v docker-compose.exe &>/dev/null; then
  COMPOSE_CMD=("docker-compose.exe")
else
  echo "ERROR: Neither podman-compose, podman compose, nor docker-compose was found in PATH." >&2
  exit 1
fi

COMPOSE_FILE="deploy/podman-compose.yml"
if [ ! -f "$COMPOSE_FILE" ]; then
  echo "ERROR: Compose file not found at $COMPOSE_FILE" >&2
  exit 1
fi

# 4. Assemble Arguments
CMD_ARGS=("-f" "$COMPOSE_FILE")
if [ "$DRY_RUN" = true ]; then
  CMD_ARGS+=("--dry-run")
fi
CMD_ARGS+=("up")
if [ "$NO_DETACH" = false ]; then
  CMD_ARGS+=("-d")
fi
if [ "$BUILD" = true ]; then
  CMD_ARGS+=("--build")
fi
if [ ${#SERVICES[@]} -gt 0 ]; then
  CMD_ARGS+=("${SERVICES[@]}")
fi

echo "Executing: ${COMPOSE_CMD[*]} ${CMD_ARGS[*]}"

if [ "$DRY_RUN" = true ]; then
  echo ""
  echo "[DRY RUN] Command formulated successfully. No containers started."
  exit 0
fi

# 5. Launch Services
"${COMPOSE_CMD[@]}" "${CMD_ARGS[@]}"

# 6. Wait for PostgreSQL Healthcheck if starting detached
should_check_postgres=false
if [ ${#SERVICES[@]} -eq 0 ]; then
  should_check_postgres=true
else
  for s in "${SERVICES[@]}"; do
    if [ "$s" = "postgres" ]; then
      should_check_postgres=true
      break
    fi
  done
fi

if [ "$should_check_postgres" = true ] && [ "$NO_DETACH" = false ]; then
  echo "Waiting for PostgreSQL service to become healthy..."
  elapsed=0
  healthy=false
  while [ "$elapsed" -lt "$TIMEOUT_SECONDS" ]; do
    status=$(podman inspect --format '{{.State.Health.Status}}' web-repo-postgres 2>/dev/null || true)
    if [ "$status" = "healthy" ]; then
      healthy=true
      break
    fi
    sleep 2
    elapsed=$((elapsed + 2))
  done
  if [ "$healthy" = true ]; then
    echo "PostgreSQL is healthy and ready for connections."
  else
    echo "WARNING: PostgreSQL healthcheck did not report 'healthy' within $TIMEOUT_SECONDS seconds."
  fi
fi

echo "------------------------------------------------------------------------"
echo "Development Services Endpoints:"
echo "  - Web Frontend   : http://localhost:5173"
echo "  - API Service    : http://localhost:8080 (Health: http://localhost:8080/health)"
echo "  - Browser Worker : http://localhost:3000"
echo "  - PostgreSQL     : localhost:5432 (DB: webrepotodev, User: webrepouser)"
echo "------------------------------------------------------------------------"
echo "Useful lifecycle commands:"
echo "  - Stream logs   : bash scripts/dev-logs.sh --follow"
echo "  - Check status  : bash scripts/dev-status.sh"
echo "  - Stop services : bash scripts/dev-down.sh"
echo "========================================================================"

exit 0
