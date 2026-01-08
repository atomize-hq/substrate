#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/mark_noop_platform_fixes_completed.sh --feature-dir <path> --slice-id <slice> [--from-smoke-run <id>] [--dry-run]

Purpose:
  Mark <slice>-integ-<platform> platform-fix tasks as status=completed (no-ops) on the orchestration branch,
  typically when PLATFORM=all smoke is green and no platform-fix worktrees/branches are expected.

Required:
  --feature-dir <path>     Feature Planning Pack dir (docs/project_management/next/<feature> or equivalent)
  --slice-id <slice>       Slice prefix (e.g., PCP0)

Optional:
  --from-smoke-run <id>    Smoke run id to record in stdout (does not query GitHub; informational only)
  --dry-run                Print what would change; do not modify tasks.json

Stdout contract (machine-parseable):
  ORCH_BRANCH=<branch>
  SLICE_ID=<slice>
  SMOKE_RUN_ID=<id or empty>
  UPDATED_TASK_ID=<id>     (repeated; one per task updated)
  SKIPPED_TASK_ID=<id>     (repeated; one per task skipped)
  TASKS_JSON=<path>
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

log() {
    echo "== $*" >&2
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Missing dependency: $1"
    fi
}

python_abs_path() {
    python3 - "$1" <<'PY'
import os
import sys

p = sys.argv[1]
if os.path.isabs(p):
    print(os.path.realpath(p))
else:
    print(os.path.realpath(os.path.join(os.getcwd(), p)))
PY
}

FEATURE_DIR=""
SLICE_ID=""
SMOKE_RUN_ID=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --slice-id)
            SLICE_ID="${2:-}"
            shift 2
            ;;
        --from-smoke-run)
            SMOKE_RUN_ID="${2:-}"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=1
            shift 1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "Unknown arg: $1"
            ;;
    esac
done

if [[ -z "${FEATURE_DIR}" || -z "${SLICE_ID}" ]]; then
    usage >&2
    die "Missing required args"
fi

require_cmd git
require_cmd jq
require_cmd python3

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi

SCHEMA_VERSION="$(jq -r '.meta.schema_version // 1' "${TASKS_JSON}")"
AUTOMATION_ENABLED="$(jq -r '.meta.automation.enabled // false' "${TASKS_JSON}")"
ORCH_BRANCH="$(jq -r '.meta.automation.orchestration_branch // empty' "${TASKS_JSON}")"

if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "This helper requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true"
fi
if [[ -z "${ORCH_BRANCH}" ]]; then
    die "tasks.json must include meta.automation.orchestration_branch"
fi

log "Ensuring orchestration branch exists/checked out: ${ORCH_BRANCH}"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" --dry-run >/dev/null
else
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" >/dev/null
fi

platforms_required="$(jq -r '.meta.ci_parity_platforms_required // .meta.platforms_required // [] | join(",")' "${TASKS_JSON}")"
wsl_required="$(jq -r '.meta.wsl_required // false' "${TASKS_JSON}")"
wsl_task_mode="$(jq -r '.meta.wsl_task_mode // "bundled"' "${TASKS_JSON}")"

if [[ -z "${platforms_required}" ]]; then
    die "tasks.json meta.ci_parity_platforms_required (or legacy meta.platforms_required) is empty; cannot infer platform-fix task ids"
fi
if [[ "${wsl_required}" == "true" && "${wsl_task_mode}" == "separate" ]]; then
    platforms_required="${platforms_required},wsl"
fi

printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'SLICE_ID=%s\n' "${SLICE_ID}"
printf 'SMOKE_RUN_ID=%s\n' "${SMOKE_RUN_ID}"
printf 'TASKS_JSON=%s\n' "${TASKS_JSON}"

IFS=',' read -r -a platforms <<<"${platforms_required}"
for p in "${platforms[@]}"; do
    p="$(echo "${p}" | xargs)"
    [[ -z "${p}" ]] && continue
    case "${p}" in
        linux|macos|windows|wsl) ;;
        *) die "Invalid platform in meta.ci_parity_platforms_required (or legacy meta.platforms_required): ${p}" ;;
    esac

    task_id="${SLICE_ID}-integ-${p}"
    if ! jq -e --arg id "${task_id}" '.tasks[] | select(.id==$id)' "${TASKS_JSON}" >/dev/null; then
        printf 'SKIPPED_TASK_ID=%s\n' "${task_id}"
        continue
    fi

    current_status="$(jq -r --arg id "${task_id}" '.tasks[] | select(.id==$id) | .status' "${TASKS_JSON}")"
    if [[ "${current_status}" == "completed" ]]; then
        printf 'SKIPPED_TASK_ID=%s\n' "${task_id}"
        continue
    fi

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf 'UPDATED_TASK_ID=%s\n' "${task_id}"
        continue
    fi

    python3 - "${TASKS_JSON}" "${task_id}" <<'PY'
import json
import sys

tasks_path = sys.argv[1]
task_id = sys.argv[2]

with open(tasks_path, "r", encoding="utf-8") as f:
    data = json.load(f)

tasks = data.get("tasks") or []
updated = False
for t in tasks:
    if isinstance(t, dict) and t.get("id") == task_id:
        t["status"] = "completed"
        updated = True
        break

if not updated:
    print(f"ERROR: task not found: {task_id}", file=sys.stderr)
    raise SystemExit(2)

tmp = tasks_path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, sort_keys=False)
    f.write("\n")
import os
os.replace(tmp, tasks_path)
PY
    printf 'UPDATED_TASK_ID=%s\n' "${task_id}"
done
