#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'EOF'
Run a focused planning agent (output-allowlisted) via Codex.

Usage:
  run_planning_agent.sh --feature-dir <path> --agent <spec_manifest|impact_map|min_spec_draft|ci_checkpoint|workstream_triage> [options]

Required:
  --feature-dir <path>         Feature directory (relative or absolute)
  --agent <id>                 Agent id: spec_manifest | impact_map | min_spec_draft | ci_checkpoint | workstream_triage

Optional:
  --codex-profile <profile>    Passed to `codex exec --profile`
  --codex-model <model>        Passed to `codex exec --model`
  --codex-jsonl                Enable `codex exec --json` (stdout is events.jsonl)
  --help                       Show this help

Contract:
  - spec_manifest -> <FEATURE_DIR>/spec_manifest.md
  - impact_map    -> <FEATURE_DIR>/impact_map.md
  - min_spec_draft -> <FEATURE_DIR>/minimal_spec_draft.md
  - ci_checkpoint  -> <FEATURE_DIR>/ci_checkpoint_plan.md (and sometimes <FEATURE_DIR>/tasks.json)
  - workstream_triage -> logs-only (no tracked output)

Notes:
  - Uses roots from: `pm_paths.py` (sibling in this directory)
  - Enforces an output allowlist: only the intended tracked output(s) within FEATURE_DIR may change.
  - Writes run artifacts under: <FEATURE_DIR>/logs/<step>/runs/<YYYYMMDD-HHMMSS>/
  - Writes stable step artifacts under: <FEATURE_DIR>/logs/<step>/ (stderr.log, codex.pid, last_message.md)
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
        BEGIN { in_md=0 }
        /^```md[[:space:]]*$/ { in_md=1; next }
        in_md && /^```[[:space:]]*$/ { exit }
        in_md { print }
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
        NR==1 { if ($0 != "---") exit 0; in_fm=1; next }
        in_fm && $0=="---" { exit 0 }
        in_fm && $0 ~ /^adr_refs:[[:space:]]*$/ { mode="adr_refs"; next }
        in_fm && mode=="adr_refs" && $0 ~ /^[[:space:]]*-[[:space:]]*/ {
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
need_cmd ps

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
STEP_DIR_NAME=""
ALLOWED_OUTPUTS_REL=()
REQUIRED_OUTPUTS_REL=()
case "${AGENT}" in
    spec_manifest)
        STEP_DIR_NAME="spec-manifest"
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/spec_manifest_agent.md"
        ALLOWED_OUTPUTS_REL=("${FEATURE_DIR_REL}/spec_manifest.md")
        REQUIRED_OUTPUTS_REL=("${FEATURE_DIR_REL}/spec_manifest.md")
        ;;
    impact_map)
        STEP_DIR_NAME="impact-map"
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/impact_map_agent.md"
        ALLOWED_OUTPUTS_REL=("${FEATURE_DIR_REL}/impact_map.md")
        REQUIRED_OUTPUTS_REL=("${FEATURE_DIR_REL}/impact_map.md")
        ;;
    min_spec_draft)
        STEP_DIR_NAME="min-spec-draft"
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/min_spec_draft_agent.md"
        ALLOWED_OUTPUTS_REL=("${FEATURE_DIR_REL}/minimal_spec_draft.md")
        REQUIRED_OUTPUTS_REL=("${FEATURE_DIR_REL}/minimal_spec_draft.md")
        ;;
    ci_checkpoint)
        STEP_DIR_NAME="CI-checkpoint"
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/ci_checkpoint_agent.md"
        ALLOWED_OUTPUTS_REL=("${FEATURE_DIR_REL}/ci_checkpoint_plan.md" "${FEATURE_DIR_REL}/tasks.json")
        REQUIRED_OUTPUTS_REL=("${FEATURE_DIR_REL}/ci_checkpoint_plan.md")
        ;;
    workstream_triage)
        STEP_DIR_NAME="workstream-triage"
        PROMPT_FILE_REL="docs/project_management/system/prompts/planning/workstream_triage_agent.md"
        ALLOWED_OUTPUTS_REL=()
        REQUIRED_OUTPUTS_REL=()
        ;;
    *)
        die "unknown --agent: ${AGENT} (expected spec_manifest|impact_map|min_spec_draft|ci_checkpoint|workstream_triage)"
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

ADR_PATHS_TASKS=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    ADR_PATHS_TASKS+=("${p}")
done < <(jq -r '.meta.adr_paths // [] | .[]' "${TASKS_JSON_ABS}")

ADR_REFS_TASKS=()
while IFS= read -r r; do
    [[ -n "${r}" ]] || continue
    ADR_REFS_TASKS+=("${r}")
done < <(jq -r '.meta.adr_refs // [] | .[]' "${TASKS_JSON_ABS}")

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
	        local -a ADR_REFS_PLAN=()
	        while IFS= read -r r; do
	            [[ -n "${r}" ]] || continue
	            ADR_REFS_PLAN+=("${r}")
	        done < <(parse_plan_frontmatter_adr_refs "${plan_file}")
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
	    local -a ADR_PATHS_IN_FEATURE=()
	    while IFS= read -r p; do
	        [[ -n "${p}" ]] || continue
	        ADR_PATHS_IN_FEATURE+=("${p}")
	    done < <(find "${FEATURE_DIR_ABS}" -maxdepth 1 -type f \( -name 'ADR-*.md' -o -iname 'adr*.md' \) 2>/dev/null || true)
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
	        local -a ADR_PATHS_SPEC=()
	        while IFS= read -r p; do
	            [[ -n "${p}" ]] || continue
	            ADR_PATHS_SPEC+=("${p}")
	        done < <(parse_spec_manifest_adr_paths "${spec_manifest_file}")
        if [[ "${#ADR_PATHS_SPEC[@]}" -gt 0 ]]; then
            while IFS= read -r p; do
                [[ -n "${p}" ]] || continue
                out+=("${p}")
            done < <(resolve_adr_paths_list "${ADR_PATHS_SPEC[@]}")
            printf '%s\n' "${out[@]}"
            return 0
	        fi
	    fi

	    local -a ADR_MATCHES=()
	    while IFS= read -r p; do
	        [[ -n "${p}" ]] || continue
	        ADR_MATCHES+=("${p}")
	    done < <(find_adrs_by_feature_dir_match "${REPO_ROOT}" "${FEATURE_DIR_REL}" "${PM_ADRS_ROOT_REL}" "${PM_ROOT_REL}")
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

ADR_PATHS=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    ADR_PATHS+=("${p}")
done < <(collect_adrs || true)
if [[ "${#ADR_PATHS[@]}" -eq 0 ]]; then
    die "unable to resolve ADR(s) for ${FEATURE_DIR_REL}; add meta.adr_refs or meta.adr_paths to ${FEATURE_DIR_REL}/tasks.json"
fi

# De-duplicate ADR paths while preserving first occurrence order.
ADR_PATHS_UNIQ=()
	for p in "${ADR_PATHS[@]}"; do
	    [[ -n "${p}" ]] || continue
	    seen=0
	    for existing in ${ADR_PATHS_UNIQ[@]+"${ADR_PATHS_UNIQ[@]}"}; do
	        if [[ "${existing}" == "${p}" ]]; then
	            seen=1
	            break
	        fi
	    done
	    if [[ "${seen}" -eq 0 ]]; then
	        ADR_PATHS_UNIQ+=("${p}")
	    fi
	done

STEP_DIR_ABS="${FEATURE_DIR_ABS}/logs/${STEP_DIR_NAME}"
RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
RUN_DIR_ABS="${STEP_DIR_ABS}/runs/${RUN_TS}"
mkdir -p "${RUN_DIR_ABS}"

PROMPT_OUT="${RUN_DIR_ABS}/prompt.md"
CODEX_LAST_MESSAGE_RUN="${RUN_DIR_ABS}/last_message.run.md"
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
	    printf -- '- Resolved feature dir: `%s/`\n' "${FEATURE_DIR_REL}"
	    printf -- '- Resolved ADR paths:\n'
	    for p in "${ADR_PATHS_UNIQ[@]}"; do
	        printf '  - `%s`\n' "${p}"
	    done
	    if [[ "${PM_PLANNING_ORCHESTRATED:-0}" = "1" ]]; then
	        printf -- '- Orchestration mode: `pre_planning_research_orchestrate.sh` overlap run (do not ask the operator to commit/stash/clean; if a Phase B gate is blocked by upstream uncommitted outputs, keep polling — orchestration will commit allowlisted outputs)\n'
	    fi
	    printf '\nOutput allowlist (non-negotiable):\n'
	    if [[ "${#ALLOWED_OUTPUTS_REL[@]}" -eq 0 ]]; then
	        printf -- '- Tracked outputs: (none)\n'
	        printf -- '- Do not modify any tracked files.\n'
	        printf -- '- Logs allowed (untracked only): `%s/logs/%s/`\n' "${FEATURE_DIR_REL}" "${STEP_DIR_NAME}"
	        printf -- '- If you find follow-ups, record them inside your logs draft(s) under that logs directory.\n'
	    else
	        printf -- '- Tracked outputs (only these may change):\n'
	        for p in "${ALLOWED_OUTPUTS_REL[@]}"; do
	            printf '  - `%s`\n' "${p}"
	        done
	        printf -- '- Logs allowed (untracked only): `%s/logs/%s/`\n' "${FEATURE_DIR_REL}" "${STEP_DIR_NAME}"
	        printf -- '- Do not edit any other tracked files. If you find follow-ups, record them inside the relevant output under a \"Follow-ups\" section.\n'
	    fi
	    printf '\n---\n\n'

	    sed \
	        -e "s|<FEATURE>|${FEATURE_SLUG}|g" \
	        -e "s|<FEATURE_DIR>|${FEATURE_DIR_REL}|g" \
	        "${PAYLOAD_TMP}"
} > "${PROMPT_OUT}"

STEP_STDERR="${STEP_DIR_ABS}/stderr.log"
STEP_PID_PATH="${STEP_DIR_ABS}/codex.pid"
STABLE_LAST_MESSAGE="${STEP_DIR_ABS}/last_message.md"

wait_for_codex_pidfile_if_running() {
    local pid_path="$1"
    if [[ ! -f "${pid_path}" ]]; then
        return 0
    fi

    local pid
    pid="$(tr -d '[:space:]' < "${pid_path}" || true)"
    if [[ -z "${pid}" ]]; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    if ! kill -0 "${pid}" 2>/dev/null; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    local cmd
    cmd="$(ps -p "${pid}" -o cmd= 2>/dev/null || true)"
    if [[ -z "${cmd}" ]]; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    # Guard against PID reuse: only wait if it still looks like a Codex invocation.
    if ! printf '%s\n' "${cmd}" | grep -Eqi -- '(^|[[:space:]/])codex([[:space:]]|$)'; then
        rm -f "${pid_path}" >/dev/null 2>&1 || true
        return 0
    fi

    echo "WARN: codex.pid already exists for ${FEATURE_DIR_REL}/logs/${STEP_DIR_NAME} (pid=${pid}); waiting for it to exit" >&2
    while kill -0 "${pid}" 2>/dev/null; do
        sleep 2
    done
    rm -f "${pid_path}" >/dev/null 2>&1 || true
}

write_missing_last_message_stub() {
    local exit_code="${1:-unknown}"
    if [[ -s "${CODEX_LAST_MESSAGE_RUN}" ]]; then
        return 0
    fi
    mkdir -p "$(dirname "${CODEX_LAST_MESSAGE_RUN}")" >/dev/null 2>&1 || true
    {
        printf '# Generated planning step summary (Codex last message missing)\n\n'
        printf 'This file was generated by `%s` because Codex did not write `--output-last-message`.\n' "${0}"
        printf 'This typically means the Codex process was interrupted or crashed.\n\n'
        printf -- '- Agent: `%s`\n' "${AGENT}"
        printf -- '- Feature dir: `%s/`\n' "${FEATURE_DIR_REL}"
        printf -- '- Exit code: `%s`\n' "${exit_code}"
        printf -- '- Stable stderr log: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${STEP_STDERR}")"
        printf -- '- Stdout log: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${CODEX_STDOUT}")"
    } >"${CODEX_LAST_MESSAGE_RUN}" 2>/dev/null || true
}

codex_pid=""
cleanup_codex() {
    local rc="$?"
    if [[ -n "${codex_pid}" ]] && kill -0 "${codex_pid}" 2>/dev/null; then
        kill "${codex_pid}" >/dev/null 2>&1 || true
    fi
    rm -f "${STEP_PID_PATH}" >/dev/null 2>&1 || true
    write_missing_last_message_stub "${rc}"
}
trap cleanup_codex EXIT INT TERM

mkdir -p "${STEP_DIR_ABS}"
wait_for_codex_pidfile_if_running "${STEP_PID_PATH}"
: > "${STEP_STDERR}"

codex_args=(codex exec --dangerously-bypass-approvals-and-sandbox --cd "${REPO_ROOT}")
# Planning agents do not need Figma MCP and it can hang when no local MCP endpoint is running.
PM_CODEX_DISABLE_MCP_FIGMA_LOCAL="${PM_CODEX_DISABLE_MCP_FIGMA_LOCAL:-1}"
if [[ "${PM_CODEX_DISABLE_MCP_FIGMA_LOCAL}" = "1" ]]; then
    codex_args+=(--config mcp_servers.figma-local.enabled=false)
fi
if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
codex_args+=(--output-last-message "${CODEX_LAST_MESSAGE_RUN}" -)

set +e
"${codex_args[@]}" < "${PROMPT_OUT}" >"${CODEX_STDOUT}" 2>"${STEP_STDERR}" &
codex_pid="$!"
printf '%s\n' "${codex_pid}" > "${STEP_PID_PATH}"
wait "${codex_pid}"
CODEX_EXIT="$?"
rm -f "${STEP_PID_PATH}" >/dev/null 2>&1 || true
set -e

LAST_MESSAGE_WRITTEN_BY_CODEX=1
if [[ ! -s "${CODEX_LAST_MESSAGE_RUN}" ]]; then
    LAST_MESSAGE_WRITTEN_BY_CODEX=0
fi
write_missing_last_message_stub "${CODEX_EXIT}"
LAST_MESSAGE_OK=1
if [[ ! -s "${CODEX_LAST_MESSAGE_RUN}" ]]; then
    LAST_MESSAGE_OK=0
fi

CHANGED_TRACKED=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    CHANGED_TRACKED+=("${p}")
done < <(git diff --name-only -- "${FEATURE_DIR_REL}" | sed '/^$/d')

UNTRACKED=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    UNTRACKED+=("${p}")
done < <(git ls-files --others --exclude-standard -- "${FEATURE_DIR_REL}" | sed '/^$/d')

UNTRACKED_UNEXPECTED=()
for p in ${UNTRACKED[@]+"${UNTRACKED[@]}"}; do
    [[ -n "${p}" ]] || continue
    allowed=0
    for allowed_path in ${ALLOWED_OUTPUTS_REL[@]+"${ALLOWED_OUTPUTS_REL[@]}"}; do
        [[ -n "${allowed_path}" ]] || continue
        if [[ "${allowed_path}" == "${p}" ]]; then
            allowed=1
            break
        fi
    done
    if [[ "${allowed}" -eq 0 ]]; then
        UNTRACKED_UNEXPECTED+=("${p}")
    fi
done

if [[ "${#UNTRACKED_UNEXPECTED[@]}" -ne 0 ]]; then
    echo "ERROR: unexpected untracked (non-ignored) files exist within feature dir after agent run: ${FEATURE_DIR_REL}" >&2
    for p in "${UNTRACKED_UNEXPECTED[@]}"; do
        echo "  - ${p}" >&2
    done
    echo "  Allowed untracked outputs for this step:" >&2
    if [[ "${#ALLOWED_OUTPUTS_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_OUTPUTS_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

REQUIRED_OUTPUTS_OK=1
for p in ${REQUIRED_OUTPUTS_REL[@]+"${REQUIRED_OUTPUTS_REL[@]}"}; do
    [[ -n "${p}" ]] || continue
    if [[ ! -f "${REPO_ROOT}/${p}" ]]; then
        echo "ERROR: required output missing after agent run (agent=${AGENT}): ${p}" >&2
        REQUIRED_OUTPUTS_OK=0
    fi
done

violations=()
for p in ${CHANGED_TRACKED[@]+"${CHANGED_TRACKED[@]}"}; do
    [[ -n "${p}" ]] || continue
    allowed=0
    for allowed_path in ${ALLOWED_OUTPUTS_REL[@]+"${ALLOWED_OUTPUTS_REL[@]}"}; do
        [[ -n "${allowed_path}" ]] || continue
        if [[ "${allowed_path}" == "${p}" ]]; then
            allowed=1
            break
        fi
    done
    if [[ "${allowed}" -eq 0 ]]; then
        violations+=("${p}")
    fi
done

if [[ "${#violations[@]}" -ne 0 ]]; then
    echo "ERROR: output allowlist violated for ${FEATURE_DIR_REL} (agent=${AGENT})" >&2
    echo "  Allowed tracked outputs:" >&2
    if [[ "${#ALLOWED_OUTPUTS_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_OUTPUTS_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Changed tracked files within feature dir:" >&2
    if [[ "${#CHANGED_TRACKED[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${CHANGED_TRACKED[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

if [[ "${CODEX_EXIT}" -eq 0 && "${LAST_MESSAGE_OK}" -eq 1 && "${REQUIRED_OUTPUTS_OK}" -eq 1 ]]; then
    if ! cp "${CODEX_LAST_MESSAGE_RUN}" "${STABLE_LAST_MESSAGE}"; then
        echo "ERROR: failed to promote stable last_message.md for step ${FEATURE_DIR_REL}/logs/${STEP_DIR_NAME}" >&2
        echo "  From: $(relpath_in_repo "${REPO_ROOT}" "${CODEX_LAST_MESSAGE_RUN}")" >&2
        echo "  To:   $(relpath_in_repo "${REPO_ROOT}" "${STABLE_LAST_MESSAGE}")" >&2
        exit 2
    fi
fi

if [[ "${CODEX_EXIT}" -eq 0 && "${LAST_MESSAGE_WRITTEN_BY_CODEX}" -ne 1 ]]; then
    echo "WARN: Codex exited 0 but did not write --output-last-message; generated a stub summary" >&2
    echo "  Stub: $(relpath_in_repo "${REPO_ROOT}" "${CODEX_LAST_MESSAGE_RUN}")" >&2
    echo "  Stable stderr log: $(relpath_in_repo "${REPO_ROOT}" "${STEP_STDERR}")" >&2
fi
if [[ "${CODEX_EXIT}" -eq 0 && "${REQUIRED_OUTPUTS_OK}" -ne 1 ]]; then
    echo "ERROR: agent exited 0 but required outputs are missing (treated as failure)" >&2
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

if [[ "${#ALLOWED_OUTPUTS_REL[@]}" -eq 0 ]]; then
    echo "OK: logs-only step (no tracked changes)"
else
    echo "OK: tracked outputs within allowlist"
fi
echo "Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")"
echo "Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")"
if [[ -f "${STABLE_LAST_MESSAGE}" ]]; then
    echo "Stable last message: $(relpath_in_repo "${REPO_ROOT}" "${STABLE_LAST_MESSAGE}")"
fi
exit "${CODEX_EXIT}"
