#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_start_integ_final.sh --feature-dir <path> --slice-id <slice> [options]

Required:
  --feature-dir <path>   Feature Planning Pack dir (docs/project_management/next/<feature> or equivalent)
  --slice-id <slice>     Slice prefix (e.g., C0)

Options:
  --launch-codex         Launch Codex headless after worktree creation
  --codex-profile <p>    Codex profile (passed to `codex exec --profile`)
  --codex-model <m>      Codex model (passed to `codex exec --model`)
  --codex-jsonl          Also capture Codex JSONL events to a file (uses `codex exec --json`)
  --dry-run              Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  ORCH_BRANCH=<branch>
  SLICE_ID=<slice>
  FINAL_TASK_ID=<id>
  WORKTREE=<path>
  TASK_BRANCH=<branch>
  KICKOFF_PROMPT=<path>
  NEXT=<recommended next command>

Guardrails:
  - Requires an automation-enabled planning pack (tasks.json meta.schema_version>=3 and meta.automation.enabled=true).
  - Requires the final integration task (<slice>-integ) to set merge_to_orchestration=true.
  - Refuses to start final integration unless all of its depends_on tasks are status=completed (for deterministic closure).

Notes:
  - Runs from the orchestration worktree (or repo root) and uses:
    - `scripts/triad/orch_ensure.sh`
    - `scripts/triad/task_start.sh`
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
LAUNCH_CODEX=0
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
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
        --launch-codex)
            LAUNCH_CODEX=1
            shift 1
            ;;
        --codex-profile)
            CODEX_PROFILE="${2:-}"
            shift 2
            ;;
        --codex-model)
            CODEX_MODEL="${2:-}"
            shift 2
            ;;
        --codex-jsonl)
            CODEX_JSONL=1
            shift 1
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
    die "task_start_integ_final requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true"
fi
if [[ -z "${ORCH_BRANCH}" ]]; then
    die "tasks.json must include meta.automation.orchestration_branch"
fi

FINAL_TASK_ID="${SLICE_ID}-integ"
final_json="$(jq -c --arg id "${FINAL_TASK_ID}" '.tasks[] | select(.id==$id)' "${TASKS_JSON}")" || true
if [[ -z "${final_json}" ]]; then
    die "Final integration task not found: ${FINAL_TASK_ID}"
fi
if [[ "$(jq -r '.type' <<<"${final_json}")" != "integration" ]]; then
    die "Final integration task ${FINAL_TASK_ID} must have type=integration"
fi
merge_to_orch="$(jq -r '.merge_to_orchestration // empty' <<<"${final_json}")"
if [[ "${merge_to_orch}" != "true" ]]; then
    die "Final integration task ${FINAL_TASK_ID} must set merge_to_orchestration=true"
fi

log "Ensuring orchestration branch exists/checked out: ${ORCH_BRANCH}"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" --dry-run >/dev/null
else
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" >/dev/null
fi

log "Validating depends_on statuses for ${FINAL_TASK_ID}"
python3 - <<PY
import json
import sys

tasks_path = ${TASKS_JSON!r}
final_id = ${FINAL_TASK_ID!r}

with open(tasks_path, "r", encoding="utf-8") as f:
    data = json.load(f)

tasks = {t.get("id"): t for t in data.get("tasks", []) if isinstance(t, dict) and isinstance(t.get("id"), str)}
final = tasks.get(final_id)
if not final:
    print(f"ERROR: task not found: {final_id}", file=sys.stderr)
    raise SystemExit(2)

deps = final.get("depends_on") or []
if not isinstance(deps, list):
    print(f"ERROR: {final_id}.depends_on must be an array", file=sys.stderr)
    raise SystemExit(2)

missing = []
not_done = []
for dep in deps:
    t = tasks.get(dep)
    if t is None:
        # external deps allowed; task_start_integ_final can't validate them.
        continue
    status = t.get("status")
    if status != "completed":
        not_done.append((dep, status))

if not_done:
    for dep, status in not_done:
        print(f"ERROR: cannot start {final_id}: depends_on {dep} is not completed (status={status!r})", file=sys.stderr)
    raise SystemExit(2)
PY

log "Starting final integration task worktree: ${FINAL_TASK_ID}"
args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${FINAL_TASK_ID}")
if [[ "${LAUNCH_CODEX}" -eq 1 ]]; then args+=(--launch-codex); fi
if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi
if [[ "${DRY_RUN}" -eq 1 ]]; then args+=(--dry-run); fi

out="$("${args[@]}")"

printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'SLICE_ID=%s\n' "${SLICE_ID}"
printf 'FINAL_TASK_ID=%s\n' "${FINAL_TASK_ID}"
printf '%s\n' "${out}"

