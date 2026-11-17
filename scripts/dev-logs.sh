#!/usr/bin/env bash
# web-repo-to-ai Development Log Viewer (scripts/dev-logs.sh)
# Views and tails container logs for Linux / macOS / CI.

set -euo pipefail

SERVICE=""
FOLLOW=false
TAIL=50
TIMESTAMPS=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --service|-s)
      SERVICE="$2"
      shift 2
      ;;
    --follow|-f)
      FOLLOW=true
      shift
      ;;
    --tail)
      TAIL="$2"
      shift 2
      ;;
    --timestamps|-t)
      TIMESTAMPS=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/dev-logs.sh [options]"
      echo "Options:"
      echo "  --service, -s <svc>  Specific service name (postgres, api, worker, browser-worker, web)"
      echo "  --follow, -f         Follow log stream in real time"
      echo "  --tail <lines>       Number of lines to show (default: 50)"
      echo "  --timestamps, -t     Show timestamps in log output"
      exit 0
      ;;
    *)
      SERVICE="$1"
      shift
      ;;
  esac
done

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

CMD_ARGS=("-f" "$COMPOSE_FILE" "logs" "--tail" "$TAIL")
if [ "$FOLLOW" = true ]; then
  CMD_ARGS+=("-f")
fi
if [ "$TIMESTAMPS" = true ]; then
  CMD_ARGS+=("-t")
fi
if [ -n "$SERVICE" ]; then
  CMD_ARGS+=("$SERVICE")
fi

echo "Streaming logs via: ${COMPOSE_CMD[*]} ${CMD_ARGS[*]}"
exec "${COMPOSE_CMD[@]}" "${CMD_ARGS[@]}"
