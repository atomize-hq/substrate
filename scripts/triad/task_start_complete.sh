#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_start_complete.sh --feature-dir <path> --slice-id <slice> [options]

Required:
  --feature-dir <path>   Feature Planning Pack dir (docs/project_management/next/<feature> or equivalent)
  --slice-id <slice>     Slice id (e.g., WCU0)

Options:
  --codex-profile <p>    Codex profile (passed to `codex exec --profile`)
  --codex-model <m>      Codex model (passed to `codex exec --model`)
  --codex-jsonl          Capture Codex JSONL event stream (uses `codex exec --json`)
  --dry-run              Print what would happen; do not mutate git/worktrees

Behavior:
  - Ensures orchestration branch is checked out and clean.
  - Marks code/test tasks in_progress in tasks.json + appends START entries to session_log.md (commits).
  - Starts code+test in parallel with Codex enabled.
  - Finishes code+test worktrees via triad-task-finish (commits on task branches).
  - Marks code/test tasks completed in tasks.json + appends END entries (commits).
  - Starts the slice’s integration merge task (determined from <slice>-code.integration_task), with Codex enabled.
  - Finishes the integration worktree via triad-task-finish (commits + merges to orchestration when configured).
  - Marks the integration task completed in tasks.json + appends END entry (commits).
  - For schema v4+ checkpointed cross-platform packs: on checkpoint-boundary slices, the integration task is expected to be <slice>-integ-core.
    - This wrapper does not run any CI checkpoint ops task (CPk-ci-checkpoint), does not start any platform-fix tasks (<slice>-integ-<platform>), and does not start final aggregation (<slice>-integ).
    - It emits NEXT_CHECKPOINT_TASK_ID so the operator can run the planned checkpoint next.

Outputs:
  - Writes a log + JSON summary under <feature_dir>/logs/<slice>/wrapper/

Stdout contract (machine-parseable):
  SLICE_ID=<slice>
  CODE_TASK_ID=<id>
  TEST_TASK_ID=<id>
  INTEG_TASK_ID=<id>
  BOUNDARY_SLICE=0|1
  NEXT_CHECKPOINT_TASK_ID=<id or empty>
  LOG_PATH=<path>
  SUMMARY_JSON_PATH=<path>
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
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

parse_kv() {
    local key="$1"
    local src="$2"
    # First exact match, strip "KEY=".
    rg -m 1 "^${key}=" "${src}" | sed -E "s/^${key}=//"
}

FEATURE_DIR=""
SLICE_ID=""
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
require_cmd rg
require_cmd codex

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
SESSION_LOG="${FEATURE_DIR_ABS}/session_log.md"
CI_CHECKPOINT_PLAN="${FEATURE_DIR_ABS}/ci_checkpoint_plan.md"

if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi
if [[ ! -f "${SESSION_LOG}" ]]; then
    die "Missing session_log.md: ${SESSION_LOG}"
fi

OUT_DIR="${FEATURE_DIR_ABS}/logs/${SLICE_ID}/wrapper"
ts="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_PATH="${OUT_DIR}/triad-task-start-complete.${ts}.log"
SUMMARY_JSON_PATH="${OUT_DIR}/triad-task-start-complete.${ts}.summary.json"
mkdir -p "${OUT_DIR}"

exec 3>>"${LOG_PATH}"

log_to_file() {
    printf '%s\n' "$*" >&3
}

log_to_file "== triad-task-start-complete ${ts} =="
log_to_file "FEATURE_DIR=${FEATURE_DIR_ABS}"
log_to_file "SLICE_ID=${SLICE_ID}"

CODE_TASK_ID="${SLICE_ID}-code"
TEST_TASK_ID="${SLICE_ID}-test"

INTEG_TASK_ID="$(jq -r --arg id "${CODE_TASK_ID}" '.tasks[] | select(.id==$id) | .integration_task // empty' "${TASKS_JSON}")"
if [[ -z "${INTEG_TASK_ID}" ]]; then
    die "Could not determine integration task id from ${CODE_TASK_ID}.integration_task in tasks.json"
fi

BOUNDARY_SLICE=0
NEXT_CHECKPOINT_TASK_ID=""
if jq -e '.meta.checkpoint_boundaries' "${TASKS_JSON}" >/dev/null 2>&1; then
    if jq -e --arg s "${SLICE_ID}" '.meta.checkpoint_boundaries | index($s) != null' "${TASKS_JSON}" >/dev/null; then
        BOUNDARY_SLICE=1
    fi
fi
if [[ "${BOUNDARY_SLICE}" -eq 1 && -f "${CI_CHECKPOINT_PLAN}" ]]; then
    NEXT_CHECKPOINT_TASK_ID="$(
        python3 - "${CI_CHECKPOINT_PLAN}" "${SLICE_ID}" <<'PY'
import json
import re
import sys
from pathlib import Path

plan_path = Path(sys.argv[1])
slice_id = sys.argv[2]
try:
    text = plan_path.read_text(encoding="utf-8")
except OSError:
    print("")
    raise SystemExit(0)

header = "## Machine-readable plan (linted)"
start = text.find(header)
if start < 0:
    print("")
    raise SystemExit(0)
remainder = text[start:]
m = re.search(r"```json\s*\n(?P<body>[\s\S]*?)\n```", remainder)
if not m:
    print("")
    raise SystemExit(0)

try:
    data = json.loads(m.group("body").strip())
except json.JSONDecodeError:
    print("")
    raise SystemExit(0)

checkpoints = data.get("checkpoints") or []
for c in checkpoints:
    if not isinstance(c, dict):
        continue
    slices = c.get("slices") or []
    if slices and slices[-1] == slice_id:
        task_id = c.get("task_id") or ""
        print(task_id)
        raise SystemExit(0)
print("")
raise SystemExit(0)
PY
	    )"
fi

log_to_file "CODE_TASK_ID=${CODE_TASK_ID}"
log_to_file "TEST_TASK_ID=${TEST_TASK_ID}"
log_to_file "INTEG_TASK_ID=${INTEG_TASK_ID}"
log_to_file "BOUNDARY_SLICE=${BOUNDARY_SLICE}"
log_to_file "NEXT_CHECKPOINT_TASK_ID=${NEXT_CHECKPOINT_TASK_ID}"

log_to_file "-- orch_ensure"
if [[ "${DRY_RUN}" -eq 1 ]]; then
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" --dry-run 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
else
    scripts/triad/orch_ensure.sh --feature-dir "${FEATURE_DIR_ABS}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
fi

if [[ "${DRY_RUN}" -ne 1 ]]; then
    if ! git diff --quiet || ! git diff --cached --quiet; then
        die "Orchestration checkout is not clean after orch_ensure; commit/stash before running"
    fi
fi

log_to_file "-- precheck: code/test deps must be completed"
python3 - "${TASKS_JSON}" "${CODE_TASK_ID}" "${TEST_TASK_ID}" <<'PY'
import json
import sys
from pathlib import Path

tasks_path = Path(sys.argv[1])
code_id = sys.argv[2]
test_id = sys.argv[3]

data = json.loads(tasks_path.read_text(encoding="utf-8"))
tasks = data.get("tasks") or []
by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}

code = by_id.get(code_id)
test = by_id.get(test_id)
if not code or not test:
    missing = code_id if not code else test_id
    print(f"ERROR: missing task in tasks.json: {missing}", file=sys.stderr)
    raise SystemExit(2)

code_status = code.get("status")
test_status = test.get("status")
if code_status != test_status:
    print(
        f"ERROR: code/test statuses must match for {code_id}/{test_id} (code={code_status!r}, test={test_status!r})",
        file=sys.stderr,
    )
    raise SystemExit(2)
if code_status not in ("pending", "in_progress", "completed"):
    print(
        f"ERROR: {code_id}/{test_id} invalid status for wrapper (status={code_status!r})",
        file=sys.stderr,
    )
    raise SystemExit(2)

for tid in (code_id, test_id):
    t = by_id.get(tid)
    deps = t.get("depends_on") or []
    if not isinstance(deps, list):
        print(f"ERROR: {tid}.depends_on must be an array", file=sys.stderr)
        raise SystemExit(2)
    for dep in deps:
        dep_task = by_id.get(dep)
        if dep_task is None:
            # External deps allowed; wrapper can't validate them.
            continue
        dep_status = dep_task.get("status")
        if dep_status != "completed":
            print(f"ERROR: cannot start {tid}: depends_on {dep} is not completed (status={dep_status!r})", file=sys.stderr)
            raise SystemExit(2)
PY

now_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

CODE_TEST_STATUS="$(jq -r --arg id "${CODE_TASK_ID}" '.tasks[] | select(.id==$id) | .status' "${TASKS_JSON}")"
CODE_TASK_BRANCH="$(jq -r --arg id "${CODE_TASK_ID}" '.tasks[] | select(.id==$id) | .git_branch' "${TASKS_JSON}")"
TEST_TASK_BRANCH="$(jq -r --arg id "${TEST_TASK_ID}" '.tasks[] | select(.id==$id) | .git_branch' "${TASKS_JSON}")"

CODEX_CODE_LAST_MESSAGE_PATH="${FEATURE_DIR_ABS}/logs/${SLICE_ID}/code/last_message.md"
CODEX_TEST_LAST_MESSAGE_PATH="${FEATURE_DIR_ABS}/logs/${SLICE_ID}/test/last_message.md"

CODE_WORKTREE=""
TEST_WORKTREE=""
pair_stdout=""
code_finish_out=""
test_finish_out=""

CODE_HEAD_SHA=""
TEST_HEAD_SHA=""

finish_one() {
    local task_id="$1"
    local worktree="$2"
    local out="$3"

    log_to_file "-- finish ${task_id} (${worktree})"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf 'TASK_BRANCH=dry-run\nWORKTREE=%s\nHEAD=dry-run\nCOMMITS=dry-run\nCHECKS=dry-run\nSMOKE_RUN=\nMERGED_TO_ORCH=\n' "${worktree}" >"${out}"
        return 0
    fi
    (cd "${worktree}" && make triad-task-finish TASK_ID="${task_id}") >"${out}" 2>>"${LOG_PATH}"
    cat "${out}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
}

task_logs_dir_for() {
    local task_id="$1"
    local slice="${task_id%%-*}"
    local kind="${task_id#${slice}-}"
    if [[ -z "${slice}" || -z "${kind}" || "${slice}" == "${task_id}" || "${kind}" == "${task_id}" ]]; then
        slice="${task_id}"
        kind="task"
    fi
    printf '%s/logs/%s/%s\n' "${FEATURE_DIR_ABS}" "${slice}" "${kind}"
}

wait_for_codex_if_running() {
    local task_id="$1"

    local out_dir
    out_dir="$(task_logs_dir_for "${task_id}")"
    local pid_path="${out_dir}/codex.pid"
    if [[ ! -f "${pid_path}" ]]; then
        return 0
    fi

    local pid
    pid="$(tr -d '[:space:]' < "${pid_path}" || true)"
    if [[ -z "${pid}" ]]; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    if ! kill -0 "${pid}" 2>/dev/null; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    local worktree_rel
    worktree_rel="$(jq -r --arg id "${task_id}" '.tasks[] | select(.id==$id) | .worktree // empty' "${TASKS_JSON}")"
    local worktree_abs=""
    if [[ -n "${worktree_rel}" && "${worktree_rel}" != "null" ]]; then
        worktree_abs="$(python_abs_path "${worktree_rel}")"
    fi

    local cmd
    cmd="$(ps -p "${pid}" -o cmd= 2>/dev/null || true)"
    if [[ -z "${cmd}" ]]; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    # Guard against PID reuse: only trust a live PID if it still looks like a Codex invocation.
    # If it doesn't, treat codex.pid as stale and remove it so we don't hang waiting on an unrelated process.
    if ! printf '%s\n' "${cmd}" | rg -qi -- '(^|[[:space:]/])codex([[:space:]]|$)'; then
        log_to_file "WARN: stale codex.pid for ${task_id} (pid=${pid}) does not look like Codex; removing pid file"
        log_to_file "WARN: cmd=${cmd}"
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    # If we can determine the expected worktree, prefer it as an additional safety check.
    # But if the cmdline doesn't match, don't silently proceed: waiting is safer than starting downstream tasks early.
    if [[ -n "${worktree_abs}" ]]; then
        if ! printf '%s\n' "${cmd}" | rg -F -q -- "--cd ${worktree_abs}"; then
            log_to_file "WARN: codex.pid for ${task_id} (pid=${pid}) does not match expected --cd ${worktree_abs}; waiting anyway"
            log_to_file "WARN: cmd=${cmd}"
        fi
    fi

    log_to_file "-- waiting for Codex to exit: task=${task_id} pid=${pid}"
    while kill -0 "${pid}" 2>/dev/null; do
        sleep 2
    done
    rm -f "${pid_path}" >/dev/null 2>&1 || true
}

ensure_last_message_or_stub() {
    local task_id="$1"
    local last_message_path="$2"
    local head_sha="${3:-}"
    local kind="$4"

    if [[ -f "${last_message_path}" ]]; then
        return 0
    fi

    mkdir -p "$(dirname "${last_message_path}")" >/dev/null 2>&1 || true
    {
        printf '# Generated task summary (Codex last message missing)\n\n'
        printf 'This file was generated by `scripts/triad/task_start_complete.sh` because the expected Codex `--output-last-message` file was missing.\n'
        printf 'This typically means the Codex process was interrupted or crashed.\n\n'
        printf -- '- Task: `%s`\n' "${task_id}"
        if [[ -n "${head_sha}" ]]; then
            printf -- '- HEAD: `%s`\n' "${head_sha}"
        fi
        printf -- '- Wrapper log: `%s`\n' "${LOG_PATH}"
        printf -- '- Codex stderr log: `%s/logs/%s/%s/stderr.log`\n' "${FEATURE_DIR_ABS}" "${SLICE_ID}" "${kind}"
    } >"${last_message_path}" 2>/dev/null || true
}

relaunch_codex_for_task_if_needed() {
    local task_id="$1"
    local last_message_path="$2"

    if [[ -f "${last_message_path}" ]]; then
        return 0
    fi

    wait_for_codex_if_running "${task_id}"
    if [[ -f "${last_message_path}" ]]; then
        return 0
    fi

    log_to_file "-- missing Codex last_message for ${task_id}; re-launching Codex"
    local start_out
    local start_err
    start_out="$(mktemp)"
    start_err="$(mktemp)"

    start_args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${task_id}" --launch-codex)
    if [[ -n "${CODEX_PROFILE}" ]]; then start_args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then start_args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then start_args+=(--codex-jsonl); fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then start_args+=(--dry-run); fi

    set +e
    "${start_args[@]}" >"${start_out}" 2>"${start_err}"
    start_rc="$?"
    set -e
    cat "${start_out}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    cat "${start_err}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    rm -f "${start_out}" "${start_err}"

    if [[ "${start_rc}" -ne 0 ]]; then
        die "Failed to relaunch Codex for ${task_id}; see ${LOG_PATH}"
    fi
}

if [[ "${CODE_TEST_STATUS}" == "pending" ]]; then
    log_to_file "-- code+test status=pending: starting both"
    log_to_file "-- planning-pack START (code+test)"
    python3 - "${TASKS_JSON}" "${SESSION_LOG}" "${now_utc}" "${CODE_TASK_ID}" "${TEST_TASK_ID}" <<'PY'
import json
import sys
from pathlib import Path

tasks_path = Path(sys.argv[1])
session_log = Path(sys.argv[2])
now_utc = sys.argv[3]
code_id = sys.argv[4]
test_id = sys.argv[5]

data = json.loads(tasks_path.read_text(encoding="utf-8"))
tasks = data.get("tasks") or []
by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}

for tid in (code_id, test_id):
    t = by_id.get(tid)
    if not t:
        print(f"ERROR: task not found in tasks.json: {tid}", file=sys.stderr)
        raise SystemExit(2)
    status = t.get("status")
    if status not in ("pending",):
        print(f"ERROR: task {tid} must be status='pending' to start (status={status!r})", file=sys.stderr)
        raise SystemExit(2)
    t["status"] = "in_progress"

tasks_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

slice_id = code_id[: -len("-code")] if code_id.endswith("-code") else code_id
cmd = f'make triad-task-start-complete FEATURE_DIR="{tasks_path.parent.as_posix()}" SLICE_ID="{slice_id}"'

lines = []
for kind, tid in (("code", code_id), ("test", test_id)):
    lines.extend([f"## START — {now_utc} — {kind} — {tid}", "- Dispatch:", f"  - `{cmd}`", ""])
session_log.write_text(session_log.read_text(encoding="utf-8") + "\n" + "\n".join(lines), encoding="utf-8")
PY

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log_to_file "DRY_RUN=1: skipping git commit for planning-pack START"
    else
        git add "${TASKS_JSON}" "${SESSION_LOG}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
        git commit -m "docs: start ${SLICE_ID} code+test" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
    fi

    pair_stdout="$(mktemp)"
    pair_stderr="$(mktemp)"
    log_to_file "-- triad-task-start-pair (codex enabled)"
    pair_args=(scripts/triad/task_start_pair.sh --feature-dir "${FEATURE_DIR_ABS}" --slice-id "${SLICE_ID}" --launch-codex)
    if [[ -n "${CODEX_PROFILE}" ]]; then pair_args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then pair_args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then pair_args+=(--codex-jsonl); fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then pair_args+=(--dry-run); fi

    set +e
    "${pair_args[@]}" >"${pair_stdout}" 2>"${pair_stderr}"
    pair_rc="$?"
    set -e
    cat "${pair_stdout}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    cat "${pair_stderr}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    rm -f "${pair_stderr}"
    if [[ "${pair_rc}" -ne 0 ]]; then
        die "triad-task-start-pair failed; see ${LOG_PATH}"
    fi

    CODE_WORKTREE="$(parse_kv CODE_WORKTREE "${pair_stdout}")"
    TEST_WORKTREE="$(parse_kv TEST_WORKTREE "${pair_stdout}")"
    CODEX_CODE_LAST_MESSAGE_PATH="$(parse_kv CODEX_CODE_LAST_MESSAGE_PATH "${pair_stdout}")"
    CODEX_TEST_LAST_MESSAGE_PATH="$(parse_kv CODEX_TEST_LAST_MESSAGE_PATH "${pair_stdout}")"

	    if [[ -z "${CODE_WORKTREE}" || -z "${TEST_WORKTREE}" ]]; then
	        die "Failed to parse CODE_WORKTREE/TEST_WORKTREE; see ${LOG_PATH}"
	    fi

	    relaunch_codex_for_task_if_needed "${CODE_TASK_ID}" "${CODEX_CODE_LAST_MESSAGE_PATH}"
	    relaunch_codex_for_task_if_needed "${TEST_TASK_ID}" "${CODEX_TEST_LAST_MESSAGE_PATH}"
	elif [[ "${CODE_TEST_STATUS}" == "in_progress" ]]; then
	    log_to_file "-- code+test status=in_progress: resuming"
	    CODE_WORKTREE="$(python_abs_path "$(jq -r --arg id "${CODE_TASK_ID}" '.tasks[] | select(.id==$id) | .worktree' "${TASKS_JSON}")")"
	    TEST_WORKTREE="$(python_abs_path "$(jq -r --arg id "${TEST_TASK_ID}" '.tasks[] | select(.id==$id) | .worktree' "${TASKS_JSON}")")"
	    relaunch_codex_for_task_if_needed "${CODE_TASK_ID}" "${CODEX_CODE_LAST_MESSAGE_PATH}"
	    relaunch_codex_for_task_if_needed "${TEST_TASK_ID}" "${CODEX_TEST_LAST_MESSAGE_PATH}"
	else
	    log_to_file "-- code+test status=completed: skipping start/finish"
	    CODE_HEAD_SHA="$(git rev-parse "${CODE_TASK_BRANCH}")"
	    TEST_HEAD_SHA="$(git rev-parse "${TEST_TASK_BRANCH}")"
	    ensure_last_message_or_stub "${CODE_TASK_ID}" "${CODEX_CODE_LAST_MESSAGE_PATH}" "${CODE_HEAD_SHA}" "code"
	    ensure_last_message_or_stub "${TEST_TASK_ID}" "${CODEX_TEST_LAST_MESSAGE_PATH}" "${TEST_HEAD_SHA}" "test"
	fi

if [[ "${CODE_TEST_STATUS}" != "completed" ]]; then
    if [[ -z "${CODE_WORKTREE}" || -z "${TEST_WORKTREE}" ]]; then
        die "Missing CODE_WORKTREE/TEST_WORKTREE for ${CODE_TASK_ID}/${TEST_TASK_ID}; see ${LOG_PATH}"
    fi

    wait_for_codex_if_running "${CODE_TASK_ID}"
    wait_for_codex_if_running "${TEST_TASK_ID}"

    code_finish_out="$(mktemp)"
    test_finish_out="$(mktemp)"
    finish_one "${CODE_TASK_ID}" "${CODE_WORKTREE}" "${code_finish_out}"
    finish_one "${TEST_TASK_ID}" "${TEST_WORKTREE}" "${test_finish_out}"

	    CODE_HEAD_SHA="$(parse_kv HEAD "${code_finish_out}")"
	    TEST_HEAD_SHA="$(parse_kv HEAD "${test_finish_out}")"
	    ensure_last_message_or_stub "${CODE_TASK_ID}" "${CODEX_CODE_LAST_MESSAGE_PATH}" "${CODE_HEAD_SHA}" "code"
	    ensure_last_message_or_stub "${TEST_TASK_ID}" "${CODEX_TEST_LAST_MESSAGE_PATH}" "${TEST_HEAD_SHA}" "test"

    now_utc_end="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    log_to_file "-- planning-pack END (code+test)"
    python3 - "${TASKS_JSON}" "${SESSION_LOG}" "${now_utc_end}" "${CODE_TASK_ID}" "${CODE_HEAD_SHA}" "${CODEX_CODE_LAST_MESSAGE_PATH}" "${TEST_TASK_ID}" "${TEST_HEAD_SHA}" "${CODEX_TEST_LAST_MESSAGE_PATH}" <<'PY'
import json
import sys
from pathlib import Path

tasks_path = Path(sys.argv[1])
session_log = Path(sys.argv[2])
now_utc = sys.argv[3]
code_id = sys.argv[4]
code_head = sys.argv[5]
code_msg = sys.argv[6]
test_id = sys.argv[7]
test_head = sys.argv[8]
test_msg = sys.argv[9]

data = json.loads(tasks_path.read_text(encoding="utf-8"))
tasks = data.get("tasks") or []
by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}

for tid in (code_id, test_id):
    t = by_id.get(tid)
    if not t:
        print(f"ERROR: task not found in tasks.json: {tid}", file=sys.stderr)
        raise SystemExit(2)
    t["status"] = "completed"

tasks_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = []
lines.extend([f"## END — {now_utc} — code — {code_id}", f"- HEAD: `{code_head}`", f"- Codex last message: `{code_msg}`", ""])
lines.extend([f"## END — {now_utc} — test — {test_id}", f"- HEAD: `{test_head}`", f"- Codex last message: `{test_msg}`", ""])
session_log.write_text(session_log.read_text(encoding="utf-8") + "\n" + "\n".join(lines), encoding="utf-8")
PY

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log_to_file "DRY_RUN=1: skipping git commit for planning-pack END (code+test)"
    else
        git add "${TASKS_JSON}" "${SESSION_LOG}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
        git commit -m "docs: finish ${SLICE_ID} code+test" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
    fi
fi

INTEG_STATUS="$(jq -r --arg id "${INTEG_TASK_ID}" '.tasks[] | select(.id==$id) | .status' "${TASKS_JSON}")"
INTEG_TASK_BRANCH="$(jq -r --arg id "${INTEG_TASK_ID}" '.tasks[] | select(.id==$id) | .git_branch' "${TASKS_JSON}")"
INTEG_KIND="${INTEG_TASK_ID#${SLICE_ID}-}"
CODEX_INTEG_LAST_MESSAGE_PATH="${FEATURE_DIR_ABS}/logs/${SLICE_ID}/${INTEG_KIND}/last_message.md"

INTEG_WORKTREE=""
integ_stdout=""
integ_finish_out=""

if [[ "${INTEG_STATUS}" == "pending" ]]; then
    log_to_file "-- integration status=pending: starting"
    now_utc_integ_start="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    log_to_file "-- planning-pack START (integration)"
    python3 - "${TASKS_JSON}" "${SESSION_LOG}" "${now_utc_integ_start}" "${INTEG_TASK_ID}" <<'PY'
import json
import sys
from pathlib import Path

tasks_path = Path(sys.argv[1])
session_log = Path(sys.argv[2])
now_utc = sys.argv[3]
task_id = sys.argv[4]

data = json.loads(tasks_path.read_text(encoding="utf-8"))
tasks = data.get("tasks") or []
by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}
t = by_id.get(task_id)
if not t:
    print(f"ERROR: task not found in tasks.json: {task_id}", file=sys.stderr)
    raise SystemExit(2)
status = t.get("status")
if status not in ("pending",):
    print(f"ERROR: integration task {task_id} must be status='pending' to start (status={status!r})", file=sys.stderr)
    raise SystemExit(2)
t["status"] = "in_progress"
tasks_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    f"## START — {now_utc} — integration — {task_id}",
    "- Dispatch:",
    f"  - `make triad-task-start FEATURE_DIR=\"{tasks_path.parent.as_posix()}\" TASK_ID=\"{task_id}\" LAUNCH_CODEX=1`",
    "",
]
session_log.write_text(session_log.read_text(encoding="utf-8") + "\n" + "\n".join(lines), encoding="utf-8")
PY

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log_to_file "DRY_RUN=1: skipping git commit for planning-pack START (integration)"
    else
        git add "${TASKS_JSON}" "${SESSION_LOG}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
        git commit -m "docs: start ${INTEG_TASK_ID}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
    fi

    integ_stdout="$(mktemp)"
    integ_stderr="$(mktemp)"
    log_to_file "-- triad-task-start (integration; codex enabled): ${INTEG_TASK_ID}"
    integ_args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${INTEG_TASK_ID}" --launch-codex)
    if [[ -n "${CODEX_PROFILE}" ]]; then integ_args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then integ_args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then integ_args+=(--codex-jsonl); fi
    if [[ "${DRY_RUN}" -eq 1 ]]; then integ_args+=(--dry-run); fi

    set +e
    "${integ_args[@]}" >"${integ_stdout}" 2>"${integ_stderr}"
    integ_rc="$?"
    set -e
    cat "${integ_stdout}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    cat "${integ_stderr}" >>"${LOG_PATH}"
    printf '\n' >>"${LOG_PATH}"
    rm -f "${integ_stderr}"
    if [[ "${integ_rc}" -ne 0 ]]; then
        die "triad-task-start failed for ${INTEG_TASK_ID}; see ${LOG_PATH}"
    fi

	    INTEG_WORKTREE="$(parse_kv WORKTREE "${integ_stdout}")"
	    CODEX_INTEG_LAST_MESSAGE_PATH="$(parse_kv CODEX_LAST_MESSAGE_PATH "${integ_stdout}")"
	    if [[ -z "${INTEG_WORKTREE}" ]]; then
	        die "Failed to parse integration WORKTREE; see ${LOG_PATH}"
	    fi
	    relaunch_codex_for_task_if_needed "${INTEG_TASK_ID}" "${CODEX_INTEG_LAST_MESSAGE_PATH}"
	elif [[ "${INTEG_STATUS}" == "in_progress" ]]; then
	    log_to_file "-- integration status=in_progress: resuming"
	    INTEG_WORKTREE="$(python_abs_path "$(jq -r --arg id "${INTEG_TASK_ID}" '.tasks[] | select(.id==$id) | .worktree' "${TASKS_JSON}")")"
	    relaunch_codex_for_task_if_needed "${INTEG_TASK_ID}" "${CODEX_INTEG_LAST_MESSAGE_PATH}"
	else
	    log_to_file "-- integration status=completed: skipping start/finish"
	    INTEG_HEAD_SHA="$(git rev-parse "${INTEG_TASK_BRANCH}")"
	    ensure_last_message_or_stub "${INTEG_TASK_ID}" "${CODEX_INTEG_LAST_MESSAGE_PATH}" "${INTEG_HEAD_SHA}" "${INTEG_KIND}"
	fi

if [[ "${INTEG_STATUS}" != "completed" ]]; then
    if [[ -z "${INTEG_WORKTREE}" ]]; then
        die "Missing INTEG_WORKTREE for ${INTEG_TASK_ID}; see ${LOG_PATH}"
    fi

    wait_for_codex_if_running "${INTEG_TASK_ID}"

	    integ_finish_out="$(mktemp)"
	    finish_one "${INTEG_TASK_ID}" "${INTEG_WORKTREE}" "${integ_finish_out}"
	    INTEG_HEAD_SHA="$(parse_kv HEAD "${integ_finish_out}")"
	    ensure_last_message_or_stub "${INTEG_TASK_ID}" "${CODEX_INTEG_LAST_MESSAGE_PATH}" "${INTEG_HEAD_SHA}" "${INTEG_KIND}"

    now_utc_integ_end="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    log_to_file "-- planning-pack END (integration)"
    python3 - "${TASKS_JSON}" "${SESSION_LOG}" "${now_utc_integ_end}" "${INTEG_TASK_ID}" "${INTEG_HEAD_SHA}" "${CODEX_INTEG_LAST_MESSAGE_PATH}" <<'PY'
import json
import sys
from pathlib import Path

tasks_path = Path(sys.argv[1])
session_log = Path(sys.argv[2])
now_utc = sys.argv[3]
task_id = sys.argv[4]
head = sys.argv[5]
msg = sys.argv[6]

data = json.loads(tasks_path.read_text(encoding="utf-8"))
tasks = data.get("tasks") or []
by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}
t = by_id.get(task_id)
if not t:
    print(f"ERROR: task not found in tasks.json: {task_id}", file=sys.stderr)
    raise SystemExit(2)
t["status"] = "completed"
tasks_path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    f"## END — {now_utc} — integration — {task_id}",
    f"- HEAD: `{head}`",
    f"- Codex last message: `{msg}`",
    "",
]
session_log.write_text(session_log.read_text(encoding="utf-8") + "\n" + "\n".join(lines), encoding="utf-8")
PY

    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log_to_file "DRY_RUN=1: skipping git commit for planning-pack END (integration)"
    else
        git add "${TASKS_JSON}" "${SESSION_LOG}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
        git commit -m "docs: finish ${INTEG_TASK_ID}" 1>>"${LOG_PATH}" 2>>"${LOG_PATH}"
    fi
fi

log_to_file "-- summary"
python3 - "${SUMMARY_JSON_PATH}" "${SLICE_ID}" "${CODE_TASK_ID}" "${TEST_TASK_ID}" "${INTEG_TASK_ID}" "${BOUNDARY_SLICE}" "${NEXT_CHECKPOINT_TASK_ID}" "${LOG_PATH}" "${CODEX_CODE_LAST_MESSAGE_PATH}" "${CODEX_TEST_LAST_MESSAGE_PATH}" "${CODEX_INTEG_LAST_MESSAGE_PATH}" "${CODE_HEAD_SHA}" "${TEST_HEAD_SHA}" "${INTEG_HEAD_SHA}" <<'PY'
import json
import sys

(
    summary_path,
    slice_id,
    code_task_id,
    test_task_id,
    integ_task_id,
    boundary_slice_raw,
    next_checkpoint_task_id,
    log_path,
    code_last_message_path,
    test_last_message_path,
    integ_last_message_path,
    code_head,
    test_head,
    integ_head,
) = sys.argv[1:]

data = {
    "slice_id": slice_id,
    "code_task_id": code_task_id,
    "test_task_id": test_task_id,
    "integ_task_id": integ_task_id,
    "boundary_slice": boundary_slice_raw == "1",
    "next_checkpoint_task_id": next_checkpoint_task_id,
    "artifacts": {
        "log_path": log_path,
        "code_last_message_path": code_last_message_path,
        "test_last_message_path": test_last_message_path,
        "integ_last_message_path": integ_last_message_path,
    },
    "heads": {
        "code": code_head,
        "test": test_head,
        "integration": integ_head,
    },
}

with open(summary_path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, sort_keys=True)
    f.write("\n")
PY

cleanup_files=()
for f in "${pair_stdout:-}" "${code_finish_out:-}" "${test_finish_out:-}" "${integ_stdout:-}" "${integ_finish_out:-}"; do
    if [[ -n "${f}" ]]; then
        cleanup_files+=("${f}")
    fi
done
if [[ "${#cleanup_files[@]}" -gt 0 ]]; then
    rm -f "${cleanup_files[@]}"
fi

printf 'SLICE_ID=%s\n' "${SLICE_ID}"
printf 'CODE_TASK_ID=%s\n' "${CODE_TASK_ID}"
printf 'TEST_TASK_ID=%s\n' "${TEST_TASK_ID}"
printf 'INTEG_TASK_ID=%s\n' "${INTEG_TASK_ID}"
printf 'BOUNDARY_SLICE=%s\n' "${BOUNDARY_SLICE}"
printf 'NEXT_CHECKPOINT_TASK_ID=%s\n' "${NEXT_CHECKPOINT_TASK_ID}"
printf 'LOG_PATH=%s\n' "${LOG_PATH}"
printf 'SUMMARY_JSON_PATH=%s\n' "${SUMMARY_JSON_PATH}"
