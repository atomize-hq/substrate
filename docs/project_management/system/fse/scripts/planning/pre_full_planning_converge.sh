#!/usr/bin/env bash
set -euo pipefail

# Legacy compatibility wrapper retained for older callers.
# Inactive for the active FSE pre-planning lane.

usage() {
    cat <<'USAGE'
Legacy compatibility utility. Inactive for the active FSE pre-planning lane.

Usage:
  pre_full_planning_converge.sh --feature-dir <path> [--workstream-triage <path>] [--codex-profile <p>] [--codex-model <m>] [--codex-jsonl]

Required:
  --feature-dir <path>        Legacy Planning Pack dir (docs/project_management/packs/<bucket>/<feature>)

Options:
  --workstream-triage <path>  Legacy path to workstream_triage.md (absolute or feature-dir-relative).
                              Default: pre-planning/workstream_triage.md (legacy fallback: workstream_triage.md)
  --codex-profile <p>         Forwarded to run_planning_agent.sh for legacy remediation runs.
  --codex-model <m>           Forwarded to run_planning_agent.sh for legacy remediation runs.
  --codex-jsonl               Forwarded to run_planning_agent.sh for legacy remediation runs.
  -h, --help                  Show this help.

Behavior:
  - Retained for legacy callers; not used by the active FSE pre-planning lane.
  - Requires a clean checkout at start.
  - Validates downstream slice coherence against the accepted slice order recorded in legacy workstream triage.
  - If safe slice drift is detected, runs the constrained slice-reconcile agent.
  - Regenerates pre-planning/alignment_report.md before successful exit.
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
WORKSTREAM_TRIAGE_REL="pre-planning/workstream_triage.md"
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR_RAW="${2:-}"
            shift 2
            ;;
        --workstream-triage)
            WORKSTREAM_TRIAGE_REL="${2:-}"
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
need_cmd python3
need_cmd jq

SKIP_CLEAN_CHECK="${PM_PRE_FULL_PLANNING_SKIP_CLEAN_CHECK:-0}"

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

if [[ "${SKIP_CLEAN_CHECK}" != "1" && -n "$(git status --porcelain=v1)" ]]; then
    die "convergence checkout is dirty; commit or stash before running"
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system/fse}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
HELPER="${PLANNING_SCRIPTS_DIR}/pre_full_planning_convergence.py"
ALIGNMENT_REPORTER="${PM_PRE_FULL_PLANNING_ALIGNMENT_REPORTER:-${PLANNING_SCRIPTS_DIR}/wrapper_alignment_report.py}"
AGENT_RUNNER="${PM_PRE_FULL_PLANNING_AGENT_RUNNER:-${PLANNING_SCRIPTS_DIR}/run_planning_agent.sh}"
MAX_ATTEMPTS="${PM_PRE_FULL_PLANNING_MAX_ATTEMPTS:-2}"
SKIP_COMMIT="${PM_PRE_FULL_PLANNING_SKIP_COMMIT:-0}"

[[ -f "${HELPER}" ]] || die "missing convergence helper: ${HELPER}"
[[ -f "${ALIGNMENT_REPORTER}" ]] || die "missing alignment reporter: ${ALIGNMENT_REPORTER}"
[[ -x "${AGENT_RUNNER}" ]] || [[ -f "${AGENT_RUNNER}" ]] || die "missing agent runner: ${AGENT_RUNNER}"

if ! [[ "${MAX_ATTEMPTS}" =~ ^[0-9]+$ ]]; then
    die "PM_PRE_FULL_PLANNING_MAX_ATTEMPTS must be an integer (got ${MAX_ATTEMPTS})"
fi
if [[ "${SKIP_COMMIT}" != "0" && "${SKIP_COMMIT}" != "1" ]]; then
    die "PM_PRE_FULL_PLANNING_SKIP_COMMIT must be 0 or 1 (got ${SKIP_COMMIT})"
fi
if [[ "${SKIP_CLEAN_CHECK}" != "0" && "${SKIP_CLEAN_CHECK}" != "1" ]]; then
    die "PM_PRE_FULL_PLANNING_SKIP_CLEAN_CHECK must be 0 or 1 (got ${SKIP_CLEAN_CHECK})"
fi

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"

PRE_PLANNING_DIR_REL="${FEATURE_DIR_REL}/pre-planning"
PRE_PLANNING_DIR_ABS="${FEATURE_DIR_ABS}/pre-planning"
mkdir -p "${PRE_PLANNING_DIR_ABS}"

RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
LOG_DIR_ABS="${FEATURE_DIR_ABS}/logs/pre-full-planning-convergence"
RUN_DIR_ABS="${LOG_DIR_ABS}/${RUN_TS}"
mkdir -p "${RUN_DIR_ABS}"
SUMMARY_PATH="${RUN_DIR_ABS}/summary.md"
STABLE_INPUT_JSON="${LOG_DIR_ABS}/remediation_input.json"
ALIGNMENT_TMP="${RUN_DIR_ABS}/alignment_report.md"
CLASSIFY_TMP="${RUN_DIR_ABS}/classification.json"

append_summary() {
    printf '%s\n' "$*" >>"${SUMMARY_PATH}"
}

append_summary "# Pre-Full-Planning Convergence Summary (legacy compatibility utility)"
append_summary ""
append_summary "- Feature dir: \`${FEATURE_DIR_REL}/\`"
append_summary "- Run (UTC): \`${RUN_TS}\`"
append_summary "- Workstream triage: \`${WORKSTREAM_TRIAGE_REL}\`"
append_summary ""

classify() {
    python3 "${HELPER}" --feature-dir "${FEATURE_DIR_ABS}" --workstream-triage "${WORKSTREAM_TRIAGE_REL}"
}

sync_alignment_report() {
    local tracked_rel="${PRE_PLANNING_DIR_REL}/alignment_report.md"
    local tracked_abs="${PRE_PLANNING_DIR_ABS}/alignment_report.md"
    if python3 "${ALIGNMENT_REPORTER}" --feature-dir "${FEATURE_DIR_REL}" >"${ALIGNMENT_TMP}" 2>"${RUN_DIR_ABS}/alignment_report.stderr.log"; then
        if [[ ! -f "${tracked_abs}" ]] || ! cmp -s "${ALIGNMENT_TMP}" "${tracked_abs}"; then
            cp "${ALIGNMENT_TMP}" "${tracked_abs}"
            append_summary "- Regenerated: \`${tracked_rel}\`"
        else
            append_summary "- Alignment report already current: \`${tracked_rel}\`"
        fi
    else
        die "failed to regenerate alignment report (see ${FEATURE_DIR_REL}/logs/pre-full-planning-convergence/${RUN_TS}/alignment_report.stderr.log)"
    fi
}

commit_allowed_outputs() {
    local msg="$1"
    local -a allow=(
        "${PRE_PLANNING_DIR_REL}/spec_manifest.md"
        "${PRE_PLANNING_DIR_REL}/impact_map.md"
        "${PRE_PLANNING_DIR_REL}/ci_checkpoint_plan.md"
        "${PRE_PLANNING_DIR_REL}/alignment_report.md"
    )

    git add -- "${allow[@]}" >/dev/null 2>&1 || true
    if git diff --cached --quiet; then
        append_summary "- No tracked convergence changes to commit"
        return 0
    fi

    while IFS= read -r staged; do
        [[ -n "${staged}" ]] || continue
        local ok=0
        local allowed
        for allowed in "${allow[@]}"; do
            if [[ "${staged}" == "${allowed}" ]]; then
                ok=1
                break
            fi
        done
        if [[ "${ok}" -eq 0 ]]; then
            die "refusing to commit non-allowlisted convergence path: ${staged}"
        fi
    done < <(git diff --cached --name-only | sed '/^$/d')

    if [[ "${SKIP_COMMIT}" == "1" ]]; then
        git reset --quiet -- "${allow[@]}" >/dev/null 2>&1 || true
        append_summary "- PM_PRE_FULL_PLANNING_SKIP_COMMIT=1; skipped commit after staging allowlisted outputs"
        return 0
    fi

    git commit -m "${msg}" >/dev/null
    append_summary "- Committed: \`$(git rev-parse HEAD)\`"
}

run_reconcile_agent() {
    local -a args=("${AGENT_RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --agent pre_planning_slice_reconcile)
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi
    "${args[@]}"
}

attempt=0
while true; do
    classify >"${CLASSIFY_TMP}"
    cp "${CLASSIFY_TMP}" "${RUN_DIR_ABS}/classification.attempt${attempt}.json"
    cp "${CLASSIFY_TMP}" "${STABLE_INPUT_JSON}"

    status="$(jq -r '.status' "${CLASSIFY_TMP}")"
    append_summary "## Attempt ${attempt}"
    append_summary "- Classification: \`${status}\`"
    append_summary "- Accepted slice order: \`$(jq -c '.accepted_slice_order' "${CLASSIFY_TMP}")\`"

    if [[ "${status}" == "hard_fail" ]]; then
        append_summary ""
        append_summary "Issues:"
        jq -r '.issues[] | "- \(.source): \(.message)"' "${CLASSIFY_TMP}" >>"${SUMMARY_PATH}" || true
        exit 1
    fi

    if [[ "${status}" == "pass" ]]; then
        sync_alignment_report
        commit_allowed_outputs "docs: pre-full-planning convergence"
        append_summary ""
        append_summary "OK: convergence passed"
        echo "OK: pre-full-planning convergence passed"
        echo "Summary: ${FEATURE_DIR_REL}/logs/pre-full-planning-convergence/${RUN_TS}/summary.md"
        exit 0
    fi

    if [[ "${attempt}" -ge "${MAX_ATTEMPTS}" ]]; then
        append_summary ""
        append_summary "FAIL: remediation attempts exhausted"
        jq -r '.issues[] | "- \(.source): \(.message)"' "${CLASSIFY_TMP}" >>"${SUMMARY_PATH}" || true
        exit 1
    fi

    append_summary "- Stale docs: \`$(jq -c '.stale_docs' "${CLASSIFY_TMP}")\`"
    append_summary "- Running: \`pre_planning_slice_reconcile\`"
    run_reconcile_agent
    sync_alignment_report
    commit_allowed_outputs "docs: pre-full-planning convergence"
    attempt=$((attempt + 1))
done
