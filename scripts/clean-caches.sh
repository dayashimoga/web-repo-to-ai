#!/usr/bin/env bash
# web-repo-to-ai Safe Podman Cache Pruning Utility (scripts/clean-caches.sh)
# Prunes Podman build caches, intermediate containers, and dangling layers to respect the 30 GB ceiling.

set -euo pipefail

DRY_RUN=false
BUILD_CACHE_ONLY=false
INCLUDE_VOLUMES=false
FORCE=false
QUIET=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --dry-run)
      DRY_RUN=true
      shift
      ;;
    --build-cache-only)
      BUILD_CACHE_ONLY=true
      shift
      ;;
    --include-volumes)
      INCLUDE_VOLUMES=true
      shift
      ;;
    --force)
      FORCE=true
      shift
      ;;
    --quiet)
      QUIET=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/clean-caches.sh [options]"
      echo "Options:"
      echo "  --dry-run            Report reclaimable space without deleting anything"
      echo "  --build-cache-only   Only prune persistent build cache"
      echo "  --include-volumes    CAUTION: Prune unused volumes (requires confirmation or --force)"
      echo "  --force              Skip interactive prompt for volume pruning"
      echo "  --quiet              Suppress verbose output"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

if ! command -v podman &>/dev/null; then
  echo "Podman is not installed or not in PATH. Nothing to clean." >&2
  exit 0
fi

get_storage_bytes() {
  python3 -c "
import subprocess, json
try:
    res = subprocess.run(['podman', 'system', 'df', '--format', 'json'], capture_output=True, text=True)
    if res.returncode == 0 and res.stdout.strip():
        data = json.loads(res.stdout)
        total = sum(int(item.get('RawSize', 0) or 0) for item in data)
        print(total)
    else:
        print(0)
except Exception:
    print(0)
"
}

format_bytes() {
  python3 -c "
b = float($1)
if b >= 1024**3:
    print(f'{b / (1024**3):.2f} GB')
elif b >= 1024**2:
    print(f'{b / (1024**2):.2f} MB')
elif b >= 1024:
    print(f'{b / 1024:.2f} KB')
else:
    print(f'{int(b)} B')
"
}

initial_bytes=$(get_storage_bytes)

if [ "$QUIET" = false ]; then
  echo "========================================================================"
  echo "          web-repo-to-ai Safe Podman Cache Pruning Utility             "
  echo "========================================================================"
  echo "Initial Podman Storage Usage: $(format_bytes "$initial_bytes")"
fi

if [ "$DRY_RUN" = true ]; then
  echo ""
  echo "[DRY RUN] Inspecting reclaimable Podman space without deletion..."
  podman system df || true
  echo ""
  echo "[DRY RUN] Actions that would be performed during normal execution:"
  echo "  1. podman image prune --build-cache -f (prune persistent build cache)"
  if [ "$BUILD_CACHE_ONLY" = false ]; then
    echo "  2. podman system prune --build -f       (prune intermediate build containers)"
    echo "  3. podman image prune -f                (prune dangling untagged images)"
  fi
  if [ "$INCLUDE_VOLUMES" = true ]; then
    echo "  4. podman volume prune -f               (CAUTION: would prune unused volumes)"
  else
    echo "  *  Persistent database volumes (e.g. postgres_data) are PROTECTED and retained."
  fi
  echo ""
  echo "[DRY RUN] No changes were made."
  exit 0
fi

# Guard volume pruning
if [ "$INCLUDE_VOLUMES" = true ]; then
  if [ "$FORCE" = false ]; then
    echo "WARNING: You specified --include-volumes. This will remove unused local container volumes." >&2
    echo "If your development database volume is not currently in use, its state will be LOST!" >&2
    read -r -p "Are you absolutely sure you want to prune volumes? (yes/NO): " confirm
    if [ "$confirm" != "yes" ]; then
      echo "Volume pruning aborted by user."
      INCLUDE_VOLUMES=false
    fi
  fi
fi

if [ "$QUIET" = false ]; then
  echo ""
  echo "Executing safe cache pruning..."
fi

# Step 1: Prune build cache
if [ "$QUIET" = false ]; then echo "  -> Pruning Podman build cache..."; fi
podman image prune --build-cache -f >/dev/null 2>&1 || true

if [ "$BUILD_CACHE_ONLY" = false ]; then
  # Step 2: Prune build containers
  if [ "$QUIET" = false ]; then echo "  -> Pruning stopped build containers..."; fi
  podman system prune --build -f >/dev/null 2>&1 || true

  # Step 3: Prune dangling images
  if [ "$QUIET" = false ]; then echo "  -> Pruning dangling (untagged) image layers..."; fi
  podman image prune -f >/dev/null 2>&1 || true
fi

# Step 4: Optional volume prune
if [ "$INCLUDE_VOLUMES" = true ]; then
  if [ "$QUIET" = false ]; then echo "  -> Pruning unused volumes (explicitly requested)..."; fi
  podman volume prune -f >/dev/null 2>&1 || true
fi

final_bytes=$(get_storage_bytes)
reclaimed_bytes=$(( initial_bytes > final_bytes ? initial_bytes - final_bytes : 0 ))

if [ "$QUIET" = false ]; then
  echo "------------------------------------------------------------------------"
  echo "Final Podman Storage Usage  : $(format_bytes "$final_bytes")"
  echo "Total Storage Reclaimed     : $(format_bytes "$reclaimed_bytes")"
  echo "------------------------------------------------------------------------"
  echo "Running post-cleanup resource ceiling verification:"
  bash scripts/check-resources.sh --quiet
  echo "========================================================================"
else
  echo "Reclaimed: $(format_bytes "$reclaimed_bytes"), Final Usage: $(format_bytes "$final_bytes")"
fi

exit 0
