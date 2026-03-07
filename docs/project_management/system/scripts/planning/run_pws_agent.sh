#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

usage() {
    cat <<'EOF'
Run a single Planning Workstream (PWS) agent (strict owns-allowlisted) via Codex.

Usage:
  run_pws_agent.sh --feature-dir <path> --pws-id <PWS_ID> [options]

Required:
  --feature-dir <path>           Feature directory (relative or absolute)
  --pws-id <PWS_ID>              Exact PWS id to run

Optional:
  --workstream-triage <path>     Path to workstream_triage.md (absolute or feature-dir-relative).
                                 Default: pre-planning/workstream_triage.md (legacy fallback: workstream_triage.md)
  --codex-profile <profile>      Passed to `codex exec --profile`
  --codex-model <model>          Passed to `codex exec --model`
  --codex-jsonl                  Enable `codex exec --json` (stdout is events.jsonl)
  --resume-thread-id <UUID>      Resume an existing Codex session (same PWS) instead of starting a new one
  --resume-message <path>        Path to a resume message to send (optional; default message if omitted)
  --help                         Show this help

Contract:
  - Hard-validates PM_PWS_INDEX via validate_pws_index.py (library import)
  - Enforces tracked-write allowlist equal to that PWS's owns (exact + prefix via trailing '/')
  - Writes run artifacts under: <FEATURE_DIR>/logs/pws/<PWS_ID>/runs/<UTC_TS>/
  - Writes stable PWS artifacts under: <FEATURE_DIR>/logs/pws/<PWS_ID>/ (stderr.log, codex.pid, last_message.md)
  - For role=tasks_checkpoints, success requires post-run validators (tasks.json + slice specs + checkpoint plan when applicable).
    - Optional: set PM_PWS_RUN_PLANNING_LINT=1 to also run `make planning-lint FEATURE_DIR="<FEATURE_DIR>"`.

Exit codes:
  0 success
  1 Codex exited non-zero (allowlist checks still enforced)
  2 usage/config/validation/allowlist violations
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

FEATURE_DIR_RAW=""
PWS_ID=""
WORKSTREAM_TRIAGE=""
CODEX_PROFILE=""
CODEX_MODEL=""
CODEX_JSONL=0
RESUME_THREAD_ID=""
RESUME_MESSAGE_PATH=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR_RAW="${2:-}"
            shift 2
            ;;
        --pws-id)
            PWS_ID="${2:-}"
            shift 2
            ;;
        --workstream-triage)
            WORKSTREAM_TRIAGE="${2:-}"
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
        --resume-thread-id)
            RESUME_THREAD_ID="${2:-}"
            shift 2
            ;;
        --resume-message)
            RESUME_MESSAGE_PATH="${2:-}"
            shift 2
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
[[ -n "${PWS_ID}" ]] || die "--pws-id is required"
if [[ -n "${RESUME_MESSAGE_PATH}" && -z "${RESUME_THREAD_ID}" ]]; then
    die "--resume-message requires --resume-thread-id"
fi
if [[ -n "${RESUME_THREAD_ID}" ]]; then
    RESUME_THREAD_ID="$(echo "${RESUME_THREAD_ID}" | xargs)"
    [[ -n "${RESUME_THREAD_ID}" ]] || die "--resume-thread-id cannot be empty"
fi

need_cmd git
need_cmd python3
need_cmd jq
need_cmd codex
need_cmd ps

REPO_ROOT="$(repo_root)"
[[ -n "${REPO_ROOT}" ]] || die "not in a git repo/worktree (git rev-parse failed)"

cd "${REPO_ROOT}"

# Enforce a clean orchestration checkout for strict allowlist semantics.
if [[ -n "$(git status --porcelain=v1)" ]]; then
    echo "ERROR: orchestration checkout is dirty; commit or stash before running" >&2
    git status --porcelain=v1 >&2
    exit 2
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist or is not a directory: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"

extract_args=(python3 "${PLANNING_SCRIPTS_DIR}/pm_pws_index_extract.py" --feature-dir "${FEATURE_DIR_ABS}" --pws-id "${PWS_ID}")
if [[ -n "${WORKSTREAM_TRIAGE}" ]]; then
    extract_args+=(--workstream-triage "${WORKSTREAM_TRIAGE}")
fi
INDEX_JSON="$("${extract_args[@]}")"

SLICE_PREFIX="$(printf '%s\n' "${INDEX_JSON}" | jq -r '.slice_prefix')"
ROLE="$(printf '%s\n' "${INDEX_JSON}" | jq -r '.role')"

DEPENDS_ON=()
while IFS= read -r d; do
    [[ -n "${d}" ]] || continue
    DEPENDS_ON+=("${d}")
done < <(printf '%s\n' "${INDEX_JSON}" | jq -r '.depends_on[]?' | sed '/^$/d')

OWNS_EXACT=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    OWNS_EXACT+=("${p}")
done < <(printf '%s\n' "${INDEX_JSON}" | jq -r '.owns_exact_norm[]?' | sed '/^$/d')

OWNS_PREFIX=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    OWNS_PREFIX+=("${p}")
done < <(printf '%s\n' "${INDEX_JSON}" | jq -r '.owns_prefix_norm[]?' | sed '/^$/d')

ALLOWED_EXACT_REL=()
for p in ${OWNS_EXACT[@]+"${OWNS_EXACT[@]}"}; do
    [[ -n "${p}" ]] || continue
    ALLOWED_EXACT_REL+=("${FEATURE_DIR_REL}/${p}")
done

ALLOWED_PREFIX_REL=()
for p in ${OWNS_PREFIX[@]+"${OWNS_PREFIX[@]}"}; do
    [[ -n "${p}" ]] || continue
    ALLOWED_PREFIX_REL+=("${FEATURE_DIR_REL}/${p}")
done

PROMPT_TEMPLATE_REL="docs/project_management/system/prompts/planning/pws_generic_agent.md"
if [[ "${ROLE}" == "contract" ]]; then
    PROMPT_TEMPLATE_REL="docs/project_management/system/prompts/planning/pws_contract_agent.md"
elif [[ "${ROLE}" == "tasks_checkpoints" ]]; then
    PROMPT_TEMPLATE_REL="docs/project_management/system/prompts/planning/pws_tasks_checkpoints_agent.md"
fi
[[ -f "${REPO_ROOT}/${PROMPT_TEMPLATE_REL}" ]] || die "missing prompt template: ${PROMPT_TEMPLATE_REL}"

STEP_DIR_ABS="${FEATURE_DIR_ABS}/logs/pws/${PWS_ID}"
RUN_TS="$(date -u +%Y%m%dT%H%M%SZ)"
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

if [[ -n "${RESUME_THREAD_ID}" ]]; then
    if [[ -n "${RESUME_MESSAGE_PATH}" ]]; then
        if [[ "${RESUME_MESSAGE_PATH}" != /* ]]; then
            RESUME_MESSAGE_PATH="${REPO_ROOT}/${RESUME_MESSAGE_PATH}"
        fi
        [[ -f "${RESUME_MESSAGE_PATH}" ]] || die "resume message file not found: ${RESUME_MESSAGE_PATH}"
        cat "${RESUME_MESSAGE_PATH}" > "${PAYLOAD_TMP}"
    else
        cat > "${PAYLOAD_TMP}" <<'EOF'
Resume: continue this PWS until all runner gates pass. If still blocked, rewrite allowlist_request.json with pws_id, requested_tracked_paths, and reason.
EOF
    fi
else
    extract_first_md_fence_payload "${REPO_ROOT}/${PROMPT_TEMPLATE_REL}" > "${PAYLOAD_TMP}" || true
    if [[ ! -s "${PAYLOAD_TMP}" ]]; then
        die "prompt template does not contain a fenced md block payload (expected a line starting with three backticks + md): ${PROMPT_TEMPLATE_REL}"
    fi
fi

{
    printf 'Dispatcher context (do not remove):\n'
    printf -- '- Resolved feature dir: `%s/`\n' "${FEATURE_DIR_REL}"
    printf -- '- slice_prefix: `%s`\n' "${SLICE_PREFIX}"
    printf -- '- PWS id: `%s` (role=%s)\n' "${PWS_ID}" "${ROLE}"
    if [[ "${#DEPENDS_ON[@]}" -eq 0 ]]; then
        printf -- '- depends_on: (none)\n'
    else
        printf -- '- depends_on (informational; runner does not auto-run deps in Step 3):\n'
        for d in "${DEPENDS_ON[@]}"; do
            printf '  - `%s`\n' "${d}"
        done
    fi

    printf '\nOutput allowlist (non-negotiable):\n'
    if [[ "${#ALLOWED_EXACT_REL[@]}" -eq 0 && "${#ALLOWED_PREFIX_REL[@]}" -eq 0 ]]; then
        printf -- '- Tracked outputs: (none)\n'
        printf -- '- Do not modify any tracked files.\n'
    else
        printf -- '- Tracked outputs (exact):\n'
        if [[ "${#ALLOWED_EXACT_REL[@]}" -eq 0 ]]; then
            printf -- '  - (none)\n'
        else
            for p in "${ALLOWED_EXACT_REL[@]}"; do
                printf '  - `%s`\n' "${p}"
            done
        fi
        printf -- '- Tracked outputs (prefix):\n'
        if [[ "${#ALLOWED_PREFIX_REL[@]}" -eq 0 ]]; then
            printf -- '  - (none)\n'
        else
            for p in "${ALLOWED_PREFIX_REL[@]}"; do
                printf '  - (prefix) `%s`\n' "${p}"
            done
        fi
    fi

    printf -- '- Logs allowed (untracked only): `%s/logs/pws/%s/`\n' "${FEATURE_DIR_REL}" "${PWS_ID}"
    printf -- '- If blocked by needing more tracked edits:\n'
    printf -- '  - Write logs-only artifacts under that logs directory:\n'
    printf -- '    - `allowlist_request.json` with exact JSON keys: `pws_id`, `requested_tracked_paths`, `reason`\n'
    printf -- '      - Migration note: legacy `requested_paths` is still accepted by the orchestrator, but do not emit it in new requests.\n'
    printf -- '    - `draft.patch` and/or `draft/<path>` (proposed changes)\n'
    printf -- '  - Do not edit disallowed tracked files.\n'

    printf '\n---\n\n'

    sed \
        -e "s|<FEATURE_DIR>|${FEATURE_DIR_REL}|g" \
        -e "s|<PWS_ID>|${PWS_ID}|g" \
        -e "s|<ROLE>|${ROLE}|g" \
        -e "s|<SLICE_PREFIX>|${SLICE_PREFIX}|g" \
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

    echo "WARN: codex.pid already exists for ${FEATURE_DIR_REL}/logs/pws/${PWS_ID} (pid=${pid}); waiting for it to exit" >&2
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
        printf '# Generated PWS run summary (Codex last message missing)\n\n'
        printf 'This file was generated by `%s` because Codex did not write `--output-last-message`.\n' "${0}"
        printf 'This typically means the Codex process was interrupted or crashed.\n\n'
        printf -- '- PWS id: `%s`\n' "${PWS_ID}"
        printf -- '- Role: `%s`\n' "${ROLE}"
        printf -- '- Feature dir: `%s/`\n' "${FEATURE_DIR_REL}"
        printf -- '- Exit code: `%s`\n' "${exit_code}"
        printf -- '- Stable stderr log: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${STEP_STDERR}")"
        printf -- '- Stdout log: `%s`\n' "$(relpath_in_repo "${REPO_ROOT}" "${CODEX_STDOUT}")"
    } >"${CODEX_LAST_MESSAGE_RUN}" 2>/dev/null || true
}

is_allowed_output_path() {
    local path="$1"
    local allowed
    for allowed in ${ALLOWED_EXACT_REL[@]+"${ALLOWED_EXACT_REL[@]}"}; do
        [[ -n "${allowed}" ]] || continue
        if [[ "${allowed}" == "${path}" ]]; then
            return 0
        fi
    done
    for allowed in ${ALLOWED_PREFIX_REL[@]+"${ALLOWED_PREFIX_REL[@]}"}; do
        [[ -n "${allowed}" ]] || continue
        if [[ "${path}" == "${allowed}"* ]]; then
            return 0
        fi
    done
    return 1
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
# PWS agents do not need Figma MCP and it can hang when no local MCP endpoint is running.
PM_CODEX_DISABLE_MCP_FIGMA_LOCAL="${PM_CODEX_DISABLE_MCP_FIGMA_LOCAL:-1}"
if [[ "${PM_CODEX_DISABLE_MCP_FIGMA_LOCAL}" = "1" ]]; then
    codex_args+=(--config mcp_servers.figma-local.enabled=false)
fi
# PWS agents primarily use local repo reads + shell commands. Disable non-essential MCP servers
# by default to reduce startup overhead and avoid hangs when local MCP endpoints are unavailable.
PM_CODEX_DISABLE_MCP_PLANNING_DEFAULTS="${PM_CODEX_DISABLE_MCP_PLANNING_DEFAULTS:-1}"
if [[ "${PM_CODEX_DISABLE_MCP_PLANNING_DEFAULTS}" = "1" ]]; then
    codex_args+=(--config mcp_servers.google-docs-mcp.enabled=false)
    codex_args+=(--config mcp_servers.cloudflare-docs.enabled=false)
    codex_args+=(--config mcp_servers.pencil.enabled=false)
    codex_args+=(--config mcp_servers.gsd.enabled=false)
    codex_args+=(--config mcp_servers.deepwiki.enabled=false)
fi
if [[ -n "${CODEX_PROFILE}" ]]; then codex_args+=(--profile "${CODEX_PROFILE}"); fi
if [[ -n "${CODEX_MODEL}" ]]; then codex_args+=(--model "${CODEX_MODEL}"); fi
if [[ "${CODEX_JSONL}" -eq 1 ]]; then codex_args+=(--json); fi
codex_args+=(--output-last-message "${CODEX_LAST_MESSAGE_RUN}")
if [[ -n "${RESUME_THREAD_ID}" ]]; then
    codex_args+=(resume "${RESUME_THREAD_ID}" -)
else
    codex_args+=(-)
fi

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

THREAD_ID=""
if [[ "${CODEX_JSONL}" -eq 1 && -s "${CODEX_STDOUT}" ]]; then
    THREAD_ID="$(python3 - "${CODEX_STDOUT}" <<'PY' 2>/dev/null || true
import json
import sys
from pathlib import Path

p = Path(sys.argv[1])
for raw in p.read_text(encoding="utf-8", errors="replace").splitlines():
    raw = raw.strip()
    if not raw:
        continue
    try:
        obj = json.loads(raw)
    except Exception:
        continue
    tid = obj.get("thread_id")
    if isinstance(tid, str) and tid.strip():
        print(tid.strip())
        raise SystemExit(0)
raise SystemExit(1)
PY
)"
fi
if [[ -z "${THREAD_ID}" && -n "${RESUME_THREAD_ID}" ]]; then
    THREAD_ID="${RESUME_THREAD_ID}"
fi
if [[ -n "${THREAD_ID}" ]]; then
    printf '%s\n' "${THREAD_ID}" > "${RUN_DIR_ABS}/thread_id.txt" 2>/dev/null || true
    printf '%s\n' "${THREAD_ID}" > "${STEP_DIR_ABS}/last_thread_id.txt" 2>/dev/null || true
fi

CHANGED_TRACKED_ALL=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    CHANGED_TRACKED_ALL+=("${p}")
done < <({ git diff --name-only; git diff --cached --name-only; } | sed '/^$/d' | sort -u)

CHANGED_TRACKED_OUTSIDE_PACK=()
CHANGED_TRACKED_IN_PACK=()
for p in ${CHANGED_TRACKED_ALL[@]+"${CHANGED_TRACKED_ALL[@]}"}; do
    [[ -n "${p}" ]] || continue
    if [[ "${p}" == "${FEATURE_DIR_REL}/"* ]]; then
        CHANGED_TRACKED_IN_PACK+=("${p}")
    else
        CHANGED_TRACKED_OUTSIDE_PACK+=("${p}")
    fi
done

if [[ "${#CHANGED_TRACKED_OUTSIDE_PACK[@]}" -ne 0 ]]; then
    echo "ERROR: tracked changes exist outside pack root after PWS run (not allowed): ${FEATURE_DIR_REL}" >&2
    for p in "${CHANGED_TRACKED_OUTSIDE_PACK[@]}"; do
        echo "  - ${p}" >&2
    done
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

UNTRACKED_IN_PACK=()
while IFS= read -r p; do
    [[ -n "${p}" ]] || continue
    UNTRACKED_IN_PACK+=("${p}")
done < <(git ls-files --others --exclude-standard -- "${FEATURE_DIR_REL}" | sed '/^$/d' | sort -u)

UNTRACKED_UNEXPECTED=()
for p in ${UNTRACKED_IN_PACK[@]+"${UNTRACKED_IN_PACK[@]}"}; do
    [[ -n "${p}" ]] || continue
    if ! is_allowed_output_path "${p}"; then
        UNTRACKED_UNEXPECTED+=("${p}")
    fi
done

if [[ "${#UNTRACKED_UNEXPECTED[@]}" -ne 0 ]]; then
    echo "ERROR: unexpected untracked (non-ignored) files exist within pack after PWS run: ${FEATURE_DIR_REL}" >&2
    for p in "${UNTRACKED_UNEXPECTED[@]}"; do
        echo "  - ${p}" >&2
    done
    echo "  Allowed exact outputs:" >&2
    if [[ "${#ALLOWED_EXACT_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_EXACT_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Allowed prefix outputs:" >&2
    if [[ "${#ALLOWED_PREFIX_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_PREFIX_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

ALLOWLIST_VIOLATIONS=()
for p in ${CHANGED_TRACKED_IN_PACK[@]+"${CHANGED_TRACKED_IN_PACK[@]}"}; do
    [[ -n "${p}" ]] || continue
    if ! is_allowed_output_path "${p}"; then
        ALLOWLIST_VIOLATIONS+=("${p}")
    fi
done

if [[ "${#ALLOWLIST_VIOLATIONS[@]}" -ne 0 ]]; then
    echo "ERROR: owns allowlist violated for ${FEATURE_DIR_REL} (pws_id=${PWS_ID}, role=${ROLE})" >&2
    echo "  Allowed exact outputs:" >&2
    if [[ "${#ALLOWED_EXACT_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_EXACT_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Allowed prefix outputs:" >&2
    if [[ "${#ALLOWED_PREFIX_REL[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${ALLOWED_PREFIX_REL[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Changed tracked files within pack:" >&2
    if [[ "${#CHANGED_TRACKED_IN_PACK[@]}" -eq 0 ]]; then
        echo "    (none)" >&2
    else
        for p in "${CHANGED_TRACKED_IN_PACK[@]}"; do
            echo "    - ${p}" >&2
        done
    fi
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

REQUIRED_OUTPUTS_OK=1
for p in ${ALLOWED_EXACT_REL[@]+"${ALLOWED_EXACT_REL[@]}"}; do
    [[ -n "${p}" ]] || continue
    if [[ ! -e "${REPO_ROOT}/${p}" ]]; then
        echo "ERROR: required output missing after PWS run: ${p}" >&2
        REQUIRED_OUTPUTS_OK=0
    fi
done

if [[ "${CODEX_EXIT}" -eq 0 && "${LAST_MESSAGE_OK}" -eq 1 && "${REQUIRED_OUTPUTS_OK}" -eq 1 ]]; then
    if [[ "${ROLE}" == "tasks_checkpoints" ]]; then
        if ! python3 "${PLANNING_SCRIPTS_DIR}/validate_tasks_json.py" --feature-dir "${FEATURE_DIR_ABS}"; then
            echo "ERROR: validate_tasks_json.py failed after tasks_checkpoints PWS run: ${FEATURE_DIR_REL}" >&2
            echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
            echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
            exit 2
        fi
        slice_inventory_args=(
            python3 "${PLANNING_SCRIPTS_DIR}/validate_slice_inventory_coherence.py"
            --feature-dir "${FEATURE_DIR_ABS}"
            --phase execution_ready
        )
        if [[ -n "${WORKSTREAM_TRIAGE}" ]]; then
            slice_inventory_args+=(--workstream-triage "${WORKSTREAM_TRIAGE}")
        fi
        if ! "${slice_inventory_args[@]}"; then
            echo "ERROR: validate_slice_inventory_coherence.py failed after tasks_checkpoints PWS run: ${FEATURE_DIR_REL}" >&2
            echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
            echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
            exit 2
        fi
        if ! python3 "${PLANNING_SCRIPTS_DIR}/validate_slice_specs.py" --feature-dir "${FEATURE_DIR_ABS}"; then
            echo "ERROR: validate_slice_specs.py failed after tasks_checkpoints PWS run: ${FEATURE_DIR_REL}" >&2
            echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
            echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
            exit 2
        fi

        checkpoint_plan_exists=0
        if [[ -f "${FEATURE_DIR_ABS}/pre-planning/ci_checkpoint_plan.md" || -f "${FEATURE_DIR_ABS}/ci_checkpoint_plan.md" ]]; then
            checkpoint_plan_exists=1
        fi
        cross_platform_enabled="$(jq -r '.meta.cross_platform // false' "${FEATURE_DIR_ABS}/tasks.json" 2>/dev/null || echo "false")"
        if [[ "${checkpoint_plan_exists}" -eq 1 || "${cross_platform_enabled}" == "true" ]]; then
            checkpoint_args=(
                python3 "${PLANNING_SCRIPTS_DIR}/validate_ci_checkpoint_plan.py"
                --feature-dir "${FEATURE_DIR_ABS}"
            )
            if [[ -n "${WORKSTREAM_TRIAGE}" ]]; then
                checkpoint_args+=(--workstream-triage "${WORKSTREAM_TRIAGE}")
            fi
            if ! "${checkpoint_args[@]}"; then
                echo "ERROR: validate_ci_checkpoint_plan.py failed after tasks_checkpoints PWS run: ${FEATURE_DIR_REL}" >&2
                echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
                echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
                exit 2
            fi
        fi

        if [[ "${PM_PWS_RUN_PLANNING_LINT:-0}" == "1" ]]; then
            if ! make planning-lint FEATURE_DIR="${FEATURE_DIR_REL}"; then
                echo "ERROR: planning-lint failed after tasks_checkpoints PWS run: ${FEATURE_DIR_REL}" >&2
                echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
                echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
                exit 2
            fi
        fi
    fi
    if ! cp "${CODEX_LAST_MESSAGE_RUN}" "${STABLE_LAST_MESSAGE}"; then
        echo "ERROR: failed to promote stable last_message.md for PWS ${PWS_ID}" >&2
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
    echo "ERROR: PWS exited 0 but required outputs are missing (treated as failure)" >&2
    echo "  Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")" >&2
    echo "  Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")" >&2
    exit 2
fi

echo "Step logs: $(relpath_in_repo "${REPO_ROOT}" "${STEP_DIR_ABS}")"
echo "Run logs:  $(relpath_in_repo "${REPO_ROOT}" "${RUN_DIR_ABS}")"
if [[ -f "${STABLE_LAST_MESSAGE}" ]]; then
    echo "Stable last message: $(relpath_in_repo "${REPO_ROOT}" "${STABLE_LAST_MESSAGE}")"
fi

if [[ "${CODEX_EXIT}" -eq 0 ]]; then
    echo "OK: PWS run complete (allowlist enforced)"
    exit 0
fi

echo "ERROR: Codex exited non-zero (exit=${CODEX_EXIT})" >&2
exit 1
