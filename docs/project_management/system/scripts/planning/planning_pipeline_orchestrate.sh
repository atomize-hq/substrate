#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  planning_pipeline_orchestrate.sh --feature-dir <path> [--start-at <step>] [--poll-s <seconds>] [--codex-profile <p>] [--codex-model <m>] [--codex-jsonl]

Required:
  --feature-dir <path>        Planning Pack dir (docs/project_management/packs/<bucket>/<feature>)

Options:
  --start-at <step>           Forwarded to pre_planning_research_orchestrate.sh.
  --poll-s <seconds>          Forwarded to pre_planning_research_orchestrate.sh.
  --codex-profile <p>         Forwarded to downstream orchestrators.
  --codex-model <m>           Forwarded to downstream orchestrators.
  --codex-jsonl               Forwarded to downstream orchestrators.
  -h, --help                  Show this help.

Behavior:
  - Runs pre_planning_research_orchestrate.sh
  - Runs pre_full_planning_converge.sh
  - Runs full_planning_orchestrate.sh
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
START_AT=""
POLL_S=""
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0

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
            shift
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

need_cmd git

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
PRE_PLANNING_ORCHESTRATOR="${PM_PRE_PLANNING_ORCHESTRATOR:-${PLANNING_SCRIPTS_DIR}/pre_planning_research_orchestrate.sh}"
PRE_FULL_PLANNING_CONVERGE="${PM_PRE_FULL_PLANNING_CONVERGE_SCRIPT:-${PLANNING_SCRIPTS_DIR}/pre_full_planning_converge.sh}"
FULL_PLANNING_ORCHESTRATOR="${PM_FULL_PLANNING_ORCHESTRATOR:-${PLANNING_SCRIPTS_DIR}/full_planning_orchestrate.sh}"

for script_path in "${PRE_PLANNING_ORCHESTRATOR}" "${PRE_FULL_PLANNING_CONVERGE}" "${FULL_PLANNING_ORCHESTRATOR}"; do
    if [[ "${script_path}" != /* ]]; then
        script_path="${REPO_ROOT}/${script_path}"
    fi
    [[ -x "${script_path}" ]] || [[ -f "${script_path}" ]] || die "missing pipeline script: ${script_path}"
done

run_pre_planning() {
    local script_path="${PRE_PLANNING_ORCHESTRATOR}"
    if [[ "${script_path}" != /* ]]; then
        script_path="${REPO_ROOT}/${script_path}"
    fi
    local -a args=("${script_path}" --feature-dir "${FEATURE_DIR_RAW}")
    if [[ -n "${START_AT}" ]]; then args+=(--start-at "${START_AT}"); fi
    if [[ -n "${POLL_S}" ]]; then args+=(--poll-s "${POLL_S}"); fi
    CODEX_PROFILE="${CODEX_PROFILE}" CODEX_MODEL="${CODEX_MODEL}" CODEX_JSONL="${CODEX_JSONL}" "${args[@]}"
}

run_convergence() {
    local script_path="${PRE_FULL_PLANNING_CONVERGE}"
    if [[ "${script_path}" != /* ]]; then
        script_path="${REPO_ROOT}/${script_path}"
    fi
    local -a args=("${script_path}" --feature-dir "${FEATURE_DIR_RAW}")
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi
    "${args[@]}"
}

run_full_planning() {
    local script_path="${FULL_PLANNING_ORCHESTRATOR}"
    if [[ "${script_path}" != /* ]]; then
        script_path="${REPO_ROOT}/${script_path}"
    fi
    local -a args=("${script_path}" --feature-dir "${FEATURE_DIR_RAW}")
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi
    "${args[@]}"
}

run_pre_planning
run_convergence
run_full_planning
