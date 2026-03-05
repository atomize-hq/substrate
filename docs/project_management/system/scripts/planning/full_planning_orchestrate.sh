#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  full_planning_orchestrate.sh --feature-dir <path> [--dry-run] [--workstream-triage <path>] [--codex-profile <p>] [--codex-model <m>] [--codex-jsonl]

Required:
  --feature-dir <path>        Planning Pack dir (docs/project_management/packs/<bucket>/<feature>)

Options:
  --dry-run                  Print the computed sequential run order and exit (no PWS runs).
  --workstream-triage <path> Path to workstream_triage.md (absolute or feature-dir-relative).
                             Default: pre-planning/workstream_triage.md (legacy fallback: workstream_triage.md)

Codex options (optional; forwarded to run_pws_agent.sh):
  --codex-profile <p>
  --codex-model <m>
  --codex-jsonl

Behavior:
  - Requires a clean orchestration checkout (git status must be empty).
  - Requires pre-planning alignment report: <FEATURE_DIR>/pre-planning/alignment_report.md
  - Runs <PREFIX>-PWS-contract first.
  - Runs all remaining runnable PWS sequentially in a stable, dependency-respecting order.
  - Runs <PREFIX>-PWS-tasks_checkpoints last.
  - After each PWS run:
    - Runs scoped micro-lint on that PWS's owns paths (hard-ban + ambiguity).
    - Commits allowlisted tracked outputs for that PWS (logs are not committed).
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
DRY_RUN=0
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
        --dry-run)
            DRY_RUN=1
            shift
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
need_cmd make
need_cmd rg
if [[ "${DRY_RUN}" -eq 0 ]]; then
    need_cmd codex
fi

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "not in a git repo/worktree (git rev-parse failed)"
cd "${REPO_ROOT}"

if [[ "${DRY_RUN}" -eq 0 && -n "$(git status --porcelain=v1)" ]]; then
    die "orchestration checkout is dirty; commit or stash before running"
fi

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_ROOT}/scripts/planning"
RUNNER="${PLANNING_SCRIPTS_DIR}/run_pws_agent.sh"
[[ -x "${RUNNER}" ]] || die "missing runner: ${RUNNER}"

FEATURE_DIR_REL="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" resolve-feature-dir --feature-dir "${FEATURE_DIR_RAW}")"
FEATURE_DIR_REL="${FEATURE_DIR_REL%/}"
FEATURE_DIR_ABS="${REPO_ROOT}/${FEATURE_DIR_REL}"
[[ -d "${FEATURE_DIR_ABS}" ]] || die "FEATURE_DIR does not exist: ${FEATURE_DIR_RAW} (resolved to ${FEATURE_DIR_REL})"

ALIGNMENT_REPORT_ABS="${FEATURE_DIR_ABS}/pre-planning/alignment_report.md"
[[ -f "${ALIGNMENT_REPORT_ABS}" ]] || die "missing required pre-planning alignment report: ${FEATURE_DIR_REL}/pre-planning/alignment_report.md"

LOGS_DIR_ABS="${FEATURE_DIR_ABS}/logs"
mkdir -p "${LOGS_DIR_ABS}"

RUN_TS="$(date -u +%Y%m%d-%H%M%S)"
WRAPPER_DIR_ABS="${LOGS_DIR_ABS}/full_planning_orchestrator/${RUN_TS}"
mkdir -p "${WRAPPER_DIR_ABS}"
SUMMARY_PATH_ABS="${WRAPPER_DIR_ABS}/summary.md"
PLAN_JSON_ABS="${WRAPPER_DIR_ABS}/pws_plan.json"

append_summary() {
    printf '%s\n' "$*" >>"${SUMMARY_PATH_ABS}"
}

append_summary "# Full Planning Orchestration Summary"
append_summary ""
append_summary "- Feature dir: \`${FEATURE_DIR_REL}/\`"
append_summary "- Run (UTC): \`${RUN_TS}\`"
append_summary "- Workstream triage: \`${WORKSTREAM_TRIAGE_REL}\`"
append_summary "- Alignment report (required): \`${FEATURE_DIR_REL}/pre-planning/alignment_report.md\`"
append_summary ""

PLANNING_SCRIPTS_DIR="${PLANNING_SCRIPTS_DIR}" python3 - "${FEATURE_DIR_ABS}" "${WORKSTREAM_TRIAGE_REL}" >"${PLAN_JSON_ABS}" <<'PY'
from __future__ import annotations

import heapq
import json
import os
import sys
from pathlib import Path

scripts_dir = os.environ.get("PLANNING_SCRIPTS_DIR", "")
if scripts_dir:
    sys.path.insert(0, scripts_dir)

import validate_pws_index as vpi

feature_dir = Path(sys.argv[1]).resolve()
triage_raw = sys.argv[2]

triage_path = vpi._resolve_triage_path(feature_dir, triage_raw, advisory=False)
assert triage_path is not None

errors = vpi._validate_doc(feature_dir, triage_path, advisory=False)
if errors:
    for e in errors:
        vpi._emit("FAIL", e.message)
    raise SystemExit(1)

text = triage_path.read_text(encoding="utf-8")
idx = vpi._extract_pm_pws_index_json(text)

slice_prefix = idx.get("slice_prefix")
if not isinstance(slice_prefix, str) or not slice_prefix.strip():
    vpi._emit("FAIL", f"{triage_path}: slice_prefix must be a non-empty string")
    raise SystemExit(1)
slice_prefix = slice_prefix.strip()

pws_list = idx.get("pws")
if not isinstance(pws_list, list) or not pws_list:
    vpi._emit("FAIL", f"{triage_path}: pws must be a non-empty array")
    raise SystemExit(1)

nodes: dict[str, dict[str, object]] = {}
deps: dict[str, set[str]] = {}
rev: dict[str, set[str]] = {}

for raw in pws_list:
    if not isinstance(raw, dict):
        continue
    pid = raw.get("id")
    role = raw.get("role")
    depends_on = raw.get("depends_on")
    owns = raw.get("owns")
    if not isinstance(pid, str) or not pid.strip():
        continue
    pid = pid.strip()
    if not isinstance(role, str) or not role.strip():
        continue
    role = role.strip()
    if not isinstance(depends_on, list) or not all(isinstance(x, str) for x in depends_on):
        continue
    if not isinstance(owns, list) or not all(isinstance(x, str) for x in owns):
        continue

    owns_norm = [vpi._normalize_owns_path(x) for x in owns]
    nodes[pid] = {
        "role": role,
        "depends_on": [x.strip() for x in depends_on],
        "owns_norm": owns_norm,
    }
    deps[pid] = {x.strip() for x in depends_on if str(x).strip()}
    rev[pid] = set()

for pid, dset in deps.items():
    for dep in dset:
        if dep in rev:
            rev[dep].add(pid)

indeg = {pid: len(d) for pid, d in deps.items()}
ready = [pid for pid, n in indeg.items() if n == 0]
heapq.heapify(ready)

topo: list[str] = []
while ready:
    pid = heapq.heappop(ready)
    topo.append(pid)
    for out in sorted(rev.get(pid, set())):
        indeg[out] -= 1
        if indeg[out] == 0:
            heapq.heappush(ready, out)

if len(topo) != len(nodes):
    vpi._emit("FAIL", f"{triage_path}: depends_on graph contains a cycle (unexpected after validation)")
    raise SystemExit(1)

contract_id = f"{slice_prefix}-PWS-contract"
tasks_id = f"{slice_prefix}-PWS-tasks_checkpoints"
if contract_id not in nodes:
    vpi._emit("FAIL", f"{triage_path}: missing required PWS: {contract_id}")
    raise SystemExit(1)
if tasks_id not in nodes:
    vpi._emit("FAIL", f"{triage_path}: missing required PWS: {tasks_id}")
    raise SystemExit(1)

contract_deps = nodes[contract_id].get("depends_on")
if isinstance(contract_deps, list) and contract_deps:
    vpi._emit("FAIL", f"{triage_path}: {contract_id}.depends_on must be empty (contract runs first)")
    raise SystemExit(1)

task_dependents = sorted(rev.get(tasks_id, set()))
if task_dependents:
    vpi._emit("FAIL", f"{triage_path}: {tasks_id} must not be a dependency of other PWS (must run last); dependents={task_dependents}")
    raise SystemExit(1)

run_order = [contract_id] + [pid for pid in topo if pid not in {contract_id, tasks_id}] + [tasks_id]
assert len(run_order) == len(nodes)

out = {
    "contract_pws_id": contract_id,
    "pws": nodes,
    "run_order": run_order,
    "slice_prefix": slice_prefix,
    "tasks_checkpoints_pws_id": tasks_id,
    "topo_order": topo,
    "triage_path": str(triage_path),
}
print(json.dumps(out, sort_keys=True))
PY

SLICE_PREFIX="$(jq -r '.slice_prefix' "${PLAN_JSON_ABS}")"
CONTRACT_ID="$(jq -r '.contract_pws_id' "${PLAN_JSON_ABS}")"
TASKS_ID="$(jq -r '.tasks_checkpoints_pws_id' "${PLAN_JSON_ABS}")"

append_summary "## Plan"
append_summary ""
append_summary "- slice_prefix: \`${SLICE_PREFIX}\`"
append_summary "- Contract: \`${CONTRACT_ID}\`"
append_summary "- Tasks/checkpoints: \`${TASKS_ID}\`"
append_summary ""
append_summary "Sequential run order:"
jq -r '.run_order[]' "${PLAN_JSON_ABS}" | nl -ba | sed 's/^/  /' >>"${SUMMARY_PATH_ABS}"
append_summary ""

echo "Full planning orchestrator: ${FEATURE_DIR_REL}/"
echo "slice_prefix: ${SLICE_PREFIX}"
echo "Run order:"
jq -r '.run_order[]' "${PLAN_JSON_ABS}" | nl -ba
echo ""
echo "Wrapper summary: ${FEATURE_DIR_REL}/logs/full_planning_orchestrator/${RUN_TS}/summary.md"
echo "Plan JSON:       ${FEATURE_DIR_REL}/logs/full_planning_orchestrator/${RUN_TS}/pws_plan.json"
echo ""

if [[ "${DRY_RUN}" -eq 1 ]]; then
    echo "DRY RUN: exiting without running any PWS."
    append_summary "DRY RUN: no PWS executed."
    exit 0
fi

commit_pws_outputs() {
    local pid="$1"
    local msg="docs: ${pid}"

    # Collect allowlisted paths (repo-relative).
    local -a allow_exact=()
    local -a allow_prefix=()
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        if [[ "${p}" == */ ]]; then
            allow_prefix+=("${FEATURE_DIR_REL}/${p}")
        else
            allow_exact+=("${FEATURE_DIR_REL}/${p}")
        fi
    done < <(jq -r --arg id "${pid}" '.pws[$id].owns_norm[]?' "${PLAN_JSON_ABS}" | sed '/^$/d')

    local -a allow=("${allow_exact[@]+"${allow_exact[@]}"}" "${allow_prefix[@]+"${allow_prefix[@]}"}")

    # Stage allowlisted paths only (safe: runner already enforces allowlist on tracked writes).
    git add -- "${allow[@]}" >/dev/null 2>&1 || true
    if git diff --cached --quiet; then
        echo "No tracked changes to commit for ${pid}"
        append_summary "- \`${pid}\`: no tracked changes to commit"
        return 0
    fi

    # Safety: ensure staged changes are a subset of allowlisted paths (exact + prefix).
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        ok=0
        for a in "${allow_exact[@]+"${allow_exact[@]}"}"; do
            if [[ "${p}" == "${a}" ]]; then
                ok=1
                break
            fi
        done
        if [[ "${ok}" -eq 0 ]]; then
            for pref in "${allow_prefix[@]+"${allow_prefix[@]}"}"; do
                if [[ "${p}" == "${pref}"* ]]; then
                    ok=1
                    break
                fi
            done
        fi
        if [[ "${ok}" -eq 0 ]]; then
            die "refusing to commit non-allowlisted path: ${p} (pws_id=${pid})"
        fi
    done < <(git diff --cached --name-only | sed '/^$/d')

    git commit -m "${msg}" >/dev/null
    sha="$(git rev-parse HEAD)"
    echo "Committed ${pid}: ${sha}"
    append_summary "- \`${pid}\`: committed \`${sha}\`"
}

micro_lint_pws() {
    local pid="$1"
    local owned_paths
    owned_paths="$(jq -r --arg id "${pid}" '.pws[$id].owns_norm[]?' "${PLAN_JSON_ABS}" | tr '\n' ' ' | xargs)"
    if [[ -z "${owned_paths}" ]]; then
        die "empty owns list for ${pid} (unexpected after validation)"
    fi
    make planning-micro-lint FEATURE_DIR="${FEATURE_DIR_REL}" OWNED_PATHS="${owned_paths}"
}

append_summary "## Execution"
append_summary ""

while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    role="$(jq -r --arg id "${pid}" '.pws[$id].role' "${PLAN_JSON_ABS}")"

    echo "== Running PWS: ${pid} (role=${role}) =="
    append_summary "- Started: \`${pid}\` (role=\`${role}\`)"

    args=("${RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --pws-id "${pid}")
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    if [[ "${CODEX_JSONL}" -eq 1 ]]; then args+=(--codex-jsonl); fi

    PM_PLANNING_ORCHESTRATED=1 "${args[@]}"

    echo "-- Micro-lint (scoped): ${pid}"
    micro_lint_pws "${pid}"

    echo "-- Commit: ${pid}"
    commit_pws_outputs "${pid}"

    echo ""
done < <(jq -r '.run_order[]' "${PLAN_JSON_ABS}")

append_summary ""
append_summary "OK: full planning orchestration completed"
echo "OK: full planning orchestration completed"
echo "Summary: ${FEATURE_DIR_REL}/logs/full_planning_orchestrator/${RUN_TS}/summary.md"
