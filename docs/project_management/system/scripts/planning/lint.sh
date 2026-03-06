#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  make planning-lint FEATURE_DIR=docs/project_management/packs/<bucket>/<feature>

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

if ! command -v git >/dev/null 2>&1; then
    echo "FAIL: git is required for planning lint" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null)" || {
    echo "FAIL: failed to locate repo root via git" >&2
    exit 2
}

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi

PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"

cd "${REPO_ROOT}"

if [[ ! -d "${FEATURE_DIR}" ]]; then
    echo "Feature dir does not exist: ${FEATURE_DIR}" >&2
    exit 2
fi

echo "== Planning lint: ${FEATURE_DIR} =="

if ! command -v rg >/dev/null 2>&1; then
    echo "FAIL: ripgrep (rg) is required for planning lint (install ripgrep and retry)" >&2
    exit 2
fi

require_path() {
    local p="$1"
    if [[ ! -e "$p" ]]; then
        echo "Missing required path: $p" >&2
        exit 1
    fi
}

require_any_path() {
    local label="$1"
    shift
    local p
    for p in "$@"; do
        if [[ -e "$p" ]]; then
            return 0
        fi
    done
    echo "Missing required path (${label}); expected one of:" >&2
    for p in "$@"; do
        echo "  - $p" >&2
    done
    exit 1
}

require_path "${FEATURE_DIR}/plan.md"
require_path "${FEATURE_DIR}/tasks.json"
require_path "${FEATURE_DIR}/session_log.md"
require_path "${FEATURE_DIR}/kickoff_prompts"
require_any_path "spec_manifest.md" "${FEATURE_DIR}/pre-planning/spec_manifest.md" "${FEATURE_DIR}/spec_manifest.md"
require_any_path "impact_map.md" "${FEATURE_DIR}/pre-planning/impact_map.md" "${FEATURE_DIR}/impact_map.md"

FEATURE_DIR_RELPATH="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR}")"
pm_roots_json="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" print-roots)"
PM_ROOT="$(jq -r '.pm_root' <<<"${pm_roots_json}")"
PM_PACKS_ROOT="$(jq -r '.pm_packs_root' <<<"${pm_roots_json}")"

PM_PACKS_PREFIX="${PM_PACKS_ROOT%/}/"

schema_version="$(jq -r '.meta.schema_version // 1' "${FEATURE_DIR}/tasks.json")"
automation_enabled="$(jq -r '.meta.automation.enabled // false' "${FEATURE_DIR}/tasks.json")"
cross_platform_enabled="$(jq -r '.meta.cross_platform // false' "${FEATURE_DIR}/tasks.json")"

if [[ "${schema_version}" -ge 3 && "${automation_enabled}" == "true" && "${cross_platform_enabled}" == "true" ]]; then
    require_any_path "ci_checkpoint_plan.md" "${FEATURE_DIR}/pre-planning/ci_checkpoint_plan.md" "${FEATURE_DIR}/ci_checkpoint_plan.md"
fi

echo "-- workstream_triage PM_PWS_INDEX (advisory)"
python3 "${PLANNING_SCRIPTS_DIR}/validate_pws_index.py" --feature-dir "${FEATURE_DIR}" --advisory

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

    echo "-- Smoke script scaffold scan"
    if rg -n --hidden --glob '!**/.git/**' 'Smoke script scaffold .*replace with feature checks' "${FEATURE_DIR}/smoke"; then
        echo "FAIL: smoke scripts still contain scaffolds; replace them with contract assertions (manual_testing_playbook.md should mirror these checks)" >&2
        exit 1
    else
        rc=$?
        if [[ "${rc}" -ne 1 ]]; then
            exit "${rc}"
        fi
    fi
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

sequencing_json="${PM_PACKS_ROOT%/}/sequencing.json"
if [[ ! -f "${sequencing_json}" ]]; then
    echo "FAIL: sequencing.json missing: ${sequencing_json}" >&2
    exit 1
fi

jq -e . "${sequencing_json}" >/dev/null

echo "-- tasks.json invariants"
python3 "${PLANNING_SCRIPTS_DIR}/validate_tasks_json.py" --feature-dir "${FEATURE_DIR}"

echo "-- spec_manifest.md required-doc existence"
python3 "${PLANNING_SCRIPTS_DIR}/validate_spec_manifest.py" --feature-dir "${FEATURE_DIR}"

echo "-- slice spec invariants (gated by meta.slice_spec_version)"
python3 "${PLANNING_SCRIPTS_DIR}/validate_slice_specs.py" --feature-dir "${FEATURE_DIR}"

echo "-- impact_map.md Touch Set validation (gated by meta.slice_spec_version)"
python3 "${PLANNING_SCRIPTS_DIR}/validate_impact_map.py" --feature-dir "${FEATURE_DIR}"

echo "-- slice inventory coherence"
python3 "${PLANNING_SCRIPTS_DIR}/validate_slice_inventory_coherence.py" --feature-dir "${FEATURE_DIR}" --phase execution_ready

if [[ "${PM_LIFT_ADVISORY:-0}" = "1" ]]; then
    echo "-- Work Lift advisory report (PM_LIFT_ADVISORY=1)"
    python3 "${PLANNING_SCRIPTS_DIR}/pm_lift_report.py" --feature-dir "${FEATURE_DIR}"
fi

if [[ "${schema_version}" -ge 3 && "${automation_enabled}" == "true" && "${cross_platform_enabled}" == "true" ]]; then
    echo "-- ci_checkpoint_plan.md invariants"
    python3 "${PLANNING_SCRIPTS_DIR}/validate_ci_checkpoint_plan.py" --feature-dir "${FEATURE_DIR}"
fi

echo "-- ADR Executive Summary drift (if ADRs found/referenced)"
adr_paths=()

while IFS= read -r p; do
    [[ -n "$p" ]] && adr_paths+=("$p")
done < <(ls -1 "${FEATURE_DIR}"/ADR-*.md 2>/dev/null || true)

while IFS= read -r p; do
    [[ -n "$p" ]] && adr_paths+=("$p")
done < <(rg -o --no-filename --no-line-number --hidden --glob '!**/.git/**' 'docs/project_management/(next|adrs/[^ )"\r\n]+)/ADR-[^ )"\r\n]+\.md' "${FEATURE_DIR}" 2>/dev/null | sort -u || true)

if [[ "${#adr_paths[@]}" -gt 0 ]]; then
    for adr in "${adr_paths[@]}"; do
        if [[ -f "$adr" ]]; then
            python3 "${PLANNING_SCRIPTS_DIR}/check_adr_exec_summary.py" --adr "$adr"
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
while IFS= read -r -d '' kickoff_dir; do
    while IFS= read -r -d '' f; do
        if ! rg -q 'Do not edit planning docs inside the worktree\.' "$f"; then
            echo "Missing sentinel in kickoff prompt: $f" >&2
            missing=1
        fi
    done < <(find "${kickoff_dir}" -maxdepth 1 -type f -name '*.md' ! -name 'README.md' -print0)
done < <(find "${FEATURE_DIR}" -type d -name 'kickoff_prompts' -print0)
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
if [[ "${FEATURE_DIR_RELPATH}" == "${PM_PACKS_PREFIX}"* ]]; then
    jq -e --arg dir "${FEATURE_DIR_RELPATH}" '.sprints[] | select(.directory==$dir) | .id' "${sequencing_json}" >/dev/null
else
    echo "SKIP: sequencing alignment (feature dir not under PM roots)"
fi

echo "-- Sequencing spine validity (completed sprints)"
python3 - "${sequencing_json}" <<'PY'
import json
import sys
from pathlib import Path

path = Path(sys.argv[1])
data = json.loads(path.read_text(encoding="utf-8"))

missing = []
for sprint in data.get("sprints", []):
    if sprint.get("status") != "completed":
        continue

    sprint_id = sprint.get("id", "<missing-id>")
    for key in ("directory", "plan"):
        p = sprint.get(key)
        if p and not Path(p).exists():
            missing.append((sprint_id, key, p))

    for entry in sprint.get("sequence") or []:
        if not isinstance(entry, dict):
            continue
        spec = entry.get("spec")
        if spec and not Path(spec).exists():
            missing.append((sprint_id, "spec", spec))

if missing:
    for sprint_id, key, p in missing:
        print(f"FAIL: sequencing.json references missing path for completed sprint: {sprint_id} {key}={p}")
    raise SystemExit(1)
print("OK: completed sprint paths resolve")
PY

echo "OK: planning lint passed"
