#!/usr/bin/env bash
# web-repo-to-ai Development Service Status & Health (scripts/dev-status.sh)
# Reports container states, health checks, ports, and storage budget for Linux / macOS / CI.

set -euo pipefail

JSON_OUTPUT=false
QUIET=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --json)
      JSON_OUTPUT=true
      shift
      ;;
    --quiet|-q)
      QUIET=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/dev-status.sh [options]"
      echo "Options:"
      echo "  --json       Emit machine-readable JSON"
      echo "  --quiet, -q  Compact single-line output"
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

python3 - <<EOF
import json
import shutil
import subprocess
import sys
import time

json_output = "${JSON_OUTPUT}".lower() == "true"
quiet_output = "${QUIET}".lower() == "true"

services = [
    {"name": "postgres", "container": "web-repo-postgres", "port": "5432", "desc": "PostgreSQL 16 Database"},
    {"name": "api", "container": "web-repo-api", "port": "8080", "desc": "Rust Axum API Service"},
    {"name": "worker", "container": "web-repo-worker", "port": "Internal", "desc": "Rust Processing Worker"},
    {"name": "browser-worker", "container": "web-repo-browser-worker", "port": "3000", "desc": "Playwright Browser Worker"},
    {"name": "web", "container": "web-repo-web", "port": "5173", "desc": "SvelteKit Web Frontend"}
]

podman_cmd = "podman.exe" if shutil.which("podman.exe") else "podman"
has_podman = shutil.which(podman_cmd) is not None

status_list = []
running_count = 0

for s in services:
    c_name = s["container"]
    state = "NOT_CREATED"
    health = "N/A"
    
    if has_podman:
        try:
            res = subprocess.run([podman_cmd, "inspect", "--format", "{{json .State}}", c_name], capture_output=True, text=True, check=False)
            if res.returncode == 0 and res.stdout.strip():
                state_data = json.loads(res.stdout)
                if state_data.get("Running", False):
                    state = "RUNNING"
                    running_count += 1
                else:
                    state = f"STOPPED ({state_data.get('Status', 'exited')})"
                
                h = state_data.get("Health")
                if h and isinstance(h, dict):
                    health = h.get("Status", "N/A")
        except Exception:
            pass

    status_list.append({
        "service": s["name"],
        "container": s["container"],
        "state": state,
        "health": health,
        "port": s["port"],
        "description": s["desc"]
    })

storage_info = "N/A"
storage_status = "UNKNOWN"
try:
    chk_res = subprocess.run(["bash", "scripts/check-resources.sh", "--json"], capture_output=True, text=True, check=False)
    if chk_res.returncode == 0 and chk_res.stdout.strip():
        chk_data = json.loads(chk_res.stdout)
        used_gb = round(chk_data.get("used_bytes", 0) / (1024**3), 2)
        storage_info = f"{used_gb:.2f} GB / {chk_data.get('ceiling_gb', 30.0):.1f} GB ({chk_data.get('used_percent', 0.0):.1f}%)"
        storage_status = chk_data.get("status", "HEALTHY")
except Exception:
    pass

if json_output:
    payload = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "total_services": len(services),
        "running_count": running_count,
        "storage_budget": {
            "usage": storage_info,
            "status": storage_status
        },
        "services": status_list
    }
    print(json.dumps(payload, indent=2))
    sys.exit(0)

if not quiet_output:
    print("=" * 72)
    print("       web-repo-to-ai Development Services Status & Health             ")
    print("=" * 72)
    print(f"Running Services : {running_count} / {len(services)}")
    print(f"Storage Footprint: {storage_info} [{storage_status}]")
    print("-" * 72)
    print(f"{'SERVICE':<16} {'CONTAINER':<26} {'STATE':<16} {'HEALTH':<10} {'PORT'}")
    print(f"{'-------':<16} {'---------':<26} {'-----':<16} {'------':<10} {'----'}")
    for s in status_list:
        print(f"{s['service']:<16} {s['container']:<26} {s['state']:<16} {s['health']:<10} {s['port']}")
    print("=" * 72)
else:
    print(f"Running: {running_count}/{len(services)}, Storage: {storage_info} [{storage_status}]")
EOF
