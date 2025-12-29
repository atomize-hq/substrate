#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/planning/lint.sh --feature-dir docs/project_management/next/<feature>

Notes:
  - This is the mechanical Planning Pack lint runner used by the quality gate reviewer.
  - It is intentionally strict and exits non-zero on any violation.
USAGE
}

FEATURE_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown arg: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -z "${FEATURE_DIR}" ]]; then
    echo "Missing --feature-dir" >&2
    usage >&2
    exit 2
fi

if [[ ! -d "${FEATURE_DIR}" ]]; then
    echo "Feature dir does not exist: ${FEATURE_DIR}" >&2
    exit 2
fi

echo "== Planning lint: ${FEATURE_DIR} =="

require_path() {
    local p="$1"
    if [[ ! -e "$p" ]]; then
        echo "Missing required path: $p" >&2
        exit 1
    fi
}

require_path "${FEATURE_DIR}/plan.md"
require_path "${FEATURE_DIR}/tasks.json"
require_path "${FEATURE_DIR}/session_log.md"
require_path "${FEATURE_DIR}/kickoff_prompts"

if [[ -d "${FEATURE_DIR}/smoke" ]]; then
    require_path "${FEATURE_DIR}/smoke/linux-smoke.sh"
    require_path "${FEATURE_DIR}/smoke/macos-smoke.sh"
    require_path "${FEATURE_DIR}/smoke/windows-smoke.ps1"
fi

echo "-- Hard-ban scan"
if rg -n --hidden --glob '!**/.git/**' '\b(TBD|TODO|WIP|TBA)\b|open question|\betc\.|and so on' "${FEATURE_DIR}"; then
    echo "FAIL: hard-ban matches found (remove these from planning outputs)" >&2
    exit 1
else
    rc=$?
    if [[ "${rc}" -ne 1 ]]; then
        exit "${rc}"
    fi
fi

echo "-- Ambiguity scan"
if rg -n --hidden --glob '!**/.git/**' --glob '!decision_register.md' '\b(should|could|might|maybe|optionally|optional)\b' "${FEATURE_DIR}"; then
    echo "FAIL: ambiguity-word matches found (rewrite behavioral contracts to be singular/testable)" >&2
    exit 1
else
    rc=$?
    if [[ "${rc}" -ne 1 ]]; then
        exit "${rc}"
    fi
fi

echo "-- JSON validity"
jq -e . "${FEATURE_DIR}/tasks.json" >/dev/null
jq -e . docs/project_management/next/sequencing.json >/dev/null

echo "-- tasks.json invariants"
python3 scripts/planning/validate_tasks_json.py --feature-dir "${FEATURE_DIR}"

echo "-- Kickoff prompt sentinel"
missing=0
while IFS= read -r -d '' f; do
    if ! rg -q 'Do not edit planning docs inside the worktree\.' "$f"; then
        echo "Missing sentinel in kickoff prompt: $f" >&2
        missing=1
    fi
done < <(find "${FEATURE_DIR}/kickoff_prompts" -maxdepth 1 -type f -name '*.md' -print0)
if [[ "${missing}" -ne 0 ]]; then
    exit 1
fi

echo "-- Manual playbook smoke linkage (if present)"
if [[ -f "${FEATURE_DIR}/manual_testing_playbook.md" ]]; then
    if [[ -d "${FEATURE_DIR}/smoke" ]]; then
        rg -n 'smoke/(linux-smoke\.sh|macos-smoke\.sh|windows-smoke\.ps1)' "${FEATURE_DIR}/manual_testing_playbook.md" >/dev/null
    fi
fi

echo "-- Sequencing alignment"
jq -e --arg dir "${FEATURE_DIR}" '.sprints[] | select(.directory==$dir) | .id' docs/project_management/next/sequencing.json >/dev/null

echo "OK: planning lint passed"
