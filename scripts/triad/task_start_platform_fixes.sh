#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_start_platform_fixes.sh --feature-dir <path> --slice-id <slice> (--platform <p> [--platform <p> ...] | --from-smoke-run <id>) [options]

Required:
  --feature-dir <path>     Feature Planning Pack dir (docs/project_management/next/<feature> or equivalent)
  --slice-id <slice>       Slice prefix (e.g., C0)
  --platform <p>           linux|macos|windows|wsl (repeatable; failing platforms only)
  --from-smoke-run <id>    GitHub Actions run id for Feature Smoke (platform=all); selects only failing platforms automatically

Options:
  --launch-codex           Launch Codex headless for ALL selected platform-fix tasks (runs in parallel after worktrees are created)
  --codex-profile <p>      Codex profile (passed to `codex exec --profile`)
  --codex-model <m>        Codex model (passed to `codex exec --model`)
  --codex-jsonl            Capture Codex JSONL event stream (uses `codex exec --json`)
  --dry-run                Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  ORCH_BRANCH=<branch>
  SLICE_ID=<slice>
  SMOKE_RUN_ID=<id or empty>
  FAILED_PLATFORMS=<csv or empty>
  PLATFORM=<platform>
  TASK_ID=<task-id>
  WORKTREE=<path>
  TASK_BRANCH=<branch>
  CODEX_EXIT=<code or empty>
  CODEX_OUT_DIR=<path>
  CODEX_LAST_MESSAGE_PATH=<path>
  CODEX_EVENTS_PATH=<path>
  CODEX_STDERR_PATH=<path>

Notes:
  - Requires an automation-enabled planning pack (tasks.json meta.schema_version>=3 and meta.automation.enabled=true).
  - Runs from the orchestration worktree (or repo root) and uses:
    - `scripts/triad/orch_ensure.sh`
    - `scripts/triad/task_start.sh`
  - This wrapper does not edit tasks.json; it only sets up branches/worktrees and optionally launches Codex.
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
LAUNCH_CODEX=0
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
DRY_RUN=0
PLATFORMS=()

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
        --platform)
            PLATFORMS+=("${2:-}")
            shift 2
            ;;
        --from-smoke-run)
            SMOKE_RUN_ID="${2:-}"
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
if [[ -n "${SMOKE_RUN_ID}" && "${#PLATFORMS[@]}" -ne 0 ]]; then
    die "Use either --platform ... or --from-smoke-run <id>, not both"
fi
if [[ -z "${SMOKE_RUN_ID}" && "${#PLATFORMS[@]}" -eq 0 ]]; then
    usage >&2
    die "Missing required platform selection (--platform or --from-smoke-run)"
fi

for p in "${PLATFORMS[@]}"; do
    case "${p}" in
        linux|macos|windows|wsl) ;;
        *) die "Invalid --platform: ${p}" ;;
    esac
done

require_cmd git
require_cmd jq
require_cmd python3
if [[ -n "${SMOKE_RUN_ID}" ]]; then
    require_cmd gh
fi

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
WSL_TASK_MODE="$(jq -r '.meta.wsl_task_mode // "bundled"' "${TASKS_JSON}")"

if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "task_start_platform_fixes requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true"
fi
if [[ -z "${ORCH_BRANCH}" || -z "${FEATURE_NAME}" ]]; then
    die "tasks.json must include meta.feature and meta.automation.orchestration_branch"
fi
case "${WSL_TASK_MODE}" in
    bundled|separate) ;;
    *) die "Invalid tasks.json meta.wsl_task_mode: ${WSL_TASK_MODE} (expected bundled|separate)" ;;
esac

failed_platforms_csv=""
if [[ -n "${SMOKE_RUN_ID}" ]]; then
    log "Selecting failing platforms from Feature Smoke run: ${SMOKE_RUN_ID}"
    run_json="$(gh run view "${SMOKE_RUN_ID}" --json status,conclusion,workflowName,url,jobs 2>/dev/null || true)"
    if [[ -z "${run_json}" ]]; then
        die "Could not read run ${SMOKE_RUN_ID} via gh (check permissions/auth)"
    fi
    set +e
    failed_platforms_csv="$(python3 - "${run_json}" 2>&1 <<'PY'
import json
import sys

data = json.loads(sys.argv[1])
status = data.get("status")
workflow = data.get("workflowName") or ""

if status != "completed":
    raise SystemExit("ERROR: smoke run is not completed yet; wait for completion and retry")

if "Feature Smoke" not in workflow:
    raise SystemExit(f"ERROR: run is not Feature Smoke workflow (workflowName={workflow!r})")

jobs = data.get("jobs") or []
failed = set()

def job_platform(name: str):
    if name.startswith("linux_"):
        return "linux"
    if name.startswith("macos_"):
        return "macos"
    if name.startswith("windows_"):
        return "windows"
    if name == "wsl":
        return "wsl"
    return None

for j in jobs:
    if not isinstance(j, dict):
        continue
    name = str(j.get("name") or "")
    concl = j.get("conclusion")
    if concl in (None, "skipped"):
        continue
    p = job_platform(name)
    if not p:
        continue
    if concl != "success":
        failed.add(p)

print(",".join(sorted(failed)))
PY
)"
    rc=$?
    set -e
    if [[ "${rc}" -ne 0 ]]; then
        die "${failed_platforms_csv}"
    fi
    if [[ -z "${failed_platforms_csv}" ]]; then
        log "Smoke run has no failing platform jobs; nothing to start"
        printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
        printf 'SLICE_ID=%s\n' "${SLICE_ID}"
        printf 'SMOKE_RUN_ID=%s\n' "${SMOKE_RUN_ID}"
        printf 'FAILED_PLATFORMS=%s\n' ""
        exit 0
    fi
    IFS=',' read -r -a PLATFORMS <<<"${failed_platforms_csv}"
    normalized=()
    for p in "${PLATFORMS[@]}"; do
        case "${p}" in
            linux|macos|windows|wsl) ;;
            *) die "Invalid platform inferred from smoke run: ${p}" ;;
        esac
        if [[ "${p}" == "wsl" && "${WSL_TASK_MODE}" == "bundled" ]]; then
            normalized+=("linux")
        else
            normalized+=("${p}")
        fi
    done
    declare -A seen=()
    PLATFORMS=()
    for p in "${normalized[@]}"; do
        [[ -z "${p}" ]] && continue
        if [[ -z "${seen[${p}]:-}" ]]; then
            seen["${p}"]=1
            PLATFORMS+=("${p}")
        fi
    done
    failed_platforms_csv="$(IFS=','; echo "${PLATFORMS[*]}")"
fi

task_exists() {
    local id="$1"
    jq -e --arg id "${id}" '.tasks[] | select(.id==$id)' "${TASKS_JSON}" >/dev/null
}

task_type_for() {
    local id="$1"
    jq -r --arg id "${id}" '.tasks[] | select(.id==$id) | .type' "${TASKS_JSON}"
}

task_platform_for() {
    local id="$1"
    jq -r --arg id "${id}" '.tasks[] | select(.id==$id) | .platform // empty' "${TASKS_JSON}"
}

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

declare -A task_worktree=()
declare -A task_branch=()
declare -A task_kickoff=()
declare -A task_codex_out_dir=()
declare -A task_codex_last_message=()
declare -A task_codex_events=()
declare -A task_codex_stderr=()

selected_task_ids=()
for p in "${PLATFORMS[@]}"; do
    task_id="${SLICE_ID}-integ-${p}"
    if ! task_exists "${task_id}"; then
        die "Task not found in tasks.json: ${task_id} (expected for slice ${SLICE_ID} platform ${p})"
    fi
    if [[ "$(task_type_for "${task_id}")" != "integration" ]]; then
        die "Task ${task_id} must have type=integration"
    fi
    declared_platform="$(task_platform_for "${task_id}")"
    if [[ -n "${declared_platform}" && "${declared_platform}" != "${p}" ]]; then
        die "Task ${task_id} has platform=${declared_platform}, expected ${p}"
    fi
    selected_task_ids+=("${task_id}")
done

for task_id in "${selected_task_ids[@]}"; do
    log "Creating platform-fix worktree: ${task_id}"
    args=(scripts/triad/task_start.sh --feature-dir "${FEATURE_DIR_ABS}" --task-id "${task_id}")
    if [[ "${DRY_RUN}" -eq 1 ]]; then args+=(--dry-run); fi
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi

    out="$("${args[@]}")"
    worktree="$(parse_kv WORKTREE "${out}")"
    branch="$(parse_kv TASK_BRANCH "${out}")"
    kickoff="$(parse_kv KICKOFF_PROMPT "${out}")"
    if [[ -z "${worktree}" || -z "${branch}" || -z "${kickoff}" ]]; then
        die "Failed to parse task_start output for ${task_id}"
    fi
    task_worktree["${task_id}"]="${worktree}"
    task_branch["${task_id}"]="${branch}"
    task_kickoff["${task_id}"]="${kickoff}"

    out_dir="${REPO_ROOT}/target/triad/${FEATURE_NAME}/codex/${task_id}"
    task_codex_out_dir["${task_id}"]="${out_dir}"
    task_codex_last_message["${task_id}"]="${out_dir}/last_message.md"
    task_codex_events["${task_id}"]="${out_dir}/events.jsonl"
    task_codex_stderr["${task_id}"]="${out_dir}/stderr.log"
done

declare -A codex_exit=()
for task_id in "${selected_task_ids[@]}"; do
    codex_exit["${task_id}"]=""
done

launch_codex_one() {
    local task_id="$1"
    local worktree="$2"
    local kickoff="$3"
    local out_dir="${task_codex_out_dir[${task_id}]}"
    local last_message="${task_codex_last_message[${task_id}]}"
    local events="${task_codex_events[${task_id}]}"
    local stderr="${task_codex_stderr[${task_id}]}"
    local pid_path="${out_dir}/codex.pid"

    mkdir -p "${out_dir}"

    codex_args=(codex exec --dangerously-bypass-approvals-and-sandbox --cd "${worktree}")
    if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
    codex_args+=(--output-last-message "${last_message}" -)
    "${codex_args[@]}" < "${kickoff}" >"${events}" 2>"${stderr}" &
    codex_pid="$!"
    printf '%s\n' "${codex_pid}" > "${pid_path}"
    wait "${codex_pid}"
    rc="$?"
    rm -f "${pid_path}" >/dev/null 2>&1 || true
    return "${rc}"
}

if [[ "${LAUNCH_CODEX}" -eq 1 ]]; then
    require_cmd codex
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        log "DRY_RUN=1: skipping codex exec"
        for task_id in "${selected_task_ids[@]}"; do
            codex_exit["${task_id}"]="dry-run"
        done
    else
        log "Launching Codex headless for platform-fix tasks (in parallel)"
        declare -A pids=()
        set +e
        for task_id in "${selected_task_ids[@]}"; do
            launch_codex_one "${task_id}" "${task_worktree[${task_id}]}" "${task_kickoff[${task_id}]}" &
            pids["${task_id}"]=$!
        done
        for task_id in "${selected_task_ids[@]}"; do
            wait "${pids[${task_id}]}"
            codex_exit["${task_id}"]="$?"
        done
        set -e
        for task_id in "${selected_task_ids[@]}"; do
            if [[ "${codex_exit[${task_id}]}" -ne 0 ]]; then
                echo "WARN: codex exec failed for ${task_id} (exit=${codex_exit[${task_id}]})" >&2
            fi
        done
    fi
fi

printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'SLICE_ID=%s\n' "${SLICE_ID}"
printf 'SMOKE_RUN_ID=%s\n' "${SMOKE_RUN_ID}"
printf 'FAILED_PLATFORMS=%s\n' "${failed_platforms_csv}"
for p in "${PLATFORMS[@]}"; do
    task_id="${SLICE_ID}-integ-${p}"
    printf 'PLATFORM=%s\n' "${p}"
    printf 'TASK_ID=%s\n' "${task_id}"
    printf 'WORKTREE=%s\n' "${task_worktree[${task_id}]}"
    printf 'TASK_BRANCH=%s\n' "${task_branch[${task_id}]}"
    printf 'CODEX_EXIT=%s\n' "${codex_exit[${task_id}]}"
    printf 'CODEX_OUT_DIR=%s\n' "${task_codex_out_dir[${task_id}]}"
    printf 'CODEX_LAST_MESSAGE_PATH=%s\n' "${task_codex_last_message[${task_id}]}"
    printf 'CODEX_EVENTS_PATH=%s\n' "${task_codex_events[${task_id}]}"
    printf 'CODEX_STDERR_PATH=%s\n' "${task_codex_stderr[${task_id}]}"
done

if [[ "${LAUNCH_CODEX}" -eq 1 && "${DRY_RUN}" -eq 0 ]]; then
    for task_id in "${selected_task_ids[@]}"; do
        if [[ "${codex_exit[${task_id}]}" != "dry-run" && "${codex_exit[${task_id}]}" -ne 0 ]]; then
            exit 1
        fi
    done
fi
