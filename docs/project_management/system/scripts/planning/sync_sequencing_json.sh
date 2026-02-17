#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  docs/project_management/system/scripts/planning/sync_sequencing_json.sh

Copies the canonical sequencing spine:
  docs/project_management/packs/sequencing.json
to the legacy compatibility mirror:
  docs/project_management/next/sequencing.json

Notes:
  - No symlinks (Windows compatibility); the legacy file is a real copy.
  - Fails if the canonical file is missing.
USAGE
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
    usage
    exit 0
fi

if ! command -v git >/dev/null 2>&1; then
    echo "FAIL: git is required" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null)" || {
    echo "FAIL: failed to locate repo root via git" >&2
    exit 2
}

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi

PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"

cd "${REPO_ROOT}"

roots_json="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" print-roots)"
PM_ROOT="$(python3 - <<'PY' "${roots_json}"
import json
import sys
print(json.loads(sys.argv[1])["pm_root"])
PY
)"
PM_PACKS_ROOT="$(python3 - <<'PY' "${roots_json}"
import json
import sys
print(json.loads(sys.argv[1])["pm_packs_root"])
PY
)"

canonical="${PM_PACKS_ROOT%/}/sequencing.json"
legacy="${PM_ROOT%/}/next/sequencing.json"

if [[ ! -f "${canonical}" ]]; then
    echo "FAIL: canonical sequencing.json missing: ${canonical}" >&2
    echo "Hint: expected packs root at ${PM_PACKS_ROOT} (override via PM_PACKS_ROOT)" >&2
    exit 2
fi

mkdir -p "$(dirname "${legacy}")"
cp "${canonical}" "${legacy}"

validate_json() {
    local path="$1"
    if command -v jq >/dev/null 2>&1; then
        jq -e . "${path}" >/dev/null
        return 0
    fi
    python3 - <<'PY' "${path}"
import json
import sys
from pathlib import Path

p = Path(sys.argv[1])
json.loads(p.read_text(encoding="utf-8"))
PY
}

validate_json "${canonical}"
validate_json "${legacy}"

echo "OK: synced ${canonical} -> ${legacy}"

