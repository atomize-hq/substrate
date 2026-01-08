#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/planning/lint.sh --feature-dir docs/project_management/next/<feature>

Notes:
  - This is the mechanical Planning Pack lint runner used by the quality gate reviewer.
  - It is intentionally strict and exits non-zero on any violation.
USAGE
}

FEATURE_DIR=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown arg: $1" >&2
            usage >&2
            exit 2
            ;;
    esac
done

if [[ -z "${FEATURE_DIR}" ]]; then
    echo "Missing --feature-dir" >&2
    usage >&2
    exit 2
fi

if [[ ! -d "${FEATURE_DIR}" ]]; then
    echo "Feature dir does not exist: ${FEATURE_DIR}" >&2
    exit 2
fi

echo "== Planning lint: ${FEATURE_DIR} =="

require_path() {
    local p="$1"
    if [[ ! -e "$p" ]]; then
        echo "Missing required path: $p" >&2
        exit 1
    fi
}

require_path "${FEATURE_DIR}/plan.md"
require_path "${FEATURE_DIR}/tasks.json"
require_path "${FEATURE_DIR}/session_log.md"
require_path "${FEATURE_DIR}/kickoff_prompts"

if [[ -d "${FEATURE_DIR}/smoke" ]]; then
    behavior_platforms_csv="$(jq -r '[.meta.behavior_platforms_required // .meta.ci_parity_platforms_required // .meta.platforms_required // []] | flatten | join(",")' "${FEATURE_DIR}/tasks.json")"
    if [[ -z "${behavior_platforms_csv}" ]]; then
        echo "FAIL: smoke/ exists but tasks.json meta is missing behavior platform declaration (expected meta.behavior_platforms_required, or legacy meta.platforms_required)" >&2
        exit 1
    fi

    IFS=',' read -r -a behavior_platforms <<<"${behavior_platforms_csv}"
    for p in "${behavior_platforms[@]}"; do
        p="$(echo "${p}" | xargs)"
        [[ -z "${p}" ]] && continue
        case "${p}" in
            linux) require_path "${FEATURE_DIR}/smoke/linux-smoke.sh" ;;
            macos) require_path "${FEATURE_DIR}/smoke/macos-smoke.sh" ;;
            windows) require_path "${FEATURE_DIR}/smoke/windows-smoke.ps1" ;;
            *) echo "FAIL: invalid platform in behavior platforms: ${p}" >&2; exit 1 ;;
        esac
    done
fi

echo "-- Hard-ban scan"
if rg -n --hidden --glob '!**/.git/**' '\b(TBD|TODO|WIP|TBA)\b|open question|\betc\.|and so on' "${FEATURE_DIR}"; then
    echo "FAIL: hard-ban matches found (remove these from planning outputs)" >&2
    exit 1
else
    rc=$?
    if [[ "${rc}" -ne 1 ]]; then
        exit "${rc}"
    fi
fi

echo "-- Ambiguity scan"
if rg -n --hidden --glob '!**/.git/**' --glob '!**/decision_register.md' --glob '!**/session_log.md' --glob '!**/quality_gate_report.md' --glob '!**/final_alignment_report.md' '\b(should|could|might|maybe)\b' "${FEATURE_DIR}"; then
    echo "FAIL: ambiguity-word matches found (rewrite behavioral contracts to be singular/testable)" >&2
    exit 1
else
    rc=$?
    if [[ "${rc}" -ne 1 ]]; then
        exit "${rc}"
    fi
fi

echo "-- JSON validity"
jq -e . "${FEATURE_DIR}/tasks.json" >/dev/null
jq -e . docs/project_management/next/sequencing.json >/dev/null

echo "-- tasks.json invariants"
python3 scripts/planning/validate_tasks_json.py --feature-dir "${FEATURE_DIR}"

echo "-- ADR Executive Summary drift (if ADRs found/referenced)"
adr_paths=()

while IFS= read -r p; do
    [[ -n "$p" ]] && adr_paths+=("$p")
done < <(ls -1 "${FEATURE_DIR}"/ADR-*.md 2>/dev/null || true)

while IFS= read -r p; do
    [[ -n "$p" ]] && adr_paths+=("$p")
done < <(rg -o --no-filename --no-line-number --hidden --glob '!**/.git/**' 'docs/project_management/next/ADR-[^ )"\r\n]+\.md' "${FEATURE_DIR}" 2>/dev/null | sort -u || true)

if [[ "${#adr_paths[@]}" -gt 0 ]]; then
    for adr in "${adr_paths[@]}"; do
        if [[ -f "$adr" ]]; then
            python3 scripts/planning/check_adr_exec_summary.py --adr "$adr"
        else
            echo "Referenced ADR not found: $adr" >&2
            exit 1
        fi
    done
else
    echo "SKIP: no ADRs found/referenced"
fi

echo "-- Kickoff prompt sentinel"
missing=0
while IFS= read -r -d '' f; do
    if ! rg -q 'Do not edit planning docs inside the worktree\.' "$f"; then
        echo "Missing sentinel in kickoff prompt: $f" >&2
        missing=1
    fi
done < <(find "${FEATURE_DIR}/kickoff_prompts" -maxdepth 1 -type f -name '*.md' ! -name 'README.md' -print0)
if [[ "${missing}" -ne 0 ]]; then
    exit 1
fi

echo "-- Manual playbook smoke linkage (if present)"
if [[ -f "${FEATURE_DIR}/manual_testing_playbook.md" ]]; then
    if [[ -d "${FEATURE_DIR}/smoke" ]]; then
        behavior_platforms_csv="$(jq -r '[.meta.behavior_platforms_required // .meta.ci_parity_platforms_required // .meta.platforms_required // []] | flatten | join(",")' "${FEATURE_DIR}/tasks.json")"
        if [[ -z "${behavior_platforms_csv}" ]]; then
            echo "FAIL: smoke/ exists but tasks.json meta is missing behavior platform declaration (expected meta.behavior_platforms_required, or legacy meta.platforms_required)" >&2
            exit 1
        fi

        IFS=',' read -r -a behavior_platforms <<<"${behavior_platforms_csv}"
        for p in "${behavior_platforms[@]}"; do
            p="$(echo "${p}" | xargs)"
            [[ -z "${p}" ]] && continue
            case "${p}" in
                linux) smoke_ref="smoke/linux-smoke.sh" ;;
                macos) smoke_ref="smoke/macos-smoke.sh" ;;
                windows) smoke_ref="smoke/windows-smoke.ps1" ;;
                *) echo "FAIL: invalid platform in behavior platforms: ${p}" >&2; exit 1 ;;
            esac
            if ! rg -nF "${smoke_ref}" "${FEATURE_DIR}/manual_testing_playbook.md" >/dev/null; then
                echo "FAIL: manual_testing_playbook.md must reference required smoke script: ${smoke_ref}" >&2
                exit 1
            fi
        done
    fi
fi

echo "-- Sequencing alignment"
jq -e --arg dir "${FEATURE_DIR}" '.sprints[] | select(.directory==$dir) | .id' docs/project_management/next/sequencing.json >/dev/null

echo "OK: planning lint passed"
