#!/usr/bin/env bash
# web-repo-to-ai Development Service Restart (scripts/dev-restart.sh)
# Restarts containerized services for Linux / macOS / CI.

set -euo pipefail

SERVICES=()
BUILD=false
TIMEOUT_SECONDS=60
DRY_RUN=false

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
    --timeout)
      TIMEOUT_SECONDS="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/dev-restart.sh [options]"
      echo "Options:"
      echo "  --services, -s <svc,...>  Comma-separated list of services (postgres, api, worker, web, browser-worker)"
      echo "  --build                   Rebuild container images before restarting"
      echo "  --timeout <sec>           Seconds to wait for healthchecks (default: 60)"
      echo "  --dry-run                 Simulate commands without restarting services"
      exit 0
      ;;
    *)
      SERVICES+=("$1")
      shift
      ;;
  esac
done

echo "========================================================================"
echo "           web-repo-to-ai Service Restart Utility                       "
echo "========================================================================"

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

# 1. Optional Rebuild
if [ "$BUILD" = true ]; then
  BUILD_ARGS=("-f" "$COMPOSE_FILE")
  if [ "$DRY_RUN" = true ]; then BUILD_ARGS+=("--dry-run"); fi
  BUILD_ARGS+=("build")
  if [ ${#SERVICES[@]} -gt 0 ]; then BUILD_ARGS+=("${SERVICES[@]}"); fi
  echo "Executing build: ${COMPOSE_CMD[*]} ${BUILD_ARGS[*]}"
  if [ "$DRY_RUN" = false ]; then
    "${COMPOSE_CMD[@]}" "${BUILD_ARGS[@]}"
  fi
fi

# 2. Restart Services
CMD_ARGS=("-f" "$COMPOSE_FILE")
if [ "$DRY_RUN" = true ]; then CMD_ARGS+=("--dry-run"); fi
CMD_ARGS+=("restart")
if [ ${#SERVICES[@]} -gt 0 ]; then CMD_ARGS+=("${SERVICES[@]}"); fi

echo "Executing restart: ${COMPOSE_CMD[*]} ${CMD_ARGS[*]}"

if [ "$DRY_RUN" = true ]; then
  echo ""
  echo "[DRY RUN] Command formulated successfully. No services restarted."
  exit 0
fi

"${COMPOSE_CMD[@]}" "${CMD_ARGS[@]}"

# 3. Wait for PostgreSQL Healthcheck if restarted
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

if [ "$should_check_postgres" = true ]; then
  echo "Waiting for PostgreSQL service healthcheck..."
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

echo "Services restarted successfully."
echo "========================================================================"
exit 0
