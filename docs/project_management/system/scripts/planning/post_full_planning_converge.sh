#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  post_full_planning_converge.sh --feature-dir <path> [--codex-profile <p>] [--codex-model <m>] [--codex-jsonl]

Required:
  --feature-dir <path>        Planning Pack dir (docs/project_management/packs/<bucket>/<feature>)

Options:
  --codex-profile <p>         Forwarded to run_planning_agent.sh when remediation is needed.
  --codex-model <m>           Forwarded to run_planning_agent.sh when remediation is needed.
  --codex-jsonl               Forwarded to run_planning_agent.sh when remediation is needed.
  -h, --help                  Show this help.

Behavior:
  - Requires a clean checkout at start.
  - Validates late-pack execution readiness after full planning completes.
  - If only safe late-pack drift is detected, runs the constrained post-full reconcile agent.
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

relpath_in_repo() {
    local repo="$1"
    local raw="$2"

    python3 - "${repo}" "${raw}" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path

repo = Path(sys.argv[1]).resolve()
raw = Path(sys.argv[2]).resolve()
print(raw.relative_to(repo).as_posix())
PY
}

FEATURE_DIR_RAW=""
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR_RAW="${2:-}"
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

SKIP_CLEAN_CHECK="${PM_POST_FULL_PLANNING_SKIP_CLEAN_CHECK:-0}"

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

if [[ "${SKIP_CLEAN_CHECK}" != "1" && -n "$(git status --porcelain=v1)" ]]; then
    die "convergence checkout is dirty; commit or stash before running"
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
HELPER="${PM_POST_FULL_PLANNING_CONVERGENCE_SCRIPT:-${PLANNING_SCRIPTS_DIR}/post_full_planning_convergence.py}"
ALIGNMENT_REPORTER="${PM_POST_FULL_PLANNING_ALIGNMENT_REPORTER:-${PLANNING_SCRIPTS_DIR}/wrapper_alignment_report.py}"
AGENT_RUNNER="${PM_POST_FULL_PLANNING_AGENT_RUNNER:-${PLANNING_SCRIPTS_DIR}/run_planning_agent.sh}"
MAX_ATTEMPTS="${PM_POST_FULL_PLANNING_MAX_ATTEMPTS:-2}"
SKIP_COMMIT="${PM_POST_FULL_PLANNING_SKIP_COMMIT:-0}"

[[ -f "${HELPER}" ]] || die "missing convergence helper: ${HELPER}"
[[ -f "${ALIGNMENT_REPORTER}" ]] || die "missing alignment reporter: ${ALIGNMENT_REPORTER}"
[[ -x "${AGENT_RUNNER}" ]] || [[ -f "${AGENT_RUNNER}" ]] || die "missing agent runner: ${AGENT_RUNNER}"

if ! [[ "${MAX_ATTEMPTS}" =~ ^[0-9]+$ ]]; then
    die "PM_POST_FULL_PLANNING_MAX_ATTEMPTS must be an integer (got ${MAX_ATTEMPTS})"
fi
if [[ "${SKIP_COMMIT}" != "0" && "${SKIP_COMMIT}" != "1" ]]; then
    die "PM_POST_FULL_PLANNING_SKIP_COMMIT must be 0 or 1 (got ${SKIP_COMMIT})"
fi
if [[ "${SKIP_CLEAN_CHECK}" != "0" && "${SKIP_CLEAN_CHECK}" != "1" ]]; then
    die "PM_POST_FULL_PLANNING_SKIP_CLEAN_CHECK must be 0 or 1 (got ${SKIP_CLEAN_CHECK})"
fi

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"

PRE_PLANNING_DIR_REL="${FEATURE_DIR_REL}/pre-planning"
PRE_PLANNING_DIR_ABS="${FEATURE_DIR_ABS}/pre-planning"
mkdir -p "${PRE_PLANNING_DIR_ABS}"

RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
LOG_DIR_ABS="${FEATURE_DIR_ABS}/logs/post-full-planning-convergence"
RUN_DIR_ABS="${LOG_DIR_ABS}/${RUN_TS}"
mkdir -p "${RUN_DIR_ABS}"
SUMMARY_PATH="${RUN_DIR_ABS}/summary.md"
STABLE_INPUT_JSON="${LOG_DIR_ABS}/remediation_input.json"
ALIGNMENT_TMP="${RUN_DIR_ABS}/alignment_report.md"
CLASSIFY_TMP="${RUN_DIR_ABS}/classification.json"

append_summary() {
    printf '%s\n' "$*" >>"${SUMMARY_PATH}"
}

append_summary "# Post-Full-Planning Convergence Summary"
append_summary ""
append_summary "- Feature dir: \`${FEATURE_DIR_REL}/\`"
append_summary "- Run (UTC): \`${RUN_TS}\`"
append_summary ""

classify() {
    python3 "${HELPER}" --feature-dir "${FEATURE_DIR_ABS}"
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
        die "failed to regenerate alignment report (see ${FEATURE_DIR_REL}/logs/post-full-planning-convergence/${RUN_TS}/alignment_report.stderr.log)"
    fi
}

collect_allowed_outputs() {
    printf '%s\n' \
        "${PRE_PLANNING_DIR_REL}/impact_map.md" \
        "${PRE_PLANNING_DIR_REL}/alignment_report.md" \
        "${FEATURE_DIR_REL}/plan.md" \
        "${FEATURE_DIR_REL}/tasks.json" \
        "${FEATURE_DIR_REL}/manual_testing_playbook.md" \
        "${FEATURE_DIR_REL}/execution_preflight_report.md"

    jq -r '.tasks[]?.kickoff_prompt // empty' "${FEATURE_DIR_ABS}/tasks.json" | sed '/^$/d'
    find "${FEATURE_DIR_ABS}/slices" -type f -name '*-closeout_report.md' 2>/dev/null | while IFS= read -r path; do
        relpath_in_repo "${REPO_ROOT}" "${path}"
    done
}

commit_allowed_outputs() {
    local msg="$1"
    local -a allow=()
    while IFS= read -r path; do
        [[ -n "${path}" ]] || continue
        allow+=("${path}")
    done < <(collect_allowed_outputs)

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
        append_summary "- PM_POST_FULL_PLANNING_SKIP_COMMIT=1; skipped commit after staging allowlisted outputs"
        return 0
    fi

    git commit -m "${msg}" >/dev/null
    append_summary "- Committed: \`$(git rev-parse HEAD)\`"
}

run_reconcile_agent() {
    local -a args=("${AGENT_RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --agent post_full_planning_reconcile)
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

    if [[ "${status}" == "hard_fail" ]]; then
        append_summary ""
        append_summary "Issues:"
        jq -r '.issues[] | "- \(.validator): \(.message)"' "${CLASSIFY_TMP}" >>"${SUMMARY_PATH}" || true
        exit 1
    fi

    if [[ "${status}" == "pass" ]]; then
        sync_alignment_report
        commit_allowed_outputs "docs: post-full-planning convergence"
        append_summary ""
        append_summary "OK: convergence passed"
        echo "OK: post-full-planning convergence passed"
        echo "Summary: ${FEATURE_DIR_REL}/logs/post-full-planning-convergence/${RUN_TS}/summary.md"
        exit 0
    fi

    if [[ "${attempt}" -ge "${MAX_ATTEMPTS}" ]]; then
        append_summary ""
        append_summary "FAIL: remediation attempts exhausted"
        jq -r '.issues[] | "- \(.validator): \(.message)"' "${CLASSIFY_TMP}" >>"${SUMMARY_PATH}" || true
        exit 1
    fi

    append_summary "- Stale docs: \`$(jq -c '.stale_docs' "${CLASSIFY_TMP}")\`"
    append_summary "- Running: \`post_full_planning_reconcile\`"
    run_reconcile_agent
    sync_alignment_report
    commit_allowed_outputs "docs: post-full-planning convergence"
    attempt=$((attempt + 1))
done
