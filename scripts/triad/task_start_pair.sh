#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_start_pair.sh --feature-dir <path> (--slice-id <slice>|--code-task-id <id> --test-task-id <id>) [options]

Required:
  --feature-dir <path>     Feature Planning Pack dir (docs/project_management/next/<feature> or equivalent)
  One of:
    --slice-id <slice>     Slice prefix (e.g., C0 -> C0-code and C0-test)
    --code-task-id <id>    Code task id (type=code)
    --test-task-id <id>    Test task id (type=test)

Options:
  --launch-codex           Launch Codex headless for BOTH tasks (runs in parallel after worktrees are created)
  --codex-profile <p>      Codex profile (passed to `codex exec --profile`)
  --codex-model <m>        Codex model (passed to `codex exec --model`)
  --codex-jsonl            Capture Codex JSONL event stream (uses `codex exec --json`)
  --dry-run                Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  ORCH_BRANCH=<branch>
  CODE_TASK_ID=<id>
  TEST_TASK_ID=<id>
  CODE_WORKTREE=<path>
  TEST_WORKTREE=<path>
  CODE_TASK_BRANCH=<branch>
  TEST_TASK_BRANCH=<branch>
  CODEX_CODE_EXIT=<code or empty>
  CODEX_TEST_EXIT=<code or empty>
  CODEX_CODE_LAST_MESSAGE_PATH=<path>
  CODEX_TEST_LAST_MESSAGE_PATH=<path>
  CODEX_CODE_EVENTS_PATH=<path>
  CODEX_TEST_EVENTS_PATH=<path>
  CODEX_CODE_STDERR_PATH=<path>
  CODEX_TEST_STDERR_PATH=<path>

Notes:
  - Requires an automation-enabled planning pack (tasks.json meta.schema_version>=3 and meta.automation.enabled=true).
  - Uses the same underlying automation scripts as single-task start:
    - `scripts/triad/orch_ensure.sh`
    - `scripts/triad/task_start.sh`
  - Worktrees are retained until feature cleanup (FZ-feature-cleanup).
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
CODE_TASK_ID=""
TEST_TASK_ID=""
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
        --code-task-id)
            CODE_TASK_ID="${2:-}"
            shift 2
            ;;
        --test-task-id)
            TEST_TASK_ID="${2:-}"
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

if [[ -z "${FEATURE_DIR}" ]]; then
    usage >&2
    die "Missing --feature-dir"
fi

if [[ -n "${SLICE_ID}" ]]; then
    if [[ -n "${CODE_TASK_ID}" || -n "${TEST_TASK_ID}" ]]; then
        die "Use either --slice-id OR --code-task-id/--test-task-id, not both"
    fi
    CODE_TASK_ID="${SLICE_ID}-code"
    TEST_TASK_ID="${SLICE_ID}-test"
else
    if [[ -z "${CODE_TASK_ID}" || -z "${TEST_TASK_ID}" ]]; then
        usage >&2
        die "Missing --slice-id OR both --code-task-id and --test-task-id"
    fi
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
FEATURE_NAME="$(jq -r '.meta.feature // empty' "${TASKS_JSON}")"

if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "task_start_pair requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true"
fi
if [[ -z "${ORCH_BRANCH}" || -z "${FEATURE_NAME}" ]]; then
    die "tasks.json must include meta.feature and meta.automation.orchestration_branch"
fi

task_type_for() {
    local id="$1"
    jq -r --arg id "${id}" '.tasks[] | select(.id==$id) | .type' "${TASKS_JSON}"
}

if [[ "$(task_type_for "${CODE_TASK_ID}")" != "code" ]]; then
    die "Task ${CODE_TASK_ID} is not type=code"
fi
if [[ "$(task_type_for "${TEST_TASK_ID}")" != "test" ]]; then
    die "Task ${TEST_TASK_ID} is not type=test"
fi

task_integration_task_for() {
    local id="$1"
    jq -r --arg id "${id}" '.tasks[] | select(.id==$id) | .integration_task // empty' "${TASKS_JSON}"
}

task_concurrent_with_for() {
    local id="$1"
    jq -r --arg id "${id}" '.tasks[] | select(.id==$id) | .concurrent_with[]?' "${TASKS_JSON}"
}

code_integration_task="$(task_integration_task_for "${CODE_TASK_ID}")"
test_integration_task="$(task_integration_task_for "${TEST_TASK_ID}")"
if [[ -z "${code_integration_task}" || -z "${test_integration_task}" ]]; then
    die "Both ${CODE_TASK_ID} and ${TEST_TASK_ID} must have integration_task set"
fi
if [[ "${code_integration_task}" != "${test_integration_task}" ]]; then
    die "Mismatched integration_task: ${CODE_TASK_ID} -> ${code_integration_task}, ${TEST_TASK_ID} -> ${test_integration_task}"
fi

if ! task_concurrent_with_for "${CODE_TASK_ID}" | grep -Fxq "${TEST_TASK_ID}"; then
    die "${CODE_TASK_ID}.concurrent_with must include ${TEST_TASK_ID} (parallel code/test is required)"
fi
if ! task_concurrent_with_for "${TEST_TASK_ID}" | grep -Fxq "${CODE_TASK_ID}"; then
    die "${TEST_TASK_ID}.concurrent_with must include ${CODE_TASK_ID} (parallel code/test is required)"
fi

require_deps_completed() {
    local id="$1"
    python3 - "${TASKS_JSON}" "${id}" <<'PY'
import json
import sys

tasks_path = sys.argv[1]
task_id = sys.argv[2]

with open(tasks_path, "r", encoding="utf-8") as f:
  data = json.load(f)

tasks = {t.get("id"): t for t in data.get("tasks", []) if isinstance(t, dict) and isinstance(t.get("id"), str)}
task = tasks.get(task_id)
if not task:
  print(f"ERROR: task not found: {task_id}", file=sys.stderr)
  raise SystemExit(2)

deps = task.get("depends_on") or []
if not isinstance(deps, list):
  print(f"ERROR: {task_id}.depends_on must be an array", file=sys.stderr)
  raise SystemExit(2)

for dep in deps:
  dep_task = tasks.get(dep)
  if not dep_task:
    # external deps are allowed via meta.external_task_ids; task_start_pair just won't validate them.
    continue
  if dep_task.get("status") != "completed":
    print(f"ERROR: dependency not completed: {task_id} depends_on {dep} (status={dep_task.get('status')!r})", file=sys.stderr)
    raise SystemExit(2)
PY
}

require_deps_completed "${CODE_TASK_ID}"
require_deps_completed "${TEST_TASK_ID}"

log "Ensuring orchestration branch exists/checked out: ${ORCH_BRANCH}"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" --dry-run >/dev/null
else
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" >/dev/null
fi

parse_kv() {
    local key="$1"
    local text="$2"
    printf '%s\n' "${text}" | awk -F= -v k="${key}" '$1==k { sub(/^[^=]*=/, "", $0); print $0; exit }'
}

log "Creating code worktree: ${CODE_TASK_ID}"
code_args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${CODE_TASK_ID}")
if [[ "${DRY_RUN}" -eq 1 ]]; then code_args+=(--dry-run); fi
if [[ -n "${CODEX_PROFILE}" ]]; then code_args+=(--codex-profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then code_args+=(--codex-model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then code_args+=(--codex-jsonl); fi
code_out="$("${code_args[@]}")"

log "Creating test worktree: ${TEST_TASK_ID}"
test_args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${TEST_TASK_ID}")
if [[ "${DRY_RUN}" -eq 1 ]]; then test_args+=(--dry-run); fi
if [[ -n "${CODEX_PROFILE}" ]]; then test_args+=(--codex-profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then test_args+=(--codex-model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then test_args+=(--codex-jsonl); fi
test_out="$("${test_args[@]}")"

CODE_WORKTREE="$(parse_kv WORKTREE "${code_out}")"
TEST_WORKTREE="$(parse_kv WORKTREE "${test_out}")"
CODE_TASK_BRANCH="$(parse_kv TASK_BRANCH "${code_out}")"
TEST_TASK_BRANCH="$(parse_kv TASK_BRANCH "${test_out}")"
CODE_KICKOFF="$(parse_kv KICKOFF_PROMPT "${code_out}")"
TEST_KICKOFF="$(parse_kv KICKOFF_PROMPT "${test_out}")"

if [[ -z "${CODE_WORKTREE}" || -z "${TEST_WORKTREE}" ]]; then
    die "Failed to parse WORKTREE from task_start output"
fi

CODEX_CODE_EXIT=""
CODEX_TEST_EXIT=""
CODEX_PAIR_FAILED=0

CODEX_CODE_OUT_DIR="${REPO_ROOT}/target/triad/${FEATURE_NAME}/codex/${CODE_TASK_ID}"
CODEX_TEST_OUT_DIR="${REPO_ROOT}/target/triad/${FEATURE_NAME}/codex/${TEST_TASK_ID}"
CODEX_CODE_LAST_MESSAGE_PATH="${CODEX_CODE_OUT_DIR}/last_message.md"
CODEX_TEST_LAST_MESSAGE_PATH="${CODEX_TEST_OUT_DIR}/last_message.md"
CODEX_CODE_EVENTS_PATH="${CODEX_CODE_OUT_DIR}/events.jsonl"
CODEX_TEST_EVENTS_PATH="${CODEX_TEST_OUT_DIR}/events.jsonl"
CODEX_CODE_STDERR_PATH="${CODEX_CODE_OUT_DIR}/stderr.log"
CODEX_TEST_STDERR_PATH="${CODEX_TEST_OUT_DIR}/stderr.log"

launch_codex_one() {
    local worktree="$1"
    local kickoff="$2"
    local out_dir="$3"
    local last_message="$4"
    local events="$5"
    local stderr="$6"

    mkdir -p "${out_dir}"

    codex_args=(codex exec --dangerously-bypass-approvals-and-sandbox --cd "${worktree}")
    if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
    codex_args+=(--output-last-message "${last_message}" -)
    "${codex_args[@]}" < "${kickoff}" >"${events}" 2>"${stderr}"
}

if [[ "${LAUNCH_CODEX}" -eq 1 ]]; then
    require_cmd codex
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log "DRY_RUN=1: skipping codex exec"
        CODEX_CODE_EXIT="dry-run"
        CODEX_TEST_EXIT="dry-run"
    else
        log "Launching Codex headless for both tasks (in parallel)"
        set +e
        launch_codex_one "${CODE_WORKTREE}" "${CODE_KICKOFF}" "${CODEX_CODE_OUT_DIR}" "${CODEX_CODE_LAST_MESSAGE_PATH}" "${CODEX_CODE_EVENTS_PATH}" "${CODEX_CODE_STDERR_PATH}" &
        pid_code=$!
        launch_codex_one "${TEST_WORKTREE}" "${TEST_KICKOFF}" "${CODEX_TEST_OUT_DIR}" "${CODEX_TEST_LAST_MESSAGE_PATH}" "${CODEX_TEST_EVENTS_PATH}" "${CODEX_TEST_STDERR_PATH}" &
        pid_test=$!

        wait "${pid_code}"
        CODEX_CODE_EXIT="$?"
        wait "${pid_test}"
        CODEX_TEST_EXIT="$?"
        set -e

        if [[ "${CODEX_CODE_EXIT}" -ne 0 || "${CODEX_TEST_EXIT}" -ne 0 ]]; then
            CODEX_PAIR_FAILED=1
        fi
    fi
fi

printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'CODE_TASK_ID=%s\n' "${CODE_TASK_ID}"
printf 'TEST_TASK_ID=%s\n' "${TEST_TASK_ID}"
printf 'CODE_WORKTREE=%s\n' "${CODE_WORKTREE}"
printf 'TEST_WORKTREE=%s\n' "${TEST_WORKTREE}"
printf 'CODE_TASK_BRANCH=%s\n' "${CODE_TASK_BRANCH}"
printf 'TEST_TASK_BRANCH=%s\n' "${TEST_TASK_BRANCH}"
printf 'CODEX_CODE_EXIT=%s\n' "${CODEX_CODE_EXIT}"
printf 'CODEX_TEST_EXIT=%s\n' "${CODEX_TEST_EXIT}"
printf 'CODEX_CODE_LAST_MESSAGE_PATH=%s\n' "${CODEX_CODE_LAST_MESSAGE_PATH}"
printf 'CODEX_TEST_LAST_MESSAGE_PATH=%s\n' "${CODEX_TEST_LAST_MESSAGE_PATH}"
printf 'CODEX_CODE_EVENTS_PATH=%s\n' "${CODEX_CODE_EVENTS_PATH}"
printf 'CODEX_TEST_EVENTS_PATH=%s\n' "${CODEX_TEST_EVENTS_PATH}"
printf 'CODEX_CODE_STDERR_PATH=%s\n' "${CODEX_CODE_STDERR_PATH}"
printf 'CODEX_TEST_STDERR_PATH=%s\n' "${CODEX_TEST_STDERR_PATH}"

if [[ "${CODEX_PAIR_FAILED}" -eq 1 ]]; then
    echo "ERROR: One or both codex exec runs failed (code=${CODEX_CODE_EXIT}, test=${CODEX_TEST_EXIT})" >&2
    exit 2
fi
