#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  make planning-micro-lint FEATURE_DIR=docs/project_management/packs/<bucket>/<feature> [AGENT=<id>] OWNED_PATHS="path1 path2 ..."

Notes:
  - This is a scoped, closeout lint intended for planning agents.
  - It runs the hard-ban scan AND the ambiguity scan ONLY on the provided paths.
  - For selected agents, it also runs the same structural validator the runner enforces.
  - Paths may be pack-relative (e.g., "contract.md"), feature-dir-prefixed, repo-relative, or absolute.
USAGE
}

FEATURE_DIR=""
AGENT="${AGENT:-}"
paths=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --agent)
            AGENT="${2:-}"
            shift 2
            ;;
        --)
            shift
            paths+=("$@")
            break
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            # Treat non-flag args as paths (fallback, allows calling without `--`).
            paths+=("$1")
            shift
            ;;
    esac
done

if [[ -z "${FEATURE_DIR}" ]]; then
    echo "Missing --feature-dir" >&2
    usage >&2
    exit 2
fi

if [[ "${#paths[@]}" -eq 0 ]]; then
    echo "Missing OWNED_PATHS (pass via: ... -- <OWNED_PATHS...>)" >&2
    usage >&2
    exit 2
fi

if ! command -v git >/dev/null 2>&1; then
    echo "FAIL: git is required for planning micro-lint" >&2
    exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null)" || {
    echo "FAIL: failed to locate repo root via git" >&2
    exit 2
}

cd "${REPO_ROOT}"

if [[ ! -d "${FEATURE_DIR}" ]]; then
    echo "Feature dir does not exist: ${FEATURE_DIR}" >&2
    exit 2
fi

if ! command -v rg >/dev/null 2>&1; then
    echo "FAIL: ripgrep (rg) is required for planning micro-lint (install ripgrep and retry)" >&2
    exit 2
fi

resolve_path() {
    local p="$1"
    local feature_prefix="${FEATURE_DIR%/}/"

    if [[ "${p}" == /* ]]; then
        echo "${p}"
        return 0
    fi

    if [[ "${p}" == "${feature_prefix}"* ]]; then
        echo "${p}"
        return 0
    fi

    if [[ "${p}" == docs/project_management/packs/* ]]; then
        echo "${p}"
        return 0
    fi

    echo "${feature_prefix}${p}"
}

resolved=()
missing=0
for p in "${paths[@]}"; do
    rp="$(resolve_path "${p}")"
    resolved+=("${rp}")
    if [[ ! -e "${rp}" ]]; then
        echo "Missing path for micro-lint scan: ${p} (resolved to ${rp})" >&2
        missing=1
    fi
done
if [[ "${missing}" -ne 0 ]]; then
    exit 2
fi

echo "== Planning micro-lint: ${FEATURE_DIR} =="
echo "-- Scope:"
for rp in "${resolved[@]}"; do
    echo "  - ${rp}"
done
if [[ -n "${AGENT}" ]]; then
    echo "-- Agent: ${AGENT}"
fi

PLANNING_SCRIPTS_DIR="${SCRIPT_DIR}"

slice_specs=()
for rp in "${resolved[@]}"; do
    if [[ -f "${rp}" ]]; then
        if [[ "${rp}" == */slices/*/*-spec.md ]]; then
            slice_specs+=("${rp}")
        fi
        continue
    fi
    if [[ -d "${rp}" ]]; then
        while IFS= read -r -d '' f; do
            [[ -n "${f}" ]] || continue
            slice_specs+=("${f}")
        done < <(find "${rp}" -type f -name '*-spec.md' -path '*/slices/*/*-spec.md' -print0 2>/dev/null || true)
    fi
done

if [[ "${#slice_specs[@]}" -gt 0 ]]; then
    if ! command -v python3 >/dev/null 2>&1; then
        echo "FAIL: python3 is required for slice spec structural checks (planning micro-lint)" >&2
        exit 2
    fi
    echo "-- Slice spec structural checks"
    python3 "${PLANNING_SCRIPTS_DIR}/validate_slice_spec_doc_only.py" --paths "${slice_specs[@]}"
fi

run_rg_fail_on_match() {
    local label="$1"
    local pattern="$2"
    shift 2

    echo "-- ${label}"
    if rg -n --hidden --glob '!**/.git/**' "$@" "${pattern}" "${resolved[@]}"; then
        echo "FAIL: ${label} matches found" >&2
        exit 1
    else
        rc=$?
        if [[ "${rc}" -ne 1 ]]; then
            echo "FAIL: rg failed during ${label} scan (exit=${rc})" >&2
            exit "${rc}"
        fi
    fi
}

run_rg_fail_on_match "Hard-ban scan" '\b(TBD|TODO|WIP|TBA)\b|open question|\betc\.|and so on'

run_rg_fail_on_match "Ambiguity scan" '\b(should|could|might|maybe)\b' \
    --glob '!**/decision_register.md' \
    --glob '!**/session_log.md' \
    --glob '!**/quality_gate_report.md' \
    --glob '!**/final_alignment_report.md'

run_agent_structural_validation() {
    case "${AGENT}" in
        impact_map)
            echo "-- Structural validation (impact_map)"
            if [[ "${PM_SKIP_IMPACT_MAP_VALIDATE:-0}" = "1" ]]; then
                echo "WARN: PM_SKIP_IMPACT_MAP_VALIDATE=1; skipping impact_map Touch Set validation for ${FEATURE_DIR}" >&2
                return 0
            fi
            python3 "${PLANNING_SCRIPTS_DIR}/validate_impact_map.py" --feature-dir "${FEATURE_DIR}"
            ;;
        workstream_triage)
            echo "-- Structural validation (workstream_triage)"
            if [[ "${PM_SKIP_PWS_INDEX_VALIDATE:-0}" = "1" ]]; then
                echo "WARN: PM_SKIP_PWS_INDEX_VALIDATE=1; skipping FSE workstream index validation for ${FEATURE_DIR}" >&2
                return 0
            fi
            python3 "${PLANNING_SCRIPTS_DIR}/validate_pws_index.py" --feature-dir "${FEATURE_DIR}" --advisory
            ;;
        *)
            ;;
    esac
}

run_agent_structural_validation

echo "OK: planning micro-lint passed"
