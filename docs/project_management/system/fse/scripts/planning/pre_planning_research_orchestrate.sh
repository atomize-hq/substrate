#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  pre_planning_research_orchestrate.sh --feature-dir <path> [--start-at <step>] [--poll-s <seconds>]

Required:
  --feature-dir <path>   Planning Pack dir (docs/project_management/packs/<bucket>/<feature>)

Options:
  --start-at <step>      spec-manifest|impact-map|min-spec-draft|CI-checkpoint|workstream-triage
                         (default: spec-manifest)
  --poll-s <seconds>     Poll interval for handoff/exit checks (default: 30)

Environment (optional; passed through to the planning runner):
  CODEX_PROFILE=<p>
  CODEX_MODEL=<m>
  CODEX_JSONL=1

Behavior:
  - Requires a clean orchestration checkout (git status must be empty).
  - Archives existing step log dirs for START_AT and downstream: <step> -> <step>_run_N
  - Launches the 5-step overlap chain with staggered overlap, triggered by upstream handoff.md.
    - Special case: `workstream-triage` is triggered by `min-spec-draft` handoff (so triage can draft while CI-checkpoint is still running).
  - Commits allowlisted tracked outputs after each step succeeds.
  - Always writes a wrapper summary under: <FEATURE_DIR>/logs/pre_planning_wrapper/<UTC_TS>/summary.md
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

need_cmd() {
    local cmd="$1"
    if ! command -v "${cmd}" >/dev/null 2>&1; then
        die "${cmd} not found on PATH"
    fi
}

FEATURE_DIR_RAW=""
START_AT="spec-manifest"
POLL_S="30"
TRANSIENT_RETRY_MAX="${PM_PRE_PLANNING_TRANSIENT_RETRY_MAX:-3}"
TRANSIENT_RETRY_BACKOFF_S="${PM_PRE_PLANNING_TRANSIENT_RETRY_BACKOFF_S:-15}"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR_RAW="${2:-}"
            shift 2
            ;;
        --start-at)
            START_AT="${2:-}"
            shift 2
            ;;
        --poll-s)
            POLL_S="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "unknown arg: $1 (use --help)"
            ;;
    esac
done

if [[ -z "${FEATURE_DIR_RAW}" ]]; then
    usage >&2
    die "missing --feature-dir"
fi

case "${START_AT}" in
    spec-manifest|impact-map|min-spec-draft|CI-checkpoint|workstream-triage) ;;
    *)
        die "invalid --start-at: ${START_AT} (expected step dir name like spec-manifest)"
        ;;
esac

if [[ ! "${POLL_S}" =~ ^[0-9]+$ ]] || [[ "${POLL_S}" -lt 1 ]]; then
    die "invalid --poll-s: ${POLL_S} (expected integer seconds >= 1)"
fi
if [[ ! "${TRANSIENT_RETRY_MAX}" =~ ^[0-9]+$ ]] || [[ "${TRANSIENT_RETRY_MAX}" -lt 0 ]]; then
    die "invalid PM_PRE_PLANNING_TRANSIENT_RETRY_MAX: ${TRANSIENT_RETRY_MAX} (expected integer >= 0)"
fi
if [[ ! "${TRANSIENT_RETRY_BACKOFF_S}" =~ ^[0-9]+$ ]] || [[ "${TRANSIENT_RETRY_BACKOFF_S}" -lt 1 ]]; then
    die "invalid PM_PRE_PLANNING_TRANSIENT_RETRY_BACKOFF_S: ${TRANSIENT_RETRY_BACKOFF_S} (expected integer >= 1)"
fi

need_cmd git
need_cmd python3
need_cmd jq
need_cmd codex

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

if [[ -n "$(git status --porcelain=v1)" ]]; then
    die "orchestration checkout is dirty; commit or stash before running"
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system/fse}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
RUNNER="${PM_PRE_PLANNING_RUNNER:-${PLANNING_SCRIPTS_DIR}/run_planning_agent.sh}"
if [[ "${RUNNER}" != /* ]]; then
    RUNNER="${REPO_ROOT}/${RUNNER}"
fi
[[ -x "${RUNNER}" ]] || die "missing runner: ${RUNNER}"
ALIGNMENT_REPORTER="${PM_PRE_PLANNING_ALIGNMENT_REPORTER:-${PLANNING_SCRIPTS_DIR}/wrapper_alignment_report.py}"
if [[ "${ALIGNMENT_REPORTER}" != /* ]]; then
    ALIGNMENT_REPORTER="${REPO_ROOT}/${ALIGNMENT_REPORTER}"
fi

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"
FSE_PRE_PLANNING_METADATA_REL="${FEATURE_DIR_REL}/fse_pre_planning.json"
FSE_PRE_PLANNING_METADATA_ABS="${FEATURE_DIR_ABS}/fse_pre_planning.json"

PRE_PLANNING_DIR_REL="${FEATURE_DIR_REL}/pre-planning"
PRE_PLANNING_DIR_ABS="${FEATURE_DIR_ABS}/pre-planning"
mkdir -p "${PRE_PLANNING_DIR_ABS}"

LOGS_DIR="${FEATURE_DIR_ABS}/logs"
mkdir -p "${LOGS_DIR}"

RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
WRAPPER_DIR="${LOGS_DIR}/pre_planning_wrapper/${RUN_TS}"
mkdir -p "${WRAPPER_DIR}"
SUMMARY_PATH="${WRAPPER_DIR}/summary.md"

steps=(spec-manifest impact-map min-spec-draft CI-checkpoint workstream-triage)
agents=(spec_manifest impact_map min_spec_draft ci_checkpoint workstream_triage)

step_index_of() {
    local needle="$1"
    local i
    for i in "${!steps[@]}"; do
        if [[ "${steps[$i]}" == "${needle}" ]]; then
            printf '%s\n' "${i}"
            return 0
        fi
    done
    return 1
}

start_index="$(step_index_of "${START_AT}")" || die "unable to resolve start index for ${START_AT}"

next_run_n() {
    local step="$1"
    local max=0
    local d
    shopt -s nullglob
    for d in "${LOGS_DIR}/${step}_run_"*; do
        [[ -d "${d}" ]] || continue
        local suffix="${d##*_run_}"
        if [[ "${suffix}" =~ ^[0-9]+$ ]] && [[ "${suffix}" -gt "${max}" ]]; then
            max="${suffix}"
        fi
    done
    shopt -u nullglob
    printf '%s\n' "$((max + 1))"
}

archive_step_dir_if_exists() {
    local step="$1"
    local step_dir="${LOGS_DIR}/${step}"
    if [[ ! -e "${step_dir}" ]]; then
        return 0
    fi
    if [[ ! -d "${step_dir}" ]]; then
        die "expected step log dir to be a directory: ${step_dir}"
    fi
    local n
    n="$(next_run_n "${step}")"
    local archived="${LOGS_DIR}/${step}_run_${n}"
    mv "${step_dir}" "${archived}"
}

append_summary() {
    printf '%s\n' "$*" >>"${SUMMARY_PATH}"
}

append_summary "# Pre-Planning Research Orchestration Summary"
append_summary ""
append_summary "- Feature dir: \`${FEATURE_DIR_REL}/\`"
append_summary "- FSE metadata: \`${FSE_PRE_PLANNING_METADATA_REL}\`"
append_summary "- Run (UTC): \`${RUN_TS}\`"
append_summary "- Start at: \`${START_AT}\`"
append_summary "- Poll interval: \`${POLL_S}s\`"
append_summary ""

echo "Pre-planning research started: ${FEATURE_DIR_REL}/"
echo "Wrapper summary: ${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/summary.md"
echo "Poll interval: ${POLL_S}s"
echo ""

# Prepare (archive + recreate) step log dirs for START_AT and downstream.
append_summary "## Log directory preparation"
for i in "${!steps[@]}"; do
    if [[ "${i}" -lt "${start_index}" ]]; then
        continue
    fi
    step="${steps[$i]}"
    step_dir="${LOGS_DIR}/${step}"
    if [[ -d "${step_dir}" ]]; then
        archive_step_dir_if_exists "${step}"
        append_summary "- Archived: \`${FEATURE_DIR_REL}/logs/${step}/\` → \`${FEATURE_DIR_REL}/logs/${step}_run_*/\`"
    fi
    mkdir -p "${step_dir}/runs"
done
append_summary ""

runner_pids=()
runner_rcs=()
commit_shas=()
runner_start_epoch=()
runner_end_epoch=()
# First-seen handoff timestamp (stable even if handoff.md is later overwritten).
handoff_seen_epoch=()
step_phase=()
step_state=()
step_ready_for_commit=()
step_wait_reason=()
step_retry_count=()

launch_step() {
    local idx="$1"
    local phase="$2"
    local step="${steps[$idx]}"
    local agent="${agents[$idx]}"

    local log_path="${WRAPPER_DIR}/${step}.runner.log"
    # Launch the runner in the background; runner manages Codex child + codex.pid/stderr.log.
    local -a args=("${RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --agent "${agent}" --phase "${phase}")
    if [[ -n "${CODEX_PROFILE:-}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL:-}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL:-0}" = "1" ]]; then args+=(--codex-jsonl); fi
    PM_PLANNING_ORCHESTRATED=1 PM_PRE_PLANNING_WRAPPER_PHASE="${phase}" "${args[@]}" >"${log_path}" 2>&1 &
    local pid="$!"
    runner_pids[idx]="${pid}"
    runner_rcs[idx]=""
    commit_shas[idx]=""
    if [[ -z "${runner_start_epoch[idx]:-}" ]]; then
        runner_start_epoch[idx]="$(date -u +%s)"
    fi
    runner_end_epoch[idx]=""
    step_phase[idx]="${phase}"
    step_state[idx]="running_${phase}"
    step_wait_reason[idx]=""
    append_summary "- Started: \`${step}\` \`${phase}\` (agent=\`${agent}\`, pid=\`${pid}\`, runner_log=\`$(basename "${log_path}")\`)"

    echo "Started: ${step} [${phase}] (agent=${agent}, pid=${pid})"
    echo "  Runner log: ${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/${step}.runner.log"
    echo "  Step stderr: ${FEATURE_DIR_REL}/logs/${step}/stderr.log"
    echo "  Tip: tail -f ${FEATURE_DIR_REL}/logs/${step}/stderr.log"
    echo ""
}

kill_downstream() {
    local from_idx="$1"
    local j
    for j in "${!steps[@]}"; do
        if [[ "${j}" -lt "${from_idx}" ]]; then
            continue
        fi
        local pid="${runner_pids[j]:-}"
        [[ -n "${pid}" ]] || continue
        if kill -0 "${pid}" 2>/dev/null; then
            kill "${pid}" >/dev/null 2>&1 || true
        fi
    done
}

step_commit_message() {
    local step="$1"
    case "${step}" in
        spec-manifest) printf '%s\n' "docs: pre-planning spec manifest" ;;
        impact-map) printf '%s\n' "docs: pre-planning impact map" ;;
        min-spec-draft) printf '%s\n' "docs: pre-planning minimal spec draft" ;;
        CI-checkpoint) printf '%s\n' "docs: pre-planning CI checkpoint plan" ;;
        workstream-triage) printf '%s\n' "docs: pre-planning workstream triage" ;;
        *) die "unknown step: ${step}" ;;
    esac
}

step_allowlist() {
    local step="$1"
    case "${step}" in
        spec-manifest) printf '%s\n' "${PRE_PLANNING_DIR_REL}/spec_manifest.md" ;;
        impact-map) printf '%s\n' "${PRE_PLANNING_DIR_REL}/impact_map.md" ;;
        min-spec-draft) printf '%s\n' "${PRE_PLANNING_DIR_REL}/minimal_spec_draft.md" ;;
        CI-checkpoint)
            printf '%s\n' "${PRE_PLANNING_DIR_REL}/ci_checkpoint_plan.md"
            ;;
        workstream-triage) printf '%s\n' "${PRE_PLANNING_DIR_REL}/workstream_triage.md" ;;
        *) die "unknown step: ${step}" ;;
    esac
}

step_required_outputs() {
    local step="$1"
    case "${step}" in
        spec-manifest) printf '%s\n' "${PRE_PLANNING_DIR_REL}/spec_manifest.md" ;;
        impact-map) printf '%s\n' "${PRE_PLANNING_DIR_REL}/impact_map.md" ;;
        min-spec-draft) printf '%s\n' "${PRE_PLANNING_DIR_REL}/minimal_spec_draft.md" ;;
        CI-checkpoint) printf '%s\n' "${PRE_PLANNING_DIR_REL}/ci_checkpoint_plan.md" ;;
        workstream-triage) printf '%s\n' "${PRE_PLANNING_DIR_REL}/workstream_triage.md" ;;
        *) die "unknown step: ${step}" ;;
    esac
}

step_staged_rel() {
    local step="$1"
    local repo_rel="$2"
    local within_feature="${repo_rel#"${FEATURE_DIR_REL}"/}"
    if [[ "${within_feature}" == "${repo_rel}" ]]; then
        die "cannot resolve staged path outside feature dir: ${repo_rel}"
    fi
    printf '%s\n' "${FEATURE_DIR_REL}/logs/${step}/staged/${within_feature}"
}

run_step_closeout_validation() {
    local step="$1"
    case "${step}" in
        impact-map)
            bash "${PLANNING_SCRIPTS_DIR}/micro_lint.sh" --feature-dir "${FEATURE_DIR_ABS}" --agent impact_map -- "${PRE_PLANNING_DIR_ABS}/impact_map.md"
            ;;
        workstream-triage)
            bash "${PLANNING_SCRIPTS_DIR}/micro_lint.sh" --feature-dir "${FEATURE_DIR_ABS}" --agent workstream_triage -- "${PRE_PLANNING_DIR_ABS}/workstream_triage.md"
            ;;
        *)
            return 0
            ;;
    esac
}

latest_run_dir_for_step() {
    local step="$1"
    local runs_dir="${LOGS_DIR}/${step}/runs"
    [[ -d "${runs_dir}" ]] || return 1
    find "${runs_dir}" -mindepth 1 -maxdepth 1 -type d | sort | tail -n 1
}

step_latest_run_state_path() {
    local step="$1"
    local run_dir
    run_dir="$(latest_run_dir_for_step "${step}")" || return 1
    local run_state="${run_dir}/run_state.json"
    [[ -f "${run_state}" ]] || return 1
    printf '%s\n' "${run_state}"
}

step_run_is_retryable_transient() {
    local step="$1"
    local run_state
    run_state="$(step_latest_run_state_path "${step}")" || return 1

    local turn_completed assistant_message_present events_rel events_abs
    turn_completed="$(jq -r '.turn_completed // false' "${run_state}" 2>/dev/null || printf 'false\n')"
    assistant_message_present="$(jq -r '.assistant_message_present // false' "${run_state}" 2>/dev/null || printf 'false\n')"
    if [[ "${turn_completed}" == "true" ]] || [[ "${assistant_message_present}" == "true" ]]; then
        return 1
    fi

    events_rel="$(jq -r '.events_path // empty' "${run_state}" 2>/dev/null || true)"
    [[ -n "${events_rel}" ]] || return 1
    if [[ "${events_rel}" = /* ]]; then
        events_abs="${events_rel}"
    else
        events_abs="${REPO_ROOT}/${events_rel}"
    fi
    [[ -f "${events_abs}" ]] || return 1

    grep -Eqi 'at capacity|try a different model|overloaded|rate limit|temporarily unavailable' "${events_abs}"
}

retry_step_if_transient() {
    local idx="$1"
    local phase="$2"
    local step="${steps[$idx]}"
    local retries="${step_retry_count[idx]:-0}"

    if [[ "${TRANSIENT_RETRY_MAX}" -eq 0 ]] || [[ "${retries}" -ge "${TRANSIENT_RETRY_MAX}" ]]; then
        return 1
    fi
    if ! step_run_is_retryable_transient "${step}"; then
        return 1
    fi

    retries="$((retries + 1))"
    step_retry_count[idx]="${retries}"
    runner_pids[idx]=""
    runner_rcs[idx]=""
    runner_end_epoch[idx]=""
    step_state[idx]="retry_wait_${phase}"
    step_wait_reason[idx]="transient Codex capacity/overload"

    echo "Retrying: ${step} [${phase}] after transient Codex capacity error (attempt ${retries}/${TRANSIENT_RETRY_MAX})"
    echo "  Backoff: ${TRANSIENT_RETRY_BACKOFF_S}s"
    echo ""
    append_summary "- Retrying: \`${step}\` \`${phase}\` after transient Codex capacity/overload error (attempt \`${retries}/${TRANSIENT_RETRY_MAX}\`)"
    sleep "${TRANSIENT_RETRY_BACKOFF_S}"
    launch_step "${idx}" "${phase}"
    return 0
}

publish_stable_last_message() {
    local step="$1"
    local run_dir
    run_dir="$(latest_run_dir_for_step "${step}")" || die "missing run dir for ${step}"
    local src="${run_dir}/last_message.run.md"
    local dest="${LOGS_DIR}/${step}/last_message.md"
    [[ -f "${src}" ]] || die "missing run last_message for ${step}: ${src}"
    cp "${src}" "${dest}"
}

commit_step_outputs() {
    local idx="$1"
    local step="${steps[$idx]}"
    local msg
    msg="$(step_commit_message "${step}")"

    local -a allow=()
    local -a required=()
    local p
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        allow+=("${p}")
    done < <(step_allowlist "${step}")
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        required+=("${p}")
    done < <(step_required_outputs "${step}")

    local backup_dir="${WRAPPER_DIR}/promotion_backup/${step}"
    rm -rf "${backup_dir}"
    mkdir -p "${backup_dir}"

    local target_rel staged_rel target_abs staged_abs backup_abs
    for target_rel in "${allow[@]}"; do
        staged_rel="$(step_staged_rel "${step}" "${target_rel}")"
        staged_abs="${REPO_ROOT}/${staged_rel}"
        target_abs="${REPO_ROOT}/${target_rel}"
        if [[ ! -f "${staged_abs}" ]]; then
            continue
        fi

        backup_abs="${backup_dir}/${target_rel#"${FEATURE_DIR_REL}"/}"
        mkdir -p "$(dirname "${backup_abs}")"
        if [[ -f "${target_abs}" ]]; then
            cp "${target_abs}" "${backup_abs}"
        else
            : > "${backup_abs}.absent"
        fi

        mkdir -p "$(dirname "${target_abs}")"
        cp "${staged_abs}" "${target_abs}"
    done

    restore_failed_promotion() {
        local target_rel_local="$1"
        local target_abs_local="${REPO_ROOT}/${target_rel_local}"
        local backup_base="${backup_dir}/${target_rel_local#"${FEATURE_DIR_REL}"/}"
        if [[ -f "${backup_base}.absent" ]]; then
            rm -f "${target_abs_local}"
        elif [[ -f "${backup_base}" ]]; then
            mkdir -p "$(dirname "${target_abs_local}")"
            cp "${backup_base}" "${target_abs_local}"
        fi
    }

    for p in "${required[@]}"; do
        if [[ ! -f "${REPO_ROOT}/${p}" ]]; then
            for target_rel in "${allow[@]}"; do
                restore_failed_promotion "${target_rel}"
            done
            die "required promoted output missing for ${step}: ${p}"
        fi
    done

    if ! run_step_closeout_validation "${step}"; then
        for target_rel in "${allow[@]}"; do
            restore_failed_promotion "${target_rel}"
        done
        die "closeout validation failed after staged promotion (step=${step})"
    fi

    local allowlisted allow_p
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        allowlisted=0
        for allow_p in "${allow[@]}"; do
            if [[ "${allow_p}" == "${p}" ]]; then
                allowlisted=1
                break
            fi
        done
        if [[ "${allowlisted}" -eq 0 ]]; then
            for target_rel in "${allow[@]}"; do
                restore_failed_promotion "${target_rel}"
            done
            die "refusing to commit: unexpected tracked change within feature dir: ${p} (step=${step})"
        fi
    done < <(git diff --name-only -- "${FEATURE_DIR_REL}" | sed '/^$/d')

    git add -- "${allow[@]}" >/dev/null 2>&1 || true
    if git diff --cached --quiet; then
        publish_stable_last_message "${step}"
        return 0
    fi

    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        allowlisted=0
        for allow_p in "${allow[@]}"; do
            if [[ "${allow_p}" == "${p}" ]]; then
                allowlisted=1
                break
            fi
        done
        if [[ "${allowlisted}" -eq 0 ]]; then
            for target_rel in "${allow[@]}"; do
                restore_failed_promotion "${target_rel}"
            done
            die "refusing to commit non-allowlisted path: ${p} (step=${step})"
        fi
    done < <(git diff --cached --name-only | sed '/^$/d')

    git commit -m "${msg}" >/dev/null
    commit_shas[idx]="$(git rev-parse HEAD)"
    publish_stable_last_message "${step}"
}

fmt_hms() {
    local total="${1:-0}"
    if [[ -z "${total}" ]] || [[ ! "${total}" =~ ^[0-9]+$ ]]; then
        printf '%s\n' ""
        return 0
    fi
    local h=$((total / 3600))
    local m=$(((total % 3600) / 60))
    local s=$((total % 60))
    printf '%02d:%02d:%02d' "${h}" "${m}" "${s}"
}

epoch_to_utc_iso() {
    local epoch="${1:-}"
    if [[ -z "${epoch}" ]] || [[ ! "${epoch}" =~ ^[0-9]+$ ]]; then
        printf '%s\n' ""
        return 0
    fi
    python3 - "${epoch}" <<'PY'
from __future__ import annotations

import datetime as dt
import sys

epoch = int(sys.argv[1])
print(dt.datetime.fromtimestamp(epoch, tz=dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
PY
}

file_mtime_epoch() {
    local path="$1"
    if [[ ! -f "${path}" ]]; then
        return 0
    fi
    python3 - "${path}" <<'PY'
from __future__ import annotations

import os
import sys

print(int(os.path.getmtime(sys.argv[1])))
PY
}

handoff_path_for() {
    local step="$1"
    printf '%s\n' "${LOGS_DIR}/${step}/handoff.md"
}

step_staged_required_outputs_exist() {
    local step="$1"
    local target_rel staged_rel staged_abs
    while IFS= read -r target_rel; do
        [[ -n "${target_rel}" ]] || continue
        staged_rel="$(step_staged_rel "${step}" "${target_rel}")"
        staged_abs="${REPO_ROOT}/${staged_rel}"
        if [[ ! -f "${staged_abs}" ]]; then
            return 1
        fi
    done < <(step_required_outputs "${step}")
    return 0
}

step_phase_a_required_outputs_exist() {
    local idx="$1"
    local step="${steps[$idx]}"
    local required_rel required_abs

    phase_a_required_outputs_for_step() {
        local step_name="$1"
        case "${step_name}" in
            spec-manifest)
                printf '%s\n' "${FEATURE_DIR_REL}/logs/spec-manifest/handoff.md"
                ;;
            impact-map)
                printf '%s\n' "${FEATURE_DIR_REL}/logs/impact-map/scratch.md"
                printf '%s\n' "${FEATURE_DIR_REL}/logs/impact-map/handoff.md"
                ;;
            min-spec-draft)
                printf '%s\n' "${FEATURE_DIR_REL}/logs/min-spec-draft/scratch.md"
                printf '%s\n' "${FEATURE_DIR_REL}/logs/min-spec-draft/handoff.md"
                ;;
            CI-checkpoint)
                printf '%s\n' "${FEATURE_DIR_REL}/logs/CI-checkpoint/scratch.md"
                printf '%s\n' "${FEATURE_DIR_REL}/logs/CI-checkpoint/handoff.md"
                ;;
            workstream-triage)
                printf '%s\n' "${FEATURE_DIR_REL}/logs/workstream-triage/workstream_triage_draft.md"
                ;;
            *)
                die "unknown step: ${step_name}"
                ;;
        esac
    }

    while IFS= read -r required_rel; do
        [[ -n "${required_rel}" ]] || continue
        required_abs="${REPO_ROOT}/${required_rel}"
        if [[ ! -f "${required_abs}" ]]; then
            return 1
        fi
    done < <(phase_a_required_outputs_for_step "${step}")
    return 0
}

step_phase_b_dependency() {
    local step="$1"
    case "${step}" in
        spec-manifest) printf '%s\n' "" ;;
        impact-map) printf '%s\n' "spec-manifest" ;;
        min-spec-draft) printf '%s\n' "impact-map" ;;
        CI-checkpoint) printf '%s\n' "min-spec-draft" ;;
        workstream-triage) printf '%s\n' "CI-checkpoint" ;;
        *) die "unknown step: ${step}" ;;
    esac
}

feature_dir_git_clean_for_phase_b() {
    local line path
    while IFS= read -r line; do
        [[ -n "${line}" ]] || continue
        path="${line:3}"
        if [[ "${path}" == *" -> "* ]]; then
            path="${path##* -> }"
        fi
        if [[ "${path}" == "${FEATURE_DIR_REL}/logs" ]] || [[ "${path}" == "${FEATURE_DIR_REL}/logs/"* ]]; then
            continue
        fi
        return 1
    done < <(git status --porcelain=v1 --untracked-files=all -- "${FEATURE_DIR_REL}")
    return 0
}

phase_b_gate_reason() {
    local idx="$1"
    local step="${steps[$idx]}"
    local dep_step
    dep_step="$(step_phase_b_dependency "${step}")"
    local reasons=()
    local dep_output

    if [[ -n "${dep_step}" ]]; then
        while IFS= read -r dep_output; do
            [[ -n "${dep_output}" ]] || continue
            if [[ ! -f "${REPO_ROOT}/${dep_output}" ]]; then
                reasons+=("missing canonical prerequisite ${dep_output}")
            fi
        done < <(step_required_outputs "${dep_step}")
    fi

    if ! feature_dir_git_clean_for_phase_b; then
        reasons+=("feature-dir git state is not clean outside logs/")
    fi

    if [[ "${#reasons[@]}" -eq 0 ]]; then
        printf '%s\n' ""
    else
        local joined=""
        local reason
        for reason in "${reasons[@]}"; do
            if [[ -n "${joined}" ]]; then
                joined="${joined}; "
            fi
            joined="${joined}${reason}"
        done
        printf '%s\n' "${joined}"
    fi
}

phase_b_gate_ready() {
    local idx="$1"
    [[ -z "$(phase_b_gate_reason "${idx}")" ]]
}

phase_a_can_close_with_wait() {
    local idx="$1"
    local step="${steps[$idx]}"
    if ! step_phase_a_required_outputs_exist "${idx}"; then
        return 1
    fi
    if step_staged_required_outputs_exist "${step}"; then
        return 1
    fi
    if phase_b_gate_ready "${idx}"; then
        return 1
    fi
    return 0
}

phase_a_can_advance_to_phase_b() {
    local idx="$1"
    local step="${steps[$idx]}"
    if ! step_phase_a_required_outputs_exist "${idx}"; then
        return 1
    fi
    if step_staged_required_outputs_exist "${step}"; then
        return 1
    fi
    return 0
}

cleanup_on_exit() {
    local rc="$?"
    # Best-effort cleanup: ensure we don't leave planning runners running on failure.
    if [[ "${rc}" -ne 0 ]]; then
        kill_downstream "${start_index}" || true
    fi
    append_summary ""
    append_summary "## Timing (UTC)"
    append_summary ""
    append_summary "| step | start | handoff | end | duration | handoff_after |"
    append_summary "| --- | --- | --- | --- | --- | --- |"
    local ti
    for ti in "${!steps[@]}"; do
        if [[ "${ti}" -lt "${start_index}" ]]; then
            continue
        fi
        local step="${steps[$ti]}"
        local start_epoch="${runner_start_epoch[ti]:-}"
        local end_epoch="${runner_end_epoch[ti]:-}"

        local start_iso end_iso dur_hms
        start_iso="$(epoch_to_utc_iso "${start_epoch}")"
        end_iso="$(epoch_to_utc_iso "${end_epoch}")"
        dur_hms=""
        if [[ -n "${start_epoch}" && -n "${end_epoch}" ]] && [[ "${start_epoch}" =~ ^[0-9]+$ ]] && [[ "${end_epoch}" =~ ^[0-9]+$ ]] && [[ "${end_epoch}" -ge "${start_epoch}" ]]; then
            dur_hms="$(fmt_hms "$((end_epoch - start_epoch))")"
        fi

        local handoff_path handoff_epoch handoff_iso handoff_after_hms
        handoff_path="$(handoff_path_for "${step}")"
        handoff_epoch="${handoff_seen_epoch[ti]:-}"
        if [[ -z "${handoff_epoch}" ]]; then
            handoff_epoch="$(file_mtime_epoch "${handoff_path}" || true)"
        fi
        handoff_iso="$(epoch_to_utc_iso "${handoff_epoch}")"
        handoff_after_hms=""
        if [[ -n "${start_epoch}" && -n "${handoff_epoch}" ]] && [[ "${start_epoch}" =~ ^[0-9]+$ ]] && [[ "${handoff_epoch}" =~ ^[0-9]+$ ]] && [[ "${handoff_epoch}" -ge "${start_epoch}" ]]; then
            handoff_after_hms="$(fmt_hms "$((handoff_epoch - start_epoch))")"
        fi

        append_summary "| \`${step}\` | \`${start_iso}\` | \`${handoff_iso}\` | \`${end_iso}\` | \`${dur_hms}\` | \`${handoff_after_hms}\` |"
    done
    append_summary ""
    append_summary "## Results"
    local i
    for i in "${!steps[@]}"; do
        if [[ "${i}" -lt "${start_index}" ]]; then
            continue
        fi
        local step="${steps[$i]}"
        local agent="${agents[$i]}"
        local pid="${runner_pids[i]:-}"
        local rcrc="${runner_rcs[i]:-}"
        local sha="${commit_shas[i]:-}"
        local step_dir_rel="${FEATURE_DIR_REL}/logs/${step}"
        local runner_log_rel="${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/${step}.runner.log"
        append_summary "- \`${step}\` (agent=\`${agent}\` pid=\`${pid:-}\` rc=\`${rcrc:-}\` commit=\`${sha:-}\`) — logs: \`${step_dir_rel}/\` sentinel: \`${step_dir_rel}/last_message.md\` runner_log: \`${runner_log_rel}\`"
    done
    append_summary ""
    append_summary "## Workstream triage"
    append_summary "- Tracked: \`${PRE_PLANNING_DIR_REL}/workstream_triage.md\`"
    append_summary "- Draft (logs): \`${FEATURE_DIR_REL}/logs/workstream-triage/workstream_triage_draft.md\`"
    append_summary ""

    # Wrapper-detected misalignment triage + consolidated follow-ups (report-only; no edits).
    local alignment_report_abs alignment_report_rel alignment_report_stderr_rel
    local tracked_alignment_abs tracked_alignment_rel
    alignment_report_abs="${WRAPPER_DIR}/alignment_report.md"
    alignment_report_rel="${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/alignment_report.md"
    alignment_report_stderr_rel="${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/alignment_report.stderr.log"
    tracked_alignment_abs="${PRE_PLANNING_DIR_ABS}/alignment_report.md"
    tracked_alignment_rel="${PRE_PLANNING_DIR_REL}/alignment_report.md"
    if [[ -x "${ALIGNMENT_REPORTER}" ]] || [[ -f "${ALIGNMENT_REPORTER}" ]]; then
        if python3 "${ALIGNMENT_REPORTER}" --feature-dir "${FEATURE_DIR_REL}" >"${alignment_report_abs}" 2>"${WRAPPER_DIR}/alignment_report.stderr.log"; then
            append_summary "## Alignment triage (wrapper-compiled)"
            append_summary ""
            append_summary "- Full report: \`${alignment_report_rel}\`"
            append_summary "- Tracked: \`${tracked_alignment_rel}\`"
            append_summary ""
            cat "${alignment_report_abs}" >>"${SUMMARY_PATH}"
            append_summary ""

            # On successful runs, also persist the report as a tracked pack artifact so it doesn't get lost in logs.
            # This is report-only (no rewriting of other pack docs) but is intentionally committed.
            if [[ "${rc}" -eq 0 ]]; then
                cp "${alignment_report_abs}" "${tracked_alignment_abs}"
                if [[ -n "$(git status --porcelain=v1 -- "${tracked_alignment_rel}")" ]]; then
                    git add -- "${tracked_alignment_rel}"
                    if ! git diff --cached --quiet; then
                        if git commit -m "docs: pre-planning alignment report" >/dev/null; then
                            echo "Committed: wrapper alignment report"
                        fi
                    fi
                fi
            fi
        elif [[ ! -f "${FEATURE_DIR_ABS}/tasks.json" && -f "${FSE_PRE_PLANNING_METADATA_ABS}" ]]; then
            cat >"${alignment_report_abs}" <<EOF
# Alignment Report

## Status

- Wrapper generated the fallback alignment report because the primary FSE alignment reporter failed during this run. Inspect \`${alignment_report_stderr_rel}\` for the concrete reporter error.

## Current pack artifacts

- Metadata: \`${FSE_PRE_PLANNING_METADATA_REL}\`
- Spec manifest: \`${PRE_PLANNING_DIR_REL}/spec_manifest.md\`
- Impact map: \`${PRE_PLANNING_DIR_REL}/impact_map.md\`
- Minimal spec draft: \`${PRE_PLANNING_DIR_REL}/minimal_spec_draft.md\`
- CI checkpoint plan: \`${PRE_PLANNING_DIR_REL}/ci_checkpoint_plan.md\`
- Workstream triage: \`${PRE_PLANNING_DIR_REL}/workstream_triage.md\`

## Follow-ups

1. Inspect \`${alignment_report_stderr_rel}\` and repair the failing reporter path.
2. Re-run alignment reporting after the failing reporter path is fixed.
EOF
            append_summary "## Alignment triage (wrapper-compiled)"
            append_summary ""
            append_summary "- Fallback report: \`${alignment_report_rel}\`"
            append_summary "- Tracked: \`${tracked_alignment_rel}\`"
            append_summary "- Reporter stderr: \`${alignment_report_stderr_rel}\`"
            append_summary ""
            cat "${alignment_report_abs}" >>"${SUMMARY_PATH}"
            append_summary ""

            if [[ "${rc}" -eq 0 ]]; then
                cp "${alignment_report_abs}" "${tracked_alignment_abs}"
                if [[ -n "$(git status --porcelain=v1 -- "${tracked_alignment_rel}")" ]]; then
                    git add -- "${tracked_alignment_rel}"
                    if ! git diff --cached --quiet; then
                        if git commit -m "docs: pre-planning alignment report" >/dev/null; then
                            echo "Committed: wrapper alignment report"
                        fi
                    fi
                fi
            fi
        else
            append_summary "## Alignment triage (wrapper-compiled)"
            append_summary ""
            append_summary "- Failed to generate alignment report (see \`${alignment_report_stderr_rel}\`)"
            append_summary ""
        fi
    fi

    append_summary "## Wrapper logs"
    append_summary "- \`${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/\`"
    exit "${rc}"
}
trap cleanup_on_exit EXIT

on_interrupt() {
    echo "Interrupted; stopping in-flight runners..." >&2
    kill_downstream "${start_index}"
    exit 130
}
trap on_interrupt INT TERM

append_summary "## Launch"
if [[ "${start_index}" -gt 0 ]] && phase_b_gate_ready "${start_index}"; then
    launch_step "${start_index}" "phase_b"
else
    launch_step "${start_index}" "phase_a"
fi

append_summary ""
append_summary "## Progress + commits"

last_index="$((${#steps[@]} - 1))"
launched_upto="${start_index}"
next_to_commit="${start_index}"

commit_done=()

all_steps_done() {
    local i
    for ((i = start_index; i <= last_index; i++)); do
        if [[ "${commit_done[i]:-0}" != "1" ]]; then
            return 1
        fi
    done
    return 0
}

# Event loop:
# - Launch next steps when upstream emits handoff.md (or exits successfully).
# - Commit allowlisted outputs as soon as each step exits successfully.
while true; do
    # 0) Record first-seen handoff timestamps for accurate timing even if handoff.md is overwritten later.
    for ((i = start_index; i <= launched_upto; i++)); do
        if [[ -n "${handoff_seen_epoch[i]:-}" ]]; then
            continue
        fi
        handoff_step="${steps[$i]}"
        handoff_path="$(handoff_path_for "${handoff_step}")"
        if [[ -f "${handoff_path}" ]]; then
            handoff_seen_epoch[i]="$(file_mtime_epoch "${handoff_path}" || true)"
            if [[ -z "${handoff_seen_epoch[i]}" ]]; then
                handoff_seen_epoch[i]="$(date -u +%s)"
            fi
        fi
    done

    # 1) Detect completed steps (for any launched step).
    for ((i = start_index; i <= launched_upto; i++)); do
        if [[ -n "${runner_rcs[i]:-}" ]]; then
            continue
        fi
        pid="${runner_pids[i]:-}"
        [[ -n "${pid}" ]] || continue

        if ! kill -0 "${pid}" 2>/dev/null; then
            set +e
            wait "${pid}"
            rc="$?"
            set -e
            runner_end_epoch[i]="$(date -u +%s)"

            phase="${step_phase[i]:-single}"
            step="${steps[$i]}"

            if [[ "${phase}" == "phase_a" ]]; then
                if [[ "${rc}" -eq 0 ]] && step_staged_required_outputs_exist "${step}"; then
                    runner_rcs[i]="${rc}"
                    step_state[i]="ready_to_commit"
                    step_ready_for_commit[i]="1"
                    echo "Completed: ${step} [phase_a] (staged outputs ready)"
                elif phase_a_can_advance_to_phase_b "${i}"; then
                    runner_rcs[i]="0"
                    step_state[i]="waiting_phase_b"
                    step_ready_for_commit[i]="0"
                    step_wait_reason[i]="$(phase_b_gate_reason "${i}")"
                    if [[ -n "${step_wait_reason[i]}" ]]; then
                        echo "Completed: ${step} [phase_a] (waiting for phase_b gate)"
                        append_summary "- Waiting: \`${step}\` phase_b gate pending — ${step_wait_reason[i]}"
                    else
                        echo "Completed: ${step} [phase_a] (ready for phase_b)"
                        append_summary "- Ready: \`${step}\` phase_b gate cleared after phase_a"
                    fi
                else
                    if retry_step_if_transient "${i}" "${phase}"; then
                        continue
                    fi
                    runner_rcs[i]="${rc}"
                    echo "FAILED: ${step} [${phase}] exited with ${rc} (see ${FEATURE_DIR_REL}/logs/${step}/stderr.log)" >&2
                    append_summary "- FAILED: \`${step}\` \`${phase}\` exited with \`${rc}\` — stopping"
                    kill_downstream "$((i + 1))"
                    exit "${rc}"
                fi
            else
                runner_rcs[i]="${rc}"
                if [[ "${rc}" -ne 0 ]]; then
                    if retry_step_if_transient "${i}" "${phase}"; then
                        continue
                    fi
                    echo "FAILED: ${step} [${phase}] exited with ${rc} (see ${FEATURE_DIR_REL}/logs/${step}/stderr.log)" >&2
                    append_summary "- FAILED: \`${step}\` \`${phase}\` exited with \`${rc}\` — stopping"
                    kill_downstream "$((i + 1))"
                    exit "${rc}"
                fi
                if ! step_staged_required_outputs_exist "${step}"; then
                    echo "FAILED: ${step} [${phase}] exited 0 but staged outputs are missing" >&2
                    append_summary "- FAILED: \`${step}\` \`${phase}\` exited with \`0\` but staged outputs are missing — stopping"
                    kill_downstream "$((i + 1))"
                    exit 2
                fi
                step_state[i]="ready_to_commit"
                step_ready_for_commit[i]="1"
                echo "Completed: ${step} [${phase}] (rc=0)"
            fi
        fi
    done

    # 2) Launch downstream phase_a steps as soon as their upstream handoff is available.
    #
    # Launch rules (overlap triggers):
    # - impact-map launches on spec-manifest handoff/exit.
    # - min-spec-draft launches on impact-map handoff/exit.
    # - CI-checkpoint launches on min-spec-draft handoff/exit.
    # - workstream-triage launches on min-spec-draft handoff/exit (so triage can draft while CI-checkpoint is running).
    idx_spec_manifest="$(step_index_of spec-manifest 2>/dev/null || true)"
    idx_impact_map="$(step_index_of impact-map 2>/dev/null || true)"
    idx_min_spec_draft="$(step_index_of min-spec-draft 2>/dev/null || true)"
    idx_ci_checkpoint="$(step_index_of CI-checkpoint 2>/dev/null || true)"

    for ((next_idx = start_index + 1; next_idx <= last_index; next_idx++)); do
        if [[ -n "${runner_pids[next_idx]:-}" ]] || [[ -n "${step_state[next_idx]:-}" ]]; then
            continue
        fi

        dep_idx=""
        case "${steps[$next_idx]}" in
            impact-map)
                dep_idx="${idx_spec_manifest}"
                ;;
            min-spec-draft)
                dep_idx="${idx_impact_map}"
                ;;
            CI-checkpoint)
                dep_idx="${idx_min_spec_draft}"
                ;;
            workstream-triage)
                dep_idx="${idx_min_spec_draft}"
                ;;
            *)
                dep_idx=""
                ;;
        esac

        if [[ -z "${dep_idx}" ]] || [[ ! "${dep_idx}" =~ ^[0-9]+$ ]]; then
            continue
        fi

        ready=0
        if [[ "${steps[$next_idx]}" == "workstream-triage" ]] && [[ -n "${idx_ci_checkpoint}" ]] && [[ "${idx_ci_checkpoint}" =~ ^[0-9]+$ ]]; then
            # Prefer min-spec-draft handoff/exit, but fall back to CI-checkpoint handoff/exit if min-spec-draft handoff is missing
            # (e.g., when resuming at START_AT=CI-checkpoint with archived/cleaned upstream logs).
            min_handoff="$(handoff_path_for "${steps[$idx_min_spec_draft]}")"
            min_rc="${runner_rcs[idx_min_spec_draft]:-}"
            ci_handoff="$(handoff_path_for "${steps[$idx_ci_checkpoint]}")"
            ci_rc="${runner_rcs[idx_ci_checkpoint]:-}"
            if [[ -f "${min_handoff}" ]] || [[ "${min_rc}" = "0" ]] || [[ -f "${ci_handoff}" ]] || [[ "${ci_rc}" = "0" ]]; then
                ready=1
            fi
        else
            dep_step="${steps[$dep_idx]}"
            dep_handoff="$(handoff_path_for "${dep_step}")"
            dep_rc="${runner_rcs[dep_idx]:-}"
            if [[ -f "${dep_handoff}" ]] || [[ "${dep_rc}" = "0" ]]; then
                ready=1
            fi
        fi

        if [[ "${ready}" -eq 1 ]]; then
            launch_step "${next_idx}" "phase_a"
            if [[ "${next_idx}" -gt "${launched_upto}" ]]; then
                launched_upto="${next_idx}"
            fi
        fi
    done

    # 3) Launch phase_b once the host verifies canonical prerequisites + clean feature-dir status.
    for ((i = start_index; i <= launched_upto; i++)); do
        if [[ "${step_state[i]:-}" != "waiting_phase_b" ]]; then
            continue
        fi
        if phase_b_gate_ready "${i}"; then
            launch_step "${i}" "phase_b"
        else
            step_wait_reason[i]="$(phase_b_gate_reason "${i}")"
            echo "Waiting: ${steps[$i]} [phase_b gate] ${step_wait_reason[i]}"
            append_summary "- Waiting: \`${steps[$i]}\` phase_b gate pending — ${step_wait_reason[i]}"
        fi
    done

    # 4) Commit allowlisted outputs in step order as soon as each step is ready.
    while [[ "${next_to_commit}" -le "${last_index}" ]]; do
        if [[ "${step_ready_for_commit[next_to_commit]:-0}" != "1" ]]; then
            break
        fi
        if [[ "${commit_done[next_to_commit]:-0}" = "1" ]]; then
            next_to_commit="$((next_to_commit + 1))"
            continue
        fi

        commit_step_outputs "${next_to_commit}"
        commit_done[next_to_commit]=1
        step_state[next_to_commit]="committed"
        if [[ -n "${commit_shas[next_to_commit]:-}" ]]; then
            append_summary "- Committed \`${steps[$next_to_commit]}\`: \`${commit_shas[next_to_commit]}\`"
            echo "Committed: ${steps[$next_to_commit]} (${commit_shas[next_to_commit]})"
        else
            append_summary "- No tracked changes to commit for \`${steps[$next_to_commit]}\`"
            echo "No tracked changes to commit for ${steps[$next_to_commit]}"
        fi
        next_to_commit="$((next_to_commit + 1))"
    done

    # 5) Exit once all steps are commit-processed.
    if [[ "${launched_upto}" -eq "${last_index}" ]] && all_steps_done; then
        exit 0
    fi

    sleep "${POLL_S}"
done
