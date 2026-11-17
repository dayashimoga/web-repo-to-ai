#!/usr/bin/env bash
# web-repo-to-ai Resource Monitor & 30 GB Podman Storage Ceiling Check (scripts/check-resources.sh)
# Enforces the 30 GB Podman storage budget on Linux / macOS / CI.

set -euo pipefail

CEILING_GB=30.0
WARN_PERCENT=80.0
JSON_OUTPUT=false
QUIET_OUTPUT=false
FAIL_ON_WARN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --ceiling-gb)
      CEILING_GB="$2"
      shift 2
      ;;
    --warn-percent)
      WARN_PERCENT="$2"
      shift 2
      ;;
    --json)
      JSON_OUTPUT=true
      shift
      ;;
    --quiet)
      QUIET_OUTPUT=true
      shift
      ;;
    --fail-on-warn)
      FAIL_ON_WARN=true
      shift
      ;;
    -h|--help)
      echo "Usage: scripts/check-resources.sh [options]"
      echo "Options:"
      echo "  --ceiling-gb <GB>      Podman storage ceiling in GB (default: 30.0)"
      echo "  --warn-percent <pct>   Warning threshold percentage (default: 80.0)"
      echo "  --json                 Emit machine-readable JSON output"
      echo "  --quiet                Minimal output"
      echo "  --fail-on-warn         Exit with code 1 on warning threshold breach"
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
import os
import shutil
import subprocess
import sys
import time

ceiling_gb = float("${CEILING_GB}")
warn_percent = float("${WARN_PERCENT}")
json_output = "${JSON_OUTPUT}".lower() == "true"
quiet_output = "${QUIET_OUTPUT}".lower() == "true"
fail_on_warn = "${FAIL_ON_WARN}".lower() == "true"

def format_bytes(b):
    if b >= 1024**3:
        return f"{b / (1024**3):.2f} GB"
    elif b >= 1024**2:
        return f"{b / (1024**2):.2f} MB"
    elif b >= 1024:
        return f"{b / 1024:.2f} KB"
    else:
        return f"{int(b)} B"

# 1. Host Resources
cpu_cores = os.cpu_count() or 1
total_mem_bytes = 0
free_mem_bytes = 0

# Check /proc/meminfo or fallback
if os.path.exists("/proc/meminfo"):
    try:
        with open("/proc/meminfo", "r") as f:
            for line in f:
                if line.startswith("MemTotal:"):
                    total_mem_bytes = int(line.split()[1]) * 1024
                elif line.startswith("MemAvailable:"):
                    free_mem_bytes = int(line.split()[1]) * 1024
    except Exception:
        pass

disk_stat = shutil.disk_usage(".")
drive_free_bytes = disk_stat.free
drive_total_bytes = disk_stat.total

# 2. Podman Inspection
podman_path = shutil.which("podman")
podman_installed = podman_path is not None
podman_version = "N/A"
podman_used_bytes = 0
podman_reclaimable_bytes = 0
breakdown = []

if podman_installed:
    try:
        v_res = subprocess.run(["podman", "--version"], capture_output=True, text=True, check=False)
        if v_res.returncode == 0:
            podman_version = v_res.stdout.strip().replace("podman version ", "")
    except Exception:
        pass

    try:
        df_res = subprocess.run(["podman", "system", "df", "--format", "json"], capture_output=True, text=True, check=False)
        if df_res.returncode == 0 and df_res.stdout.strip():
            df_data = json.loads(df_res.stdout)
            for item in df_data:
                raw_size = int(item.get("RawSize", 0) or 0)
                raw_reclaim = int(item.get("RawReclaimable", 0) or 0)
                podman_used_bytes += raw_size
                podman_reclaimable_bytes += raw_reclaim
                breakdown.append({
                    "Type": item.get("Type", "Unknown"),
                    "TotalCount": item.get("TotalCount", 0),
                    "ActiveCount": item.get("Active", 0),
                    "RawSize": raw_size,
                    "Size": item.get("Size", format_bytes(raw_size)),
                    "Reclaimable": item.get("Reclaimable", format_bytes(raw_reclaim))
                })
    except Exception:
        pass

# 3. Budget & Threshold Evaluation
ceiling_bytes = ceiling_gb * (1024**3)
warn_threshold_bytes = ceiling_bytes * (warn_percent / 100.0)
used_percent = round((podman_used_bytes / ceiling_bytes) * 100.0, 2) if ceiling_bytes > 0 else 0.0

if not podman_installed:
    status = "PODMAN_NOT_INSTALLED"
    status_message = "Podman is not installed or not in PATH."
    exit_code = 0
elif podman_used_bytes > ceiling_bytes:
    status = "CRITICAL_CEILING_BREACH"
    status_message = f"Podman storage footprint ({format_bytes(podman_used_bytes)}) EXCEEDS the {ceiling_gb:.1f} GB ceiling ({used_percent:.1f}%)! Run scripts/clean-caches.sh immediately."
    exit_code = 1
elif podman_used_bytes >= warn_threshold_bytes:
    status = "WARNING_STORAGE_HIGH"
    status_message = f"Podman storage footprint ({format_bytes(podman_used_bytes)}) exceeds {warn_percent:.1f}% of {ceiling_gb:.1f} GB ceiling ({used_percent:.1f}%). Run scripts/clean-caches.sh."
    exit_code = 1 if fail_on_warn else 0
else:
    status = "HEALTHY_WITHIN_BUDGET"
    status_message = f"Podman storage footprint ({format_bytes(podman_used_bytes)} / {ceiling_gb:.1f} GB, {used_percent:.1f}%) is within budget."
    exit_code = 0

if json_output:
    payload = {
        "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "status": status,
        "exit_code": exit_code,
        "status_message": status_message,
        "ceiling_gb": ceiling_gb,
        "ceiling_bytes": ceiling_bytes,
        "used_bytes": podman_used_bytes,
        "used_percent": used_percent,
        "reclaimable_bytes": podman_reclaimable_bytes,
        "host": {
            "cpu_cores": cpu_cores,
            "total_mem_bytes": total_mem_bytes,
            "total_mem_formatted": format_bytes(total_mem_bytes),
            "free_mem_bytes": free_mem_bytes,
            "free_mem_formatted": format_bytes(free_mem_bytes),
            "drive_free_bytes": drive_free_bytes,
            "drive_free_formatted": format_bytes(drive_free_bytes),
            "drive_total_bytes": drive_total_bytes,
            "drive_total_formatted": format_bytes(drive_total_bytes)
        },
        "podman": {
            "installed": podman_installed,
            "version": podman_version,
            "breakdown": breakdown
        }
    }
    print(json.dumps(payload, indent=2))
    sys.exit(exit_code)

if not quiet_output:
    print("=" * 72)
    print("  web-repo-to-ai Resource Monitor & 30 GB Podman Storage Ceiling Check  ")
    print("=" * 72)
    print(f"Host CPU Cores     : {cpu_cores}")
    print(f"Host RAM (Total/Free): {format_bytes(total_mem_bytes)} / {format_bytes(free_mem_bytes)}")
    print(f"Host Disk Space    : {format_bytes(drive_free_bytes)} free / {format_bytes(drive_total_bytes)} total")
    print(f"Podman Installed   : {podman_installed} ({podman_version})")
    print("-" * 72)
    print("Podman Storage Usage Breakdown:")
    if breakdown:
        for b in breakdown:
            print(f"  - {b['Type']:<14} : {b['Size']:<10} (Count: {b['TotalCount']}, Reclaimable: {b['Reclaimable']})")
    else:
        print("  (No Podman storage items found or Podman not initialized)")
    print("-" * 72)
    print(f"Total Podman Usage : {format_bytes(podman_used_bytes)} / {ceiling_gb:.1f} GB ({used_percent:.1f}%)")
    print(f"Reclaimable Space  : {format_bytes(podman_reclaimable_bytes)}")
    print(f"Status             : [{status}]")
    print(f"Message            : {status_message}")
    print("=" * 72)
else:
    print(f"[{status}] {status_message}")

sys.exit(exit_code)
EOF
