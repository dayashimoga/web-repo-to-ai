import glob
import sys
import yaml

def main():
    workflow_files = sorted(glob.glob(".github/workflows/*.yml") + glob.glob(".github/workflows/*.yaml"))
    if not workflow_files:
        print("ERROR: No workflow files found in .github/workflows/")
        sys.exit(1)

    errors = 0
    for file_path in workflow_files:
        try:
            with open(file_path, "r", encoding="utf-8") as f:
                data = yaml.safe_load(f)
                name = data.get("name", "unnamed")
                jobs = list(data.get("jobs", {}).keys())
                on_block = data.get("on") if "on" in data else data.get(True)
                triggers = list(on_block.keys()) if isinstance(on_block, dict) else on_block
                permissions = data.get("permissions")
                print(f"[PASS] {file_path}")
                print(f"       Name: {name}")
                print(f"       Triggers: {triggers}")
                print(f"       Permissions: {permissions}")
                print(f"       Jobs: {jobs}")
        except Exception as e:
            print(f"[FAIL] {file_path}: {e}")
            errors += 1

    print(f"\nSummary: {len(workflow_files) - errors}/{len(workflow_files)} workflows passed.")
    if errors > 0:
        sys.exit(1)

if __name__ == "__main__":
    main()
