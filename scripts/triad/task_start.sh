#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_start.sh --feature-dir <path> --task-id <id> [options]

Required:
  --feature-dir <path>   Feature Planning Pack dir (e.g., docs/project_management/next/<feature>)
  --task-id <id>         Task id in tasks.json (e.g., C0-code)

Options:
  --launch-codex         Launch Codex headless after worktree creation
  --codex-profile <p>    Codex profile (passed to `codex exec --profile`)
  --codex-model <m>      Codex model (passed to `codex exec --model`)
  --codex-jsonl          Also capture Codex JSONL events to a file (uses `codex exec --json`)
  --platform <p>         linux|macos|windows|wsl (recorded in .taskmeta.json; optional)
  --dry-run              Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  WORKTREE=<path>
  TASK_BRANCH=<branch>
  ORCH_BRANCH=<branch>
  KICKOFF_PROMPT=<path>
  CODEX_OUT_DIR=<path>
  CODEX_LAST_MESSAGE_PATH=<path>
  CODEX_EVENTS_PATH=<path>
  CODEX_STDERR_PATH=<path>
  CODEX_EXIT=<code or empty>
  NEXT=<recommended next command>

Notes:
  - This script requires an automation-enabled planning pack (tasks.json meta.schema_version >= 3 and meta.automation.enabled=true).
  - Codex invocation always uses: --dangerously-bypass-approvals-and-sandbox (do not use --sandbox/--add-dir here).
  - Feature registry location is deterministic and shared across worktrees:
    <git-common-dir>/triad/features/<feature>/worktrees.json
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

json_get() {
    local json_path="$1"
    local jq_expr="$2"
    jq -r "$jq_expr" "$json_path"
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

utc_now() {
    date -u +%Y-%m-%dT%H:%M:%SZ
}

shell_escape() {
    printf '%q' "$1"
}

FEATURE_DIR=""
TASK_ID=""
LAUNCH_CODEX=0
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
PLATFORM=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --task-id)
            TASK_ID="${2:-}"
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
        --platform)
            PLATFORM="${2:-}"
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

if [[ -z "${FEATURE_DIR}" || -z "${TASK_ID}" ]]; then
    usage >&2
    die "Missing required args"
fi

case "${PLATFORM}" in
    ""|linux|macos|windows|wsl) ;;
    *)
        die "Invalid --platform: ${PLATFORM}"
        ;;
esac

require_cmd git
require_cmd jq
require_cmd python3

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"

if [[ ! -d "${FEATURE_DIR_ABS}" ]]; then
    die "Feature dir does not exist: ${FEATURE_DIR_ABS}"
fi
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi

SCHEMA_VERSION="$(json_get "${TASKS_JSON}" '.meta.schema_version // 1')"
AUTOMATION_ENABLED="$(json_get "${TASKS_JSON}" '.meta.automation.enabled // false')"
if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "task_start requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true (opt-in automation)"
fi

ORCH_BRANCH="$(json_get "${TASKS_JSON}" '.meta.automation.orchestration_branch // empty')"
FEATURE_NAME="$(json_get "${TASKS_JSON}" '.meta.feature // empty')"
if [[ -z "${ORCH_BRANCH}" || -z "${FEATURE_NAME}" ]]; then
    die "tasks.json meta.automation must include orchestration_branch, and meta.feature must be set"
fi

report_recommends_accept() {
    local report_path="$1"
    local report_label="$2"
    python3 - "${report_path}" "${report_label}" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
label = sys.argv[2]

try:
    text = path.read_text(encoding="utf-8")
except FileNotFoundError:
    print(f"ERROR: missing {label}: {path}", file=sys.stderr)
    raise SystemExit(2)

def clean(value: str) -> str:
    value = value.strip()
    value = value.replace("`", "")
    value = value.replace("*", "")
    return value.strip()

def is_accept(raw_value: str) -> bool:
    # Refuse template placeholders like "ACCEPT | REVISE".
    if "|" in raw_value:
        return False
    cleaned = clean(raw_value)
    upper = cleaned.upper()
    if "REVISE" in upper or "FLAG" in upper:
        return False
    return upper.startswith("ACCEPT")

recommendations = []
for line in text.splitlines():
    m = re.match(r"^\s*RECOMMENDATION:\s*(.+?)\s*$", line, flags=re.IGNORECASE)
    if m:
        recommendations.append(m.group(1))
        continue
    m = re.match(r"^\s*-?\s*Recommendation:\s*(.+?)\s*$", line, flags=re.IGNORECASE)
    if m:
        recommendations.append(m.group(1))
        continue

if not recommendations:
    print(f"ERROR: {label} does not contain a recommendation line: {path}", file=sys.stderr)
    raise SystemExit(2)

last = recommendations[-1]
if is_accept(last):
    raise SystemExit(0)

print(f"ERROR: {label} does not contain RECOMMENDATION: ACCEPT: {path}", file=sys.stderr)
raise SystemExit(2)
PY
}

require_feature_start_gates() {
    local tasks_json="$1"
    local feature_dir="$2"

    local quality_gate_report="${feature_dir}/quality_gate_report.md"
    report_recommends_accept "${quality_gate_report}" "quality gate report (quality_gate_report.md)"

    local execution_gates
    execution_gates="$(jq -r '.meta.execution_gates // false' "${tasks_json}")"
    if [[ "${execution_gates}" == "true" ]]; then
        local preflight_report="${feature_dir}/execution_preflight_report.md"
        report_recommends_accept "${preflight_report}" "execution preflight report (execution_preflight_report.md)"
    fi
}

require_task_deps_completed() {
    local tasks_json="$1"
    local task_id="$2"
    python3 - "${tasks_json}" "${task_id}" <<'PY'
import json
import sys

tasks_path = sys.argv[1]
task_id = sys.argv[2]

with open(tasks_path, "r", encoding="utf-8") as f:
    data = json.load(f)

tasks = {
    t.get("id"): t
    for t in data.get("tasks", [])
    if isinstance(t, dict) and isinstance(t.get("id"), str)
}
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
    if dep_task is None:
        # External deps are allowed via meta.external_task_ids; task_start can't validate them.
        continue
    status = dep_task.get("status")
    if status != "completed":
        print(
            f"ERROR: cannot start {task_id}: depends_on {dep} is not completed (status={status!r})",
            file=sys.stderr,
        )
        raise SystemExit(2)
PY
}

TASK_JSON="$(jq -c --arg id "${TASK_ID}" '.tasks[] | select(.id==$id)' "${TASKS_JSON}")" || true
if [[ -z "${TASK_JSON}" ]]; then
    die "Task not found in tasks.json: ${TASK_ID}"
fi

TASK_TYPE="$(jq -r '.type' <<<"${TASK_JSON}")"
WORKTREE_RELPATH="$(jq -r '.worktree' <<<"${TASK_JSON}")"
KICKOFF_RELPATH="$(jq -r '.kickoff_prompt' <<<"${TASK_JSON}")"
TASK_BRANCH="$(jq -r '.git_branch // empty' <<<"${TASK_JSON}")"

case "${TASK_TYPE}" in
    code|test|integration) ;;
    *)
        die "task_start only supports code/test/integration tasks; got type=${TASK_TYPE}"
        ;;
esac

# Feature-level start gates are execution-time guardrails: refuse to start worktrees unless the
# Planning Pack is approved and (when enabled) the execution preflight gate recommends ACCEPT.
require_feature_start_gates "${TASKS_JSON}" "${FEATURE_DIR_ABS}"

# Task-level gating: refuse to start unless depends_on tasks are completed.
require_task_deps_completed "${TASKS_JSON}" "${TASK_ID}"

if [[ -z "${WORKTREE_RELPATH}" || "${WORKTREE_RELPATH}" == "null" ]]; then
    die "tasks.json task.worktree must be set for ${TASK_ID}"
fi
if [[ -z "${KICKOFF_RELPATH}" || "${KICKOFF_RELPATH}" == "null" ]]; then
    die "tasks.json task.kickoff_prompt must be set for ${TASK_ID}"
fi
if [[ -z "${TASK_BRANCH}" ]]; then
    die "tasks.json task.git_branch is required for automation packs (task ${TASK_ID})"
fi

WORKTREE_ABS="$(python_abs_path "${WORKTREE_RELPATH}")"
KICKOFF_ABS="$(python_abs_path "${KICKOFF_RELPATH}")"

if [[ ! -f "${KICKOFF_ABS}" ]]; then
    die "Kickoff prompt does not exist: ${KICKOFF_ABS}"
fi

GIT_COMMON_DIR="$(python_abs_path "$(git rev-parse --git-common-dir)")"
REGISTRY_ABS="${GIT_COMMON_DIR}/triad/features/${FEATURE_NAME}/worktrees.json"
REGISTRY_DIR="$(dirname "${REGISTRY_ABS}")"

ensure_git_clean_or_die() {
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        return 0
    fi
    if ! git diff --quiet; then
        die "Working tree has unstaged changes (expected clean orchestration worktree)"
    fi
    if ! git diff --cached --quiet; then
        die "Working tree has staged changes (expected clean orchestration worktree)"
    fi
}

select_remote_for_branch_setup() {
    if git remote get-url origin >/dev/null 2>&1; then
        echo "origin"
        return 0
    fi
    # Deterministic fallback: first remote in sorted order.
    first="$(git remote | sort | head -n 1)"
    if [[ -n "${first}" ]]; then
        echo "${first}"
        return 0
    fi
    echo ""
}

remote_branch_exists() {
    local remote="$1"
    local branch="$2"
    git ls-remote --exit-code --heads "${remote}" "${branch}" >/dev/null 2>&1
}

checkout_orch_branch() {
    ensure_git_clean_or_die
    current="$(git rev-parse --abbrev-ref HEAD)"
    if [[ "${current}" != "${ORCH_BRANCH}" ]]; then
        log "Checking out orchestration branch: ${ORCH_BRANCH} (was ${current})"
        if [[ "${DRY_RUN}" -eq 1 ]]; then
            return 0
        fi

        if git show-ref --verify --quiet "refs/heads/${ORCH_BRANCH}"; then
            git checkout "${ORCH_BRANCH}" >/dev/null
        else
            remote="$(select_remote_for_branch_setup)"
            if [[ -z "${remote}" ]]; then
                die "Orchestration branch ${ORCH_BRANCH} does not exist locally, and no git remote is configured (create the branch and push it first)"
            fi
            if ! remote_branch_exists "${remote}" "${ORCH_BRANCH}"; then
                die "Orchestration branch ${ORCH_BRANCH} does not exist locally or on ${remote}; create it and push: git checkout -b ${ORCH_BRANCH} && git push -u ${remote} ${ORCH_BRANCH}"
            fi
            git fetch "${remote}" "${ORCH_BRANCH}:${ORCH_BRANCH}" >/dev/null
            git branch --set-upstream-to "${remote}/${ORCH_BRANCH}" "${ORCH_BRANCH}" >/dev/null 2>&1 || true
            git checkout "${ORCH_BRANCH}" >/dev/null
        fi
    fi

    # Best-effort ff-only pull if upstream exists.
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        return 0
    fi
    if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
        log "Pulling orchestration branch (ff-only)"
        git pull --ff-only >/dev/null
    else
        remote="$(select_remote_for_branch_setup)"
        if [[ -n "${remote}" ]] && remote_branch_exists "${remote}" "${ORCH_BRANCH}"; then
            git branch --set-upstream-to "${remote}/${ORCH_BRANCH}" "${ORCH_BRANCH}" >/dev/null 2>&1 || true
            if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
                log "Pulling orchestration branch (ff-only)"
                git pull --ff-only >/dev/null
            else
                log "No upstream configured for ${ORCH_BRANCH}; skipping git pull"
            fi
        else
            log "No upstream configured for ${ORCH_BRANCH}; skipping git pull"
        fi
    fi
}

ensure_task_branch_available() {
    if git show-ref --verify --quiet "refs/heads/${TASK_BRANCH}"; then
        return 0
    fi
    return 1
}

branch_in_use_by_worktree() {
    local branch="$1"
    git worktree list --porcelain | awk '
        $1=="branch"{print $2}
    ' | grep -Fxq "refs/heads/${branch}"
}

write_taskmeta() {
    local created_from_sha="$1"
    local created_at_utc="$2"
    local out_path="${WORKTREE_ABS}/.taskmeta.json"
    python3 - "${out_path}" "${TASK_ID}" "${FEATURE_DIR_ABS}" "${REPO_ROOT}" "${ORCH_BRANCH}" "${TASK_BRANCH}" "${created_from_sha}" "${created_at_utc}" "${PLATFORM}" <<'PY'
import json
import os
import sys

out_path = sys.argv[1]
task_id = sys.argv[2]
feature_dir_abs = sys.argv[3]
repo_root = sys.argv[4]
orch_branch = sys.argv[5]
task_branch = sys.argv[6]
created_from_sha = sys.argv[7]
created_at_utc = sys.argv[8]
platform = sys.argv[9]

data = {
  "schema_version": 1,
  "task_id": task_id,
  "feature_dir": os.path.relpath(feature_dir_abs, repo_root),
  "orchestration_branch": orch_branch,
  "task_branch": task_branch,
  "created_from_sha": created_from_sha,
  "created_at_utc": created_at_utc,
}
if platform:
  data["platform"] = platform
os.makedirs(os.path.dirname(out_path), exist_ok=True)
tmp = out_path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
  json.dump(data, f, indent=2, sort_keys=True)
  f.write("\n")
os.replace(tmp, out_path)
PY
}

update_registry() {
    local created_from_sha="$1"
    local created_at_utc="$2"
    python3 - "${REPO_ROOT}" "${REGISTRY_ABS}" "${FEATURE_DIR_ABS}" "${WORKTREE_ABS}" "${TASK_ID}" "${TASK_TYPE}" "${TASK_BRANCH}" "${created_from_sha}" "${created_at_utc}" "${ORCH_BRANCH}" "${PLATFORM}" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone

repo_root = sys.argv[1]
registry_abs = sys.argv[2]
feature_dir_abs = sys.argv[3]
worktree_abs = sys.argv[4]
task_id = sys.argv[5]
task_type = sys.argv[6]
task_branch = sys.argv[7]
created_from_sha = sys.argv[8]
created_at_utc = sys.argv[9]
orch_branch = sys.argv[10]
platform = sys.argv[11]

feature_dir_rel = os.path.relpath(feature_dir_abs, repo_root)
worktree_rel = os.path.relpath(worktree_abs, repo_root)

entry = {
  "task_id": task_id,
  "task_type": task_type,
  "task_branch": task_branch,
  "worktree": worktree_abs,
  "created_from_sha": created_from_sha,
  "created_at_utc": created_at_utc,
}
if platform:
  entry["platform"] = platform

os.makedirs(os.path.dirname(registry_abs), exist_ok=True)
if os.path.exists(registry_abs):
  with open(registry_abs, "r", encoding="utf-8") as f:
    data = json.load(f)
else:
  data = {
    "schema_version": 1,
    "feature_dir": feature_dir_rel,
    "orchestration_branch": orch_branch,
    "updated_at_utc": None,
    "entries": [],
  }

entries = data.get("entries", [])
replaced = False
for i, e in enumerate(entries):
  if e.get("task_id") == entry["task_id"]:
    entries[i] = {**e, **entry}
    replaced = True
    break
if not replaced:
  entries.append(entry)

data["entries"] = entries
data["updated_at_utc"] = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

tmp = registry_abs + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
  json.dump(data, f, indent=2, sort_keys=True)
  f.write("\n")
os.replace(tmp, registry_abs)
PY
}

create_worktree_if_needed() {
    if [[ -d "${WORKTREE_ABS}" ]]; then
        if ! git -C "${WORKTREE_ABS}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
            die "Worktree path exists but is not a git worktree: ${WORKTREE_ABS}"
        fi
        existing_branch="$(git -C "${WORKTREE_ABS}" rev-parse --abbrev-ref HEAD)"
        if [[ "${existing_branch}" != "${TASK_BRANCH}" ]]; then
            die "Worktree path already exists but is on branch ${existing_branch}; expected ${TASK_BRANCH}: ${WORKTREE_ABS}"
        fi
        return 0
    fi

    if branch_in_use_by_worktree "${TASK_BRANCH}"; then
        die "Task branch is already checked out by an existing worktree: ${TASK_BRANCH} (run git worktree list)"
    fi

    log "Creating worktree: ${WORKTREE_ABS} (branch ${TASK_BRANCH} from ${ORCH_BRANCH})"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        return 0
    fi

    mkdir -p "$(dirname "${WORKTREE_ABS}")"
    if ensure_task_branch_available; then
        git worktree add "${WORKTREE_ABS}" "${TASK_BRANCH}" >/dev/null
    else
        git worktree add -b "${TASK_BRANCH}" "${WORKTREE_ABS}" "${ORCH_BRANCH}" >/dev/null
    fi
}

checkout_orch_branch
created_from_sha="$(git rev-parse HEAD)"
created_at_utc="$(utc_now)"

create_worktree_if_needed

if [[ "${DRY_RUN}" -eq 0 ]]; then
    mkdir -p "${REGISTRY_DIR}"
    write_taskmeta "${created_from_sha}" "${created_at_utc}"
    update_registry "${created_from_sha}" "${created_at_utc}"
fi

codex_cmd="codex exec --dangerously-bypass-approvals-and-sandbox --cd \"${WORKTREE_ABS}\""
if [[ -n "${CODEX_PROFILE}" ]]; then
    codex_cmd="${codex_cmd} --profile \"${CODEX_PROFILE}\""
fi
if [[ -n "${CODEX_MODEL}" ]]; then
    codex_cmd="${codex_cmd} --model \"${CODEX_MODEL}\""
fi

codex_out_dir="${REPO_ROOT}/target/triad/${FEATURE_NAME}/codex/${TASK_ID}"
codex_last_message="${codex_out_dir}/last_message.md"
codex_events="${codex_out_dir}/events.jsonl"
codex_stderr="${codex_out_dir}/stderr.log"
codex_exit=""

if [[ "${CODEX_JSONL}" -eq 1 ]]; then
    codex_cmd="${codex_cmd} --json"
fi
codex_cmd="${codex_cmd} --output-last-message \"${codex_last_message}\" - < \"${KICKOFF_ABS}\""

next_cmd="${codex_cmd}"
if [[ "${LAUNCH_CODEX}" -eq 1 ]]; then
    next_cmd="(already launched)"
fi

printf 'WORKTREE=%s\n' "${WORKTREE_ABS}"
printf 'TASK_BRANCH=%s\n' "${TASK_BRANCH}"
printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'KICKOFF_PROMPT=%s\n' "${KICKOFF_ABS}"
printf 'CODEX_OUT_DIR=%s\n' "${codex_out_dir}"
printf 'CODEX_LAST_MESSAGE_PATH=%s\n' "${codex_last_message}"
printf 'CODEX_EVENTS_PATH=%s\n' "${codex_events}"
printf 'CODEX_STDERR_PATH=%s\n' "${codex_stderr}"
if [[ "${LAUNCH_CODEX}" -ne 1 ]]; then
    # Make NEXT copy/pasteable by ensuring the output dir exists.
    next_cmd="mkdir -p $(shell_escape "${codex_out_dir}") && ${next_cmd}"
fi
printf 'NEXT=%s\n' "${next_cmd}"

if [[ "${LAUNCH_CODEX}" -eq 1 ]]; then
    require_cmd codex
    log "Launching Codex headless (output captured under target/triad/${FEATURE_NAME}/codex/${TASK_ID}/)"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf 'CODEX_EXIT=%s\n' "dry-run"
        exit 0
    fi
    mkdir -p "${codex_out_dir}"
    codex_pid_path="${codex_out_dir}/codex.pid"
    codex_args=(codex exec --dangerously-bypass-approvals-and-sandbox --cd "${WORKTREE_ABS}")
    if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
    codex_args+=(--output-last-message "${codex_last_message}" -)
    set +e
    "${codex_args[@]}" < "${KICKOFF_ABS}" >"${codex_events}" 2>"${codex_stderr}" &
    codex_pid="$!"
    printf '%s\n' "${codex_pid}" > "${codex_pid_path}"
    wait "${codex_pid}"
    codex_exit="$?"
    rm -f "${codex_pid_path}" >/dev/null 2>&1 || true
    set -e
    printf 'CODEX_EXIT=%s\n' "${codex_exit}"
    exit "${codex_exit}"
fi

printf 'CODEX_EXIT=%s\n' "${codex_exit}"
