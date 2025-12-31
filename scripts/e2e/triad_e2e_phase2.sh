#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/e2e/triad_e2e_phase2.sh --feature-dir <path> [options]

Purpose:
  Phase 2 of the end-to-end triad automation smoke:
    - start and finish C0-integ-core (merge code+test into integ-core; run integ-checks)
    - dispatch cross-platform smoke via GitHub Actions (self-hosted runners by default)
    - optionally start platform-fix integration tasks in parallel and run per-platform smoke
    - start and finish the final aggregator (C0-integ), re-run smoke, and fast-forward merge back to orchestration
    - optionally run feature cleanup (remove retained worktrees/prune branches)

Required:
  --feature-dir <path>         Feature Planning Pack dir (docs/project_management/next/<feature>)

Options:
  --remote <name>              Git remote for CI temp branches and push (default: origin)
  --runner-kind <kind>         github-hosted|self-hosted (default: self-hosted)
  --run-wsl                    Include WSL coverage in smoke (requires self-hosted runners)
  --workflow-ref <ref>         Ref containing the workflow definition (default: meta.automation.orchestration_branch)

  --platform-fixes <csv>       Force-start platform-fix tasks for these platforms (e.g., linux,macos,windows[,wsl])
                               If omitted, platform-fix tasks are NOT started unless --platform-fixes is provided.

  --codex-profile <p>          Passed to Codex (`codex exec --profile`)
  --codex-model <m>            Passed to Codex (`codex exec --model`)
  --codex-jsonl                Capture Codex JSONL events (uses `codex exec --json`)
  --skip-codex                 Do not launch Codex

  --push-orch                  Push orchestration branch after final merge-back
  --cleanup                    Run feature cleanup at the end (worktree retention model)
  --force-cleanup              Pass FORCE=1 to feature cleanup

  --log-dir <dir>              Log directory (default: target/e2e/<feature>/)
  --dry-run                    Print actions; do not mutate git/worktrees
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

utc_now() {
    date -u +%Y-%m-%dT%H:%M:%SZ
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

log() {
    echo "== $*" >&2
}

run() {
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ $*" >&2
        return 0
    fi
    echo "+ $*" >&2
    "$@"
}

set_task_status() {
    local tasks_json="$1"
    local task_id="$2"
    local status="$3"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ set ${tasks_json} ${task_id}.status=${status}" >&2
        return 0
    fi
    python3 - "${tasks_json}" "${task_id}" "${status}" <<'PY'
import json
import sys

path, task_id, status = sys.argv[1], sys.argv[2], sys.argv[3]
with open(path, "r", encoding="utf-8") as f:
    data = json.load(f)
tasks = data.get("tasks")
if not isinstance(tasks, list):
    raise SystemExit("tasks.json: missing tasks[]")
for t in tasks:
    if isinstance(t, dict) and t.get("id") == task_id:
        t["status"] = status
        break
else:
    raise SystemExit(f"tasks.json: task not found: {task_id}")
tmp = path + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
import os
os.replace(tmp, path)
PY
}

append_session_log() {
    local session_log="$1"
    local line="$2"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ append ${session_log}: ${line}" >&2
        return 0
    fi
    printf '%s\n' "${line}" >>"${session_log}"
}

task_branch() {
    local tasks_json="$1"
    local task_id="$2"
    jq -r --arg id "${task_id}" '.tasks[] | select(.id==$id) | .git_branch' "${tasks_json}"
}

merge_if_needed() {
    local repo_dir="$1"
    local branch="$2"
    if git -C "${repo_dir}" merge-base --is-ancestor "${branch}" HEAD >/dev/null 2>&1; then
        log "Already contains ${branch}: ${repo_dir}"
        return 0
    fi
    log "Merging ${branch} into $(git -C "${repo_dir}" rev-parse --abbrev-ref HEAD) at ${repo_dir}"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        echo "+ (cd ${repo_dir} && git merge --no-edit ${branch})" >&2
        return 0
    fi
    git -C "${repo_dir}" merge --no-edit "${branch}"
}

FEATURE_DIR=""
REMOTE="origin"
RUNNER_KIND="self-hosted"
RUN_WSL=0
WORKFLOW_REF=""
PLATFORM_FIXES_CSV=""

CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
SKIP_CODEX=0

PUSH_ORCH=0
CLEANUP=0
FORCE_CLEANUP=0

LOG_DIR=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --remote)
            REMOTE="${2:-}"
            shift 2
            ;;
        --runner-kind)
            RUNNER_KIND="${2:-}"
            shift 2
            ;;
        --run-wsl)
            RUN_WSL=1
            shift 1
            ;;
        --workflow-ref)
            WORKFLOW_REF="${2:-}"
            shift 2
            ;;
        --platform-fixes)
            PLATFORM_FIXES_CSV="${2:-}"
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
        --skip-codex)
            SKIP_CODEX=1
            shift 1
            ;;
        --push-orch)
            PUSH_ORCH=1
            shift 1
            ;;
        --cleanup)
            CLEANUP=1
            shift 1
            ;;
        --force-cleanup)
            FORCE_CLEANUP=1
            shift 1
            ;;
        --log-dir)
            LOG_DIR="${2:-}"
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

if [[ -z "${FEATURE_DIR}" ]]; then
    usage >&2
    die "Missing --feature-dir"
fi

case "${RUNNER_KIND}" in
    github-hosted|self-hosted) ;;
    *) die "Invalid --runner-kind: ${RUNNER_KIND}" ;;
esac

require_cmd git
require_cmd jq
require_cmd rg
require_cmd python3
require_cmd make
require_cmd gh

if [[ "${SKIP_CODEX}" -eq 0 ]]; then
    require_cmd codex
fi

gh auth status >/dev/null

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
SESSION_LOG="${FEATURE_DIR_ABS}/session_log.md"
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi

FEATURE_NAME="$(jq -r '.meta.feature // empty' "${TASKS_JSON}")"
ORCH_BRANCH="$(jq -r '.meta.automation.orchestration_branch // empty' "${TASKS_JSON}")"
if [[ -z "${FEATURE_NAME}" || -z "${ORCH_BRANCH}" ]]; then
    die "tasks.json must include meta.feature and meta.automation.orchestration_branch"
fi

if [[ -z "${WORKFLOW_REF}" ]]; then
    WORKFLOW_REF="${ORCH_BRANCH}"
fi

if [[ -z "${LOG_DIR}" ]]; then
    LOG_DIR="target/e2e/${FEATURE_NAME}"
fi
LOG_DIR_ABS="$(python_abs_path "${LOG_DIR}")"
mkdir -p "${LOG_DIR_ABS}"
LOG_PATH="${LOG_DIR_ABS}/phase2.log"

exec > >(tee -a "${LOG_PATH}") 2>&1

log "Repo: ${REPO_ROOT}"
log "Feature: ${FEATURE_DIR_ABS}"
log "Orchestration branch: ${ORCH_BRANCH}"
log "Remote: ${REMOTE}"
log "Runner kind: ${RUNNER_KIND}"
log "Workflow ref: ${WORKFLOW_REF}"
log "Log: ${LOG_PATH}"

log "Ensuring orchestration branch exists/checked out"
run make triad-orch-ensure FEATURE_DIR="${FEATURE_DIR}"

log "Starting C0-integ-core (worktree + optional Codex headless)"
start_core=(make triad-task-start FEATURE_DIR="${FEATURE_DIR}" TASK_ID="C0-integ-core")
if [[ "${SKIP_CODEX}" -eq 0 ]]; then start_core+=(LAUNCH_CODEX=1); fi
if [[ -n "${CODEX_PROFILE}" ]]; then start_core+=(CODEX_PROFILE="${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then start_core+=(CODEX_MODEL="${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then start_core+=(CODEX_JSONL=1); fi
core_out="$("${start_core[@]}")"
echo "${core_out}"
core_wt="$(printf '%s' "${core_out}" | awk -F= '$1=="WORKTREE"{sub($1"=","",$0); print $0}')"
if [[ -z "${core_wt}" ]]; then
    die "Could not parse WORKTREE from C0-integ-core task_start output"
fi

code_branch="$(task_branch "${TASKS_JSON}" "C0-code")"
test_branch="$(task_branch "${TASKS_JSON}" "C0-test")"
core_branch="$(task_branch "${TASKS_JSON}" "C0-integ-core")"

log "Merging code/test into C0-integ-core worktree"
merge_if_needed "${core_wt}" "${code_branch}"
merge_if_needed "${core_wt}" "${test_branch}"

log "Finishing C0-integ-core (runs integ-checks; no merge-back)"
run bash -lc "cd \"${core_wt}\" && make triad-task-finish TASK_ID=\"C0-integ-core\""

log "Marking C0-integ-core completed in tasks.json (orchestration branch)"
run git checkout "${ORCH_BRANCH}"
set_task_status "${TASKS_JSON}" "C0-integ-core" "completed"
append_session_log "${SESSION_LOG}" ""
append_session_log "${SESSION_LOG}" "END   $(utc_now) C0-integ-core (e2e smoke)"
run git add "${TASKS_JSON}" "${SESSION_LOG}"
run git commit -m "docs: complete C0-integ-core (${FEATURE_NAME})"

log "Dispatching cross-platform smoke (PLATFORM=all)"
smoke_cmd=(make feature-smoke FEATURE_DIR="${FEATURE_DIR}" PLATFORM=all RUNNER_KIND="${RUNNER_KIND}" WORKFLOW_REF="${WORKFLOW_REF}" REMOTE="${REMOTE}" CLEANUP=1)
if [[ "${RUN_WSL}" -eq 1 ]]; then smoke_cmd+=(RUN_WSL=1); fi
smoke_all_rc=0
set +e
if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "+ (cd ${core_wt} && ${smoke_cmd[*]})" >&2
    smoke_all_rc=0
else
    bash -lc "cd \"${core_wt}\" && ${smoke_cmd[*]}"
    smoke_all_rc=$?
fi
set -e

if [[ "${smoke_all_rc}" -ne 0 && -z "${PLATFORM_FIXES_CSV}" ]]; then
    die "Cross-platform smoke failed; re-run with --platform-fixes linux,macos,windows[,wsl] to exercise platform-fix tasks"
fi

if [[ "${smoke_all_rc}" -eq 0 && -z "${PLATFORM_FIXES_CSV}" ]]; then
    log "Smoke is green and --platform-fixes not provided; completing platform-fix tasks as no-op to unblock final aggregator"
    platforms_required="$(jq -r '.meta.platforms_required // [] | join(\",\")' "${TASKS_JSON}")"
    wsl_required="$(jq -r '.meta.wsl_required // false' "${TASKS_JSON}")"
    wsl_mode="$(jq -r '.meta.wsl_task_mode // \"bundled\"' "${TASKS_JSON}")"
    PLATFORM_FIXES_CSV="${platforms_required}"
    if [[ "${wsl_required}" == "true" && "${wsl_mode}" == "separate" ]]; then
        if [[ -n "${PLATFORM_FIXES_CSV}" ]]; then PLATFORM_FIXES_CSV="${PLATFORM_FIXES_CSV},wsl"; else PLATFORM_FIXES_CSV="wsl"; fi
    fi

    IFS=',' read -r -a platforms <<<"${PLATFORM_FIXES_CSV}"
    for p in "${platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        task_id="C0-integ-${p}"
        run git checkout "${ORCH_BRANCH}"
        set_task_status "${TASKS_JSON}" "${task_id}" "completed"
        append_session_log "${SESSION_LOG}" "END   $(utc_now) ${task_id} (no-op; smoke green)"
        run git add "${TASKS_JSON}" "${SESSION_LOG}"
        run git commit -m "docs: complete ${task_id} (no-op) (${FEATURE_NAME})"
    done
fi

if [[ -n "${PLATFORM_FIXES_CSV}" ]]; then
    log "Starting platform-fix tasks in parallel (forced): ${PLATFORM_FIXES_CSV}"
    pf_cmd=(make triad-task-start-platform-fixes FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="C0" PLATFORMS="${PLATFORM_FIXES_CSV}")
    if [[ "${SKIP_CODEX}" -eq 0 ]]; then pf_cmd+=(LAUNCH_CODEX=1); fi
    if [[ -n "${CODEX_PROFILE}" ]]; then pf_cmd+=(CODEX_PROFILE="${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then pf_cmd+=(CODEX_MODEL="${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then pf_cmd+=(CODEX_JSONL=1); fi
    pf_out="$("${pf_cmd[@]}")"
    echo "${pf_out}"

    IFS=',' read -r -a platforms <<<"${PLATFORM_FIXES_CSV}"
    for p in "${platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        task_id="C0-integ-${p}"
        wt_rel="$(jq -r --arg id "${task_id}" '.tasks[] | select(.id==$id) | .worktree' "${TASKS_JSON}")"
        wt_abs="$(python_abs_path "${wt_rel}")"

        log "Platform-fix ${task_id}: merging integ-core branch (${core_branch})"
        merge_if_needed "${wt_abs}" "${core_branch}"

        log "Platform-fix ${task_id}: finishing (per-platform smoke via CI)"
        run bash -lc "cd \"${wt_abs}\" && make triad-task-finish TASK_ID=\"${task_id}\" SMOKE=1 TASK_PLATFORM=\"${p}\""

        run git checkout "${ORCH_BRANCH}"
        set_task_status "${TASKS_JSON}" "${task_id}" "completed"
        append_session_log "${SESSION_LOG}" "END   $(utc_now) ${task_id} (e2e smoke)"
        run git add "${TASKS_JSON}" "${SESSION_LOG}"
        run git commit -m "docs: complete ${task_id} (${FEATURE_NAME})"
    done
fi

log "Starting final aggregator (C0-integ) via wrapper (requires deps completed)"
start_final=(make triad-task-start-integ-final FEATURE_DIR="${FEATURE_DIR}" SLICE_ID="C0")
if [[ "${SKIP_CODEX}" -eq 0 ]]; then start_final+=(LAUNCH_CODEX=1); fi
if [[ -n "${CODEX_PROFILE}" ]]; then start_final+=(CODEX_PROFILE="${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then start_final+=(CODEX_MODEL="${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then start_final+=(CODEX_JSONL=1); fi
final_out="$("${start_final[@]}")"
echo "${final_out}"
final_wt="$(printf '%s' "${final_out}" | awk -F= '$1=="WORKTREE"{sub($1"=","",$0); print $0}')"
if [[ -z "${final_wt}" ]]; then
    die "Could not parse WORKTREE from final task_start output"
fi

log "Final aggregator: merging integ-core branch (${core_branch})"
merge_if_needed "${final_wt}" "${core_branch}"

if [[ -n "${PLATFORM_FIXES_CSV}" ]]; then
    IFS=',' read -r -a platforms <<<"${PLATFORM_FIXES_CSV}"
    for p in "${platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        merge_if_needed "${final_wt}" "$(task_branch "${TASKS_JSON}" "C0-integ-${p}")"
    done
fi

log "Final aggregator: dispatching cross-platform smoke (PLATFORM=all)"
run bash -lc "cd \"${final_wt}\" && ${smoke_cmd[*]}"

log "Final aggregator: finishing (runs integ-checks; merges back FF-only)"
run bash -lc "cd \"${final_wt}\" && make triad-task-finish TASK_ID=\"C0-integ\""

log "Marking C0-integ completed in tasks.json (orchestration branch)"
run git checkout "${ORCH_BRANCH}"
set_task_status "${TASKS_JSON}" "C0-integ" "completed"
append_session_log "${SESSION_LOG}" "END   $(utc_now) C0-integ (e2e smoke)"
run git add "${TASKS_JSON}" "${SESSION_LOG}"
run git commit -m "docs: complete C0-integ (${FEATURE_NAME})"

if [[ "${PUSH_ORCH}" -eq 1 ]]; then
    log "Pushing orchestration branch: ${ORCH_BRANCH} -> ${REMOTE}"
    run git push "${REMOTE}" "${ORCH_BRANCH}"
fi

if [[ "${CLEANUP}" -eq 1 ]]; then
    log "Running feature cleanup (retention model)"
    cleanup_cmd=(make triad-feature-cleanup FEATURE_DIR="${FEATURE_DIR}" REMOVE_WORKTREES=1 PRUNE_LOCAL=1)
    if [[ "${FORCE_CLEANUP}" -eq 1 ]]; then cleanup_cmd+=(FORCE=1); fi
    run "${cleanup_cmd[@]}" DRY_RUN=1
    run "${cleanup_cmd[@]}"
    run git checkout "${ORCH_BRANCH}"
    set_task_status "${TASKS_JSON}" "FZ-feature-cleanup" "completed"
    append_session_log "${SESSION_LOG}" "END   $(utc_now) FZ-feature-cleanup"
    run git add "${TASKS_JSON}" "${SESSION_LOG}"
    run git commit -m "docs: complete FZ-feature-cleanup (${FEATURE_NAME})"
fi

echo ""
echo "PHASE2_OK=1"
echo "FEATURE_DIR=${FEATURE_DIR}"
echo "ORCH_BRANCH=${ORCH_BRANCH}"
echo "LOG=${LOG_PATH}"
