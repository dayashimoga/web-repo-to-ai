#!/usr/bin/env bash
# web-repo-to-ai Development Environment Teardown (scripts/dev-down.sh)
# Stops containerized services for Linux / macOS / CI.

set -euo pipefail

VOLUMES=false
FORCE=false
PRUNE_CACHES=false
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --volumes|-v)
      VOLUMES=true
      shift
      ;;
    --force)
      FORCE=true
      shift
      ;;
    --prune-caches)
      PRUNE_CACHES=true
      shift
      ;;
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/dev-down.sh [options]"
      echo "Options:"
      echo "  --volumes, -v    CAUTION: Remove persistent data volumes (postgres_data, artifact_data)"
      echo "  --force          Skip confirmation when --volumes is specified"
      echo "  --prune-caches   Run scripts/clean-caches.sh after teardown"
      echo "  --dry-run        Simulate commands without stopping containers"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

echo "========================================================================"
echo "           web-repo-to-ai Development Teardown Utility                  "
echo "========================================================================"

# Volume confirmation guard
if [ "$VOLUMES" = true ]; then
  if [ "$FORCE" = false ]; then
    echo "CAUTION: You specified --volumes." >&2
    echo "This will permanently delete the local PostgreSQL database (postgres_data) and cached artifacts (artifact_data)!" >&2
    read -r -p "Are you absolutely sure you want to delete persistent volumes? (yes/NO): " confirm
    if [ "$confirm" != "yes" ]; then
      echo "Volume deletion aborted by user. Retaining persistent data volumes."
      VOLUMES=false
    fi
  fi
fi

# Resolve Compose Binary
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

CMD_ARGS=("-f" "$COMPOSE_FILE")
if [ "$DRY_RUN" = true ]; then
  CMD_ARGS+=("--dry-run")
fi
CMD_ARGS+=("down")
if [ "$VOLUMES" = true ]; then
  CMD_ARGS+=("-v")
fi

echo "Executing: ${COMPOSE_CMD[*]} ${CMD_ARGS[*]}"

if [ "$DRY_RUN" = true ]; then
  echo ""
  echo "[DRY RUN] Command formulated successfully. No containers stopped."
  if [ "$PRUNE_CACHES" = true ]; then
    echo "[DRY RUN] Would invoke scripts/clean-caches.sh after teardown."
  fi
  exit 0
fi

# Execute Compose Down
"${COMPOSE_CMD[@]}" "${CMD_ARGS[@]}"
echo "Development services stopped successfully."

# Optional Cache Pruning
if [ "$PRUNE_CACHES" = true ]; then
  echo ""
  echo "Executing post-teardown cache pruning..."
  if [ -f "scripts/clean-caches.sh" ]; then
    bash scripts/clean-caches.sh
  fi
fi

echo "========================================================================"
exit 0
