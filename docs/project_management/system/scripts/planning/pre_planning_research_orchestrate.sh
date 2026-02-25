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
  --poll-s <seconds>     Poll interval for handoff/exit checks (default: 60)

Environment (optional; passed through to the planning runner):
  CODEX_PROFILE=<p>
  CODEX_MODEL=<m>
  CODEX_JSONL=1

Behavior:
  - Requires a clean orchestration checkout (git status must be empty).
  - Archives existing step log dirs for START_AT and downstream: <step> -> <step>_run_N
  - Launches the 5-step chain with staggered overlap, triggered by upstream handoff.md.
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
POLL_S="60"

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

need_cmd git
need_cmd python3
need_cmd jq
need_cmd codex

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

if [[ -n "$(git status --porcelain=v1)" ]]; then
    die "orchestration checkout is dirty; commit or stash before running"
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
RUNNER="${PLANNING_SCRIPTS_DIR}/run_planning_agent.sh"
[[ -x "${RUNNER}" ]] || die "missing runner: ${RUNNER}"

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"
[[ -f "${FEATURE_DIR_ABS}/tasks.json" ]] || die "missing required tasks.json: ${FEATURE_DIR_REL}/tasks.json"

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
append_summary "- Run (UTC): \`${RUN_TS}\`"
append_summary "- Start at: \`${START_AT}\`"
append_summary "- Poll interval: \`${POLL_S}s\`"
append_summary ""

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

launch_step() {
    local idx="$1"
    local step="${steps[$idx]}"
    local agent="${agents[$idx]}"

    local log_path="${WRAPPER_DIR}/${step}.runner.log"
    # Launch the runner in the background; runner manages Codex child + codex.pid/stderr.log.
    local -a args=("${RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --agent "${agent}")
    if [[ -n "${CODEX_PROFILE:-}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL:-}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL:-0}" = "1" ]]; then args+=(--codex-jsonl); fi
    "${args[@]}" >"${log_path}" 2>&1 &
    local pid="$!"
    runner_pids[$idx]="${pid}"
    runner_rcs[$idx]=""
    commit_shas[$idx]=""
    append_summary "- Started: \`${step}\` (agent=\`${agent}\`, pid=\`${pid}\`, runner_log=\`$(basename "${log_path}")\`)"
}

kill_downstream() {
    local from_idx="$1"
    local j
    for j in "${!steps[@]}"; do
        if [[ "${j}" -lt "${from_idx}" ]]; then
            continue
        fi
        local pid="${runner_pids[$j]:-}"
        [[ -n "${pid}" ]] || continue
        if kill -0 "${pid}" 2>/dev/null; then
            kill "${pid}" >/dev/null 2>&1 || true
        fi
    done
}

wait_for_handoff_or_exit_success() {
    local idx="$1"
    local step="${steps[$idx]}"
    local pid="${runner_pids[$idx]}"
    local handoff="${LOGS_DIR}/${step}/handoff.md"

    while true; do
        if [[ -f "${handoff}" ]]; then
            return 0
        fi

        if ! kill -0 "${pid}" 2>/dev/null; then
            set +e
            wait "${pid}"
            rc="$?"
            set -e
            runner_rcs[$idx]="${rc}"
            return "${rc}"
        fi

        sleep "${POLL_S}"
    done
}

commit_step_outputs() {
    local idx="$1"
    local step="${steps[$idx]}"
    local msg=""
    local -a allow=()

    case "${step}" in
        spec-manifest)
            msg="docs: pre-planning spec manifest"
            allow=("${FEATURE_DIR_REL}/spec_manifest.md")
            ;;
        impact-map)
            msg="docs: pre-planning impact map"
            allow=("${FEATURE_DIR_REL}/impact_map.md")
            ;;
        min-spec-draft)
            msg="docs: pre-planning minimal spec draft"
            allow=("${FEATURE_DIR_REL}/minimal_spec_draft.md")
            ;;
        CI-checkpoint)
            msg="docs: pre-planning CI checkpoint plan"
            allow=("${FEATURE_DIR_REL}/ci_checkpoint_plan.md" "${FEATURE_DIR_REL}/tasks.json")
            ;;
        workstream-triage)
            # logs only
            return 0
            ;;
        *)
            die "unknown step: ${step}"
            ;;
    esac

    # Safety: refuse to commit if any unexpected tracked changes exist under the feature dir.
    local -a changed=()
    mapfile -t changed < <(git diff --name-only -- "${FEATURE_DIR_REL}" | sed '/^$/d')
    declare -A allow_set=()
    for p in "${allow[@]}"; do allow_set["${p}"]=1; done
    for p in "${changed[@]}"; do
        if [[ -z "${allow_set[${p}]+x}" ]]; then
            die "refusing to commit: unexpected tracked change within feature dir: ${p} (step=${step})"
        fi
    done

    # Stage allowlisted paths only.
    git add -- "${allow[@]}" >/dev/null 2>&1 || true
    if git diff --cached --quiet; then
        return 0
    fi

    # Safety: ensure we are only committing allowlisted paths.
    local -a staged=()
    mapfile -t staged < <(git diff --cached --name-only | sed '/^$/d')
    for p in "${staged[@]}"; do
        if [[ -z "${allow_set[${p}]+x}" ]]; then
            die "refusing to commit non-allowlisted path: ${p} (step=${step})"
        fi
    done

    git commit -m "${msg}" >/dev/null
    commit_shas[$idx]="$(git rev-parse HEAD)"
}

cleanup_on_exit() {
    local rc="$?"
    append_summary ""
    append_summary "## Results"
    local i
    for i in "${!steps[@]}"; do
        if [[ "${i}" -lt "${start_index}" ]]; then
            continue
        fi
        local step="${steps[$i]}"
        local agent="${agents[$i]}"
        local pid="${runner_pids[$i]:-}"
        local rcrc="${runner_rcs[$i]:-}"
        local sha="${commit_shas[$i]:-}"
        local step_dir_rel="${FEATURE_DIR_REL}/logs/${step}"
        local runner_log_rel="${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/${step}.runner.log"
        append_summary "- \`${step}\` (agent=\`${agent}\` pid=\`${pid:-}\` rc=\`${rcrc:-}\` commit=\`${sha:-}\`) — logs: \`${step_dir_rel}/\` sentinel: \`${step_dir_rel}/last_message.md\` runner_log: \`${runner_log_rel}\`"
    done
    append_summary ""
    append_summary "## Workstream triage draft"
    append_summary "- \`${FEATURE_DIR_REL}/logs/workstream-triage/workstream_triage_draft.md\`"
    append_summary ""
    append_summary "## Wrapper logs"
    append_summary "- \`${FEATURE_DIR_REL}/logs/pre_planning_wrapper/${RUN_TS}/\`"
    exit "${rc}"
}
trap cleanup_on_exit EXIT

append_summary "## Launch"

# Launch the chain with staggered overlap.
launch_step "${start_index}"

for ((i = start_index; i < ${#steps[@]} - 1; i++)); do
    # Wait for upstream handoff or upstream exit. On exit!=0, stop.
    set +e
    wait_for_handoff_or_exit_success "${i}"
    rc="$?"
    set -e

    if [[ "${rc}" -ne 0 ]]; then
        append_summary "- FAILED: \`${steps[$i]}\` exited with \`${rc}\` — stopping"
        runner_rcs[$i]="${rc}"
        kill_downstream "$((i + 1))"
        exit "${rc}"
    fi

    # Upstream is either at handoff (still running) or has already exited successfully.
    if [[ -z "${runner_pids[$((i + 1))]:-}" ]]; then
        launch_step "$((i + 1))"
    fi
done

append_summary ""
append_summary "## Completion + commits"

# Wait for each step to complete and commit allowlisted outputs.
for ((i = start_index; i < ${#steps[@]}; i++)); do
    if [[ -z "${runner_rcs[$i]:-}" ]]; then
        pid="${runner_pids[$i]}"
        set +e
        wait "${pid}"
        rc="$?"
        set -e
        runner_rcs[$i]="${rc}"
    else
        rc="${runner_rcs[$i]}"
    fi

    if [[ "${rc}" -ne 0 ]]; then
        append_summary "- FAILED: \`${steps[$i]}\` exited with \`${rc}\` — stopping"
        kill_downstream "$((i + 1))"
        exit "${rc}"
    fi

    commit_step_outputs "${i}"
    if [[ -n "${commit_shas[$i]:-}" ]]; then
        append_summary "- Committed \`${steps[$i]}\`: \`${commit_shas[$i]}\`"
    else
        append_summary "- No tracked changes to commit for \`${steps[$i]}\`"
    fi
done

exit 0
