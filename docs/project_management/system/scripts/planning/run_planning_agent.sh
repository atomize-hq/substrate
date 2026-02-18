#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'EOF'
Run a focused planning agent (single-output) via Codex.

Usage:
  run_planning_agent.sh --feature-dir <path> --agent <spec_manifest|impact_map> [options]

Required:
  --feature-dir <path>         Feature directory (relative or absolute)
  --agent <id>                 Agent id: spec_manifest | impact_map

Optional:
  --codex-profile <profile>    Passed to `codex exec --profile`
  --codex-model <model>        Passed to `codex exec --model`
  --codex-jsonl                Enable `codex exec --json` (stdout is events.jsonl)
  --help                       Show this help

Contract:
  - spec_manifest -> <FEATURE_DIR>/spec_manifest.md
  - impact_map    -> <FEATURE_DIR>/impact_map.md

Notes:
  - Uses roots from: `pm_paths.py` (sibling in this directory)
  - Enforces a single-output rule: only the intended output file within FEATURE_DIR may change.
  - Writes run artifacts under: <FEATURE_DIR>/logs/planning_agents/<AGENT>/<YYYYMMDD-HHMMSS>/
EOF
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

repo_root() {
    git -C "${SCRIPT_DIR}" rev-parse --show-toplevel 2>/dev/null
}

relpath_in_repo() {
    local repo="$1"
    local raw="$2"

    python3 - "${repo}" "${raw}" <<'PY'
from __future__ import annotations

import sys
from pathlib import Path

repo = Path(sys.argv[1]).resolve()
raw = sys.argv[2]
p = Path(raw)
if p.is_absolute():
    abs_path = p.resolve()
else:
    abs_path = (repo / p).resolve()

try:
    rel = abs_path.relative_to(repo)
except Exception:
    print(f"ERROR: path resolves outside repo root: {raw!r} -> {abs_path}", file=sys.stderr)
    raise SystemExit(2)

print(rel.as_posix())
PY
}

extract_first_md_fence_payload() {
    local prompt_file="$1"
    awk '
        BEGIN { in=0 }
        /^```md[[:space:]]*$/ { in=1; next }
        in && /^```[[:space:]]*$/ { exit }
        in { print }
    ' "${prompt_file}"
}

json_get_roots() {
    python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" print-roots
}

parse_plan_frontmatter_adr_refs() {
    local plan_file="$1"
    if [[ ! -f "${plan_file}" ]]; then
        return 0
    fi

    awk '
        NR==1 { if ($0 != "---") exit 0; in=1; next }
        in && $0=="---" { exit 0 }
        in && $0 ~ /^adr_refs:[[:space:]]*$/ { mode="adr_refs"; next }
        in && mode=="adr_refs" && $0 ~ /^[[:space:]]*-[[:space:]]*/ {
            gsub(/^[[:space:]]*-[[:space:]]*/, "", $0);
            if ($0 != "") print $0;
            next
        }
    ' "${plan_file}"
}

parse_spec_manifest_adr_paths() {
    local spec_manifest="$1"
    if [[ ! -f "${spec_manifest}" ]]; then
        return 0
    fi

    # Extract bullet items under "ADR(s):" in the Inputs section; prefer backticked paths.
    awk '
        BEGIN { in_inputs=0; in_adrs=0 }
        /^##[[:space:]]+Inputs[[:space:]]*$/ { in_inputs=1; next }
        in_inputs && /^##[[:space:]]+/ { exit }
        in_inputs && $0 ~ /^-[[:space:]]+ADR\(s\):/ { in_adrs=1; next }
        in_adrs && $0 ~ /^-[[:space:]]+/ {
            # bullet inside ADR list
            if (match($0, /`[^`]+`/)) {
                s = substr($0, RSTART+1, RLENGTH-2)
                print s
            }
            next
        }
        in_adrs && $0 !~ /^-[[:space:]]+/ { in_adrs=0 }
    ' "${spec_manifest}"
}

find_adrs_by_feature_dir_match() {
    local repo="$1"
    local feature_dir_rel="$2" # no trailing slash
    local pm_adrs_root_rel="$3"
    local pm_root_rel="$4"

    local needle="Feature directory: \`${feature_dir_rel}/\`"

    local -a search_dirs=()
    if [[ -n "${pm_adrs_root_rel}" && -d "${repo}/${pm_adrs_root_rel}" ]]; then
        search_dirs+=("${repo}/${pm_adrs_root_rel}")
    fi
    if [[ -n "${pm_root_rel}" && -d "${repo}/${pm_root_rel}/next" ]]; then
        search_dirs+=("${repo}/${pm_root_rel}/next")
    fi

    if [[ "${#search_dirs[@]}" -eq 0 ]]; then
        return 0
    fi

    if command -v rg >/dev/null 2>&1; then
        rg -l --fixed-strings "${needle}" "${search_dirs[@]}" --glob 'ADR-*.md' 2>/dev/null || true
        return 0
    fi

    # grep fallback
    grep -R -l -F "${needle}" "${search_dirs[@]}" 2>/dev/null | grep -E 'ADR-.*\.md$' || true
}

resolve_adr_ref_to_path() {
    local repo="$1"
    local adr_ref="$2"
    local pm_adrs_root_rel="$3"
    local pm_root_rel="$4"

    local -a search_dirs=()
    if [[ -n "${pm_adrs_root_rel}" && -d "${repo}/${pm_adrs_root_rel}" ]]; then
        search_dirs+=("${repo}/${pm_adrs_root_rel}")
    fi
    if [[ -n "${pm_root_rel}" && -d "${repo}/${pm_root_rel}/next" ]]; then
        search_dirs+=("${repo}/${pm_root_rel}/next")
    fi

    local -a matches=()
    local d
    for d in "${search_dirs[@]}"; do
        while IFS= read -r p; do
            [[ -n "${p}" ]] || continue
            matches+=("${p}")
        done < <(find "${d}" -type f -name "${adr_ref}*.md" 2>/dev/null || true)
    done

    if [[ "${#matches[@]}" -eq 0 ]]; then
        die "missing ADR for ref ${adr_ref} (no ${adr_ref}*.md found under ADR stores); use meta.adr_paths to specify an exact path"
    fi
    if [[ "${#matches[@]}" -gt 1 ]]; then
        echo "ERROR: ambiguous ADR ref ${adr_ref}; multiple matches found:" >&2
        local m
        for m in "${matches[@]}"; do
            echo "  - $(relpath_in_repo "${repo}" "${m}")" >&2
        done
        die "ambiguous ADR ref ${adr_ref}; use meta.adr_paths to disambiguate"
    fi

    relpath_in_repo "${repo}" "${matches[0]}"
}

FEATURE_DIR_RAW=""
AGENT=""
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR_RAW="${2:-}"
            shift 2
            ;;
        --agent)
            AGENT="${2:-}"
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
        --help|-h)
            usage
            exit 0
            ;;
        *)
            die "unknown arg: $1 (use --help)"
            ;;
    esac
done

[[ -n "${FEATURE_DIR_RAW}" ]] || die "--feature-dir is required"
[[ -n "${AGENT}" ]] || die "--agent is required"

need_cmd git
need_cmd python3
need_cmd jq
need_cmd codex

REPO_ROOT="$(repo_root)"
[[ -n "${REPO_ROOT}" ]] || die "not in a git repo/worktree (git rev-parse failed)"

cd "${REPO_ROOT}"

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist or is not a directory: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"

TASKS_JSON_ABS="${FEATURE_DIR_ABS}/tasks.json"
[[ -f "${TASKS_JSON_ABS}" ]] || die "missing required tasks.json: ${FEATURE_DIR_REL}/tasks.json"

PROMPT_FILE_REL=""
ALLOWED_OUTPUT_REL=""
case "${AGENT}" in
    spec_manifest)
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/spec_manifest_agent.md"
        ALLOWED_OUTPUT_REL="${FEATURE_DIR_REL}/spec_manifest.md"
        ;;
    impact_map)
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/impact_map_agent.md"
        ALLOWED_OUTPUT_REL="${FEATURE_DIR_REL}/impact_map.md"
        ;;
    *)
        die "unknown --agent: ${AGENT} (expected spec_manifest|impact_map)"
        ;;
esac

[[ -f "${REPO_ROOT}/${PROMPT_FILE_REL}" ]] || die "missing prompt file: ${PROMPT_FILE_REL}"

FEATURE_SLUG="$(basename "${FEATURE_DIR_REL}")"

ROOTS_JSON="$(json_get_roots)"
PM_ROOT_REL="$(printf '%s\n' "${ROOTS_JSON}" | jq -r '.pm_root')"
PM_ADRS_ROOT_REL="$(printf '%s\n' "${ROOTS_JSON}" | jq -r '.pm_adrs_root')"

SLICE_SPEC_VERSION_RAW="$(jq -r '.meta.slice_spec_version // empty' "${TASKS_JSON_ABS}" || true)"
STRICT=0
if [[ -n "${SLICE_SPEC_VERSION_RAW}" ]]; then
    if [[ "${SLICE_SPEC_VERSION_RAW}" =~ ^[0-9]+$ ]] && [[ "${SLICE_SPEC_VERSION_RAW}" -ge 2 ]]; then
        STRICT=1
    fi
fi

mapfile -t ADR_PATHS_TASKS < <(jq -r '.meta.adr_paths // [] | .[]' "${TASKS_JSON_ABS}")
mapfile -t ADR_REFS_TASKS < <(jq -r '.meta.adr_refs // [] | .[]' "${TASKS_JSON_ABS}")

resolve_adr_paths_list() {
    local -a raw_paths=("$@")
    local -a resolved=()
    local p
    for p in "${raw_paths[@]}"; do
        [[ -n "${p}" ]] || continue
        local rel
        rel="$(relpath_in_repo "${REPO_ROOT}" "${p}")"
        [[ -f "${REPO_ROOT}/${rel}" ]] || die "ADR path does not exist: ${p} (resolved to ${rel})"
        resolved+=("${rel}")
    done
    printf '%s\n' "${resolved[@]}"
}

resolve_adr_refs_list() {
    local -a refs=("$@")
    local -a resolved=()
    local r
    for r in "${refs[@]}"; do
        [[ -n "${r}" ]] || continue
        resolved+=("$(resolve_adr_ref_to_path "${REPO_ROOT}" "${r}" "${PM_ADRS_ROOT_REL}" "${PM_ROOT_REL}")")
    done
    printf '%s\n' "${resolved[@]}"
}

collect_adrs() {
    local -a out=()

    if [[ "${STRICT}" -eq 1 ]]; then
        if [[ "${#ADR_PATHS_TASKS[@]}" -eq 0 && "${#ADR_REFS_TASKS[@]}" -eq 0 ]]; then
            die "strict pack (meta.slice_spec_version >= 2): add meta.adr_refs (or meta.adr_paths) to tasks.json to run the dispatcher"
        fi
    fi

    if [[ "${#ADR_PATHS_TASKS[@]}" -gt 0 ]]; then
        while IFS= read -r p; do
            [[ -n "${p}" ]] || continue
            out+=("${p}")
        done < <(resolve_adr_paths_list "${ADR_PATHS_TASKS[@]}")
        printf '%s\n' "${out[@]}"
        return 0
    fi

    if [[ "${#ADR_REFS_TASKS[@]}" -gt 0 ]]; then
        while IFS= read -r p; do
            [[ -n "${p}" ]] || continue
            out+=("${p}")
        done < <(resolve_adr_refs_list "${ADR_REFS_TASKS[@]}")
        printf '%s\n' "${out[@]}"
        return 0
    fi

    # Legacy fallback order
    local plan_file="${FEATURE_DIR_ABS}/plan.md"
    if [[ -f "${plan_file}" ]]; then
        mapfile -t ADR_REFS_PLAN < <(parse_plan_frontmatter_adr_refs "${plan_file}")
        if [[ "${#ADR_REFS_PLAN[@]}" -gt 0 ]]; then
            while IFS= read -r p; do
                [[ -n "${p}" ]] || continue
                out+=("${p}")
            done < <(resolve_adr_refs_list "${ADR_REFS_PLAN[@]}")
            printf '%s\n' "${out[@]}"
            return 0
        fi
    fi

    # Legacy fallback: ADR markdown files stored in the feature dir (historical packs).
    mapfile -t ADR_PATHS_IN_FEATURE < <(find "${FEATURE_DIR_ABS}" -maxdepth 1 -type f \( -name 'ADR-*.md' -o -iname 'adr*.md' \) 2>/dev/null || true)
    if [[ "${#ADR_PATHS_IN_FEATURE[@]}" -gt 0 ]]; then
        local a
        for a in "${ADR_PATHS_IN_FEATURE[@]}"; do
            out+=("$(relpath_in_repo "${REPO_ROOT}" "${a}")")
        done
        printf '%s\n' "${out[@]}"
        return 0
    fi

    local spec_manifest_file="${FEATURE_DIR_ABS}/spec_manifest.md"
    if [[ -f "${spec_manifest_file}" ]]; then
        mapfile -t ADR_PATHS_SPEC < <(parse_spec_manifest_adr_paths "${spec_manifest_file}")
        if [[ "${#ADR_PATHS_SPEC[@]}" -gt 0 ]]; then
            while IFS= read -r p; do
                [[ -n "${p}" ]] || continue
                out+=("${p}")
            done < <(resolve_adr_paths_list "${ADR_PATHS_SPEC[@]}")
            printf '%s\n' "${out[@]}"
            return 0
        fi
    fi

    mapfile -t ADR_MATCHES < <(find_adrs_by_feature_dir_match "${REPO_ROOT}" "${FEATURE_DIR_REL}" "${PM_ADRS_ROOT_REL}" "${PM_ROOT_REL}")
    if [[ "${#ADR_MATCHES[@]}" -gt 0 ]]; then
        local a
        for a in "${ADR_MATCHES[@]}"; do
            out+=("$(relpath_in_repo "${REPO_ROOT}" "${a}")")
        done
        printf '%s\n' "${out[@]}"
        return 0
    fi

    return 1
}

mapfile -t ADR_PATHS < <(collect_adrs || true)
if [[ "${#ADR_PATHS[@]}" -eq 0 ]]; then
    die "unable to resolve ADR(s) for ${FEATURE_DIR_REL}; add meta.adr_refs or meta.adr_paths to ${FEATURE_DIR_REL}/tasks.json"
fi

# De-duplicate ADR paths while preserving first occurrence order.
declare -A ADR_SEEN=()
ADR_PATHS_UNIQ=()
for p in "${ADR_PATHS[@]}"; do
    [[ -n "${p}" ]] || continue
    if [[ -z "${ADR_SEEN[${p}]+x}" ]]; then
        ADR_SEEN["${p}"]=1
        ADR_PATHS_UNIQ+=("${p}")
    fi
done

RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
RUN_DIR_ABS="${FEATURE_DIR_ABS}/logs/planning_agents/${AGENT}/${RUN_TS}"
mkdir -p "${RUN_DIR_ABS}"

PROMPT_OUT="${RUN_DIR_ABS}/prompt.md"
CODEX_LAST_MESSAGE="${RUN_DIR_ABS}/last_message.md"
CODEX_STDERR="${RUN_DIR_ABS}/stderr.txt"
if [[ "${CODEX_JSONL}" -eq 1 ]]; then
    CODEX_STDOUT="${RUN_DIR_ABS}/events.jsonl"
else
    CODEX_STDOUT="${RUN_DIR_ABS}/stdout.txt"
fi

PAYLOAD_TMP="${RUN_DIR_ABS}/_payload.txt"
extract_first_md_fence_payload "${REPO_ROOT}/${PROMPT_FILE_REL}" > "${PAYLOAD_TMP}" || true
if [[ ! -s "${PAYLOAD_TMP}" ]]; then
    die "prompt file does not contain a fenced md block payload (expected a line starting with three backticks + md): ${PROMPT_FILE_REL}"
fi

{
    printf 'Dispatcher context (do not remove):\n'
    printf '- Resolved feature dir: `%s/`\n' "${FEATURE_DIR_REL}"
    printf '- Resolved ADR paths:\n'
    for p in "${ADR_PATHS_UNIQ[@]}"; do
        printf '  - `%s`\n' "${p}"
    done
    printf '\nSingle-output rule:\n'
    printf '- Only write/overwrite: `%s`\n' "${ALLOWED_OUTPUT_REL}"
    printf '- Do not edit any other files. If you find follow-ups, record them *inside that output file* under a \"Follow-ups\" section.\n'
    printf '\n---\n\n'

	    sed \
	        -e "s|<FEATURE>|${FEATURE_SLUG}|g" \
	        -e "s|<FEATURE_DIR>|${FEATURE_DIR_REL}|g" \
	        "${PAYLOAD_TMP}"
} > "${PROMPT_OUT}"

codex_args=(codex exec --dangerously-bypass-approvals-and-sandbox --cd "${REPO_ROOT}")
if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
codex_args+=(--output-last-message "${CODEX_LAST_MESSAGE}" -)

set +e
"${codex_args[@]}" < "${PROMPT_OUT}" >"${CODEX_STDOUT}" 2>"${CODEX_STDERR}"
CODEX_EXIT="$?"
set -e

if [[ ! -f "${CODEX_LAST_MESSAGE}" ]]; then
    {
        printf 'This file was generated by `%s` because Codex did not write `--output-last-message`.\n' "${0}"
        printf '\n'
        printf 'Codex exit: %s\n' "${CODEX_EXIT}"
        printf 'Codex stderr: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${CODEX_STDERR}")"
        printf 'Codex stdout: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${CODEX_STDOUT}")"
    } >"${CODEX_LAST_MESSAGE}" 2>/dev/null || true
fi

mapfile -t CHANGED_TRACKED < <(git diff --name-only -- "${FEATURE_DIR_REL}" | sed '/^$/d')
mapfile -t UNTRACKED < <(git ls-files --others --exclude-standard -- "${FEATURE_DIR_REL}" | sed '/^$/d')

declare -A CHANGED_SEEN=()
CHANGED_UNION=()
for p in "${CHANGED_TRACKED[@]}" "${UNTRACKED[@]}"; do
    [[ -n "${p}" ]] || continue
    if [[ -z "${CHANGED_SEEN[${p}]+x}" ]]; then
        CHANGED_SEEN["${p}"]=1
        CHANGED_UNION+=("${p}")
    fi
done

if [[ "${#CHANGED_UNION[@]}" -ne 1 || "${CHANGED_UNION[0]}" != "${ALLOWED_OUTPUT_REL}" ]]; then
    echo "ERROR: single-output rule violated for ${FEATURE_DIR_REL}" >&2
    echo "  Allowed: ${ALLOWED_OUTPUT_REL}" >&2
    echo "  Changed/untracked within feature dir:" >&2
    if [[ "${#CHANGED_UNION[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${CHANGED_UNION[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Logs: $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

echo "OK: wrote/updated ${ALLOWED_OUTPUT_REL}"
echo "Logs: $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")"
exit "${CODEX_EXIT}"
