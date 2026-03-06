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

MAX_RESUMES="${PM_FULL_PLANNING_MAX_RESUMES:-6}"
if ! [[ "${MAX_RESUMES}" =~ ^[0-9]+$ ]]; then
    die "PM_FULL_PLANNING_MAX_RESUMES must be an integer (got ${MAX_RESUMES})"
fi

compute_plan_json() {
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
}

compute_plan_json

SLICE_PREFIX="$(jq -r '.slice_prefix' "${PLAN_JSON_ABS}")"
CONTRACT_ID="$(jq -r '.contract_pws_id' "${PLAN_JSON_ABS}")"
TASKS_ID="$(jq -r '.tasks_checkpoints_pws_id' "${PLAN_JSON_ABS}")"
TRIAGE_PATH_ABS="$(jq -r '.triage_path' "${PLAN_JSON_ABS}")"

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

PWS_DONE=()

is_pws_done() {
    local pid="$1"
    local x
    for x in ${PWS_DONE[@]+"${PWS_DONE[@]}"}; do
        if [[ "${x}" == "${pid}" ]]; then
            return 0
        fi
    done
    return 1
}

mark_pws_done() {
    local pid="$1"
    if ! is_pws_done "${pid}"; then
        PWS_DONE+=("${pid}")
    fi
}

normalize_requested_path() {
    local raw="$1"
    python3 - "${REPO_ROOT}" "${FEATURE_DIR_REL}" "${raw}" <<'PY'
from __future__ import annotations

import re
import sys
from pathlib import Path

repo = Path(sys.argv[1]).resolve()
feature_rel = sys.argv[2].rstrip("/")
raw = (sys.argv[3] or "").strip()
if not raw:
    raise SystemExit(2)

p = Path(raw)
if p.is_absolute():
    try:
        rel = p.resolve().relative_to(repo).as_posix()
    except Exception:
        raise SystemExit(2)
else:
    rel = raw.replace("\\", "/")

rel = rel.lstrip("./")
rel = re.sub(r"/{2,}", "/", rel)

feature_prefix = feature_rel + "/"
if rel == feature_rel:
    raise SystemExit(2)
if rel.startswith(feature_prefix):
    pack_rel = rel[len(feature_prefix) :]
else:
    pack_rel = rel

pack_rel = pack_rel.strip().lstrip("./")
if not pack_rel or pack_rel.startswith("/"):
    raise SystemExit(2)
parts = [seg for seg in pack_rel.split("/") if seg]
if any(seg == ".." for seg in parts):
    raise SystemExit(2)
if pack_rel.startswith("docs/"):
    # pack-relative paths must not be repo-root paths
    raise SystemExit(2)

print(pack_rel)
PY
}

find_owner_pws_for_pack_rel() {
    local pack_rel="$1"
    python3 - "${PLAN_JSON_ABS}" "${pack_rel}" <<'PY'
from __future__ import annotations

import json
import sys
from pathlib import Path

plan = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
path = (sys.argv[2] or "").strip()
if not path:
    raise SystemExit(0)

exact_owners: list[str] = []
best_owner: str | None = None
best_prefix: str | None = None

for pid, node in plan.get("pws", {}).items():
    owns = node.get("owns_norm") or []
    if not isinstance(owns, list):
        continue
    for own in owns:
        if not isinstance(own, str) or not own:
            continue
        if own.endswith("/"):
            continue
        if own == path:
            exact_owners.append(pid)

if len(exact_owners) > 1:
    print(f"ERROR: multiple exact owners for {path!r}: {sorted(exact_owners)!r}", file=sys.stderr)
    raise SystemExit(2)
if len(exact_owners) == 1:
    print(exact_owners[0])
    raise SystemExit(0)

for pid, node in plan.get("pws", {}).items():
    owns = node.get("owns_norm") or []
    if not isinstance(owns, list):
        continue
    for own in owns:
        if not isinstance(own, str) or not own:
            continue
        if not own.endswith("/"):
            continue
        if path.startswith(own):
            if best_prefix is None or len(own) > len(best_prefix):
                best_prefix = own
                best_owner = pid

if best_owner is not None:
    print(best_owner)
PY
}

grant_owns_in_triage() {
    local pid="$1"
    shift
    local -a to_add=("$@")
    [[ "${#to_add[@]}" -gt 0 ]] || return 0

    PLANNING_SCRIPTS_DIR="${PLANNING_SCRIPTS_DIR}" python3 - "${TRIAGE_PATH_ABS}" "${pid}" "${to_add[@]}" <<'PY'
from __future__ import annotations

import json
import os
import re
import sys
from pathlib import Path

scripts_dir = os.environ.get("PLANNING_SCRIPTS_DIR", "")
if scripts_dir:
    sys.path.insert(0, scripts_dir)

import validate_pws_index as vpi

triage_path = Path(sys.argv[1]).resolve()
pid = (sys.argv[2] or "").strip()
paths = [str(x) for x in sys.argv[3:]]

text = triage_path.read_text(encoding="utf-8")
idx = vpi._extract_pm_pws_index_json(text)

pws = idx.get("pws")
if not isinstance(pws, list):
    raise SystemExit(2)

entry = None
for raw in pws:
    if isinstance(raw, dict) and raw.get("id") == pid:
        entry = raw
        break
if entry is None:
    raise SystemExit(2)

owns = entry.get("owns")
if not isinstance(owns, list) or not all(isinstance(x, str) for x in owns):
    raise SystemExit(2)

normalized_add = []
for p in paths:
    norm = vpi._normalize_owns_path(p)
    if norm.endswith("//"):
        norm = norm.rstrip("/") + "/"
    normalized_add.append(norm)

for p in normalized_add:
    if p not in owns:
        owns.append(p)

begin = text.find(vpi.BEGIN_MARKER)
end = text.find(vpi.END_MARKER)
if begin < 0 or end < 0 or begin >= end:
    raise SystemExit(2)

segment = text[begin:end]
m = vpi.JSON_FENCE_RE.search(segment)
if not m:
    raise SystemExit(2)

new_body = json.dumps(idx, indent=2, sort_keys=False)
segment2 = segment[: m.start("body")] + new_body + segment[m.end("body") :]

out = text[:begin] + segment2 + text[end:]
triage_path.write_text(out, encoding="utf-8")
PY
}

commit_tracked_path() {
    local path="$1"
    local msg="$2"
    git add -- "${path}" >/dev/null 2>&1 || true
    if git diff --cached --quiet; then
        return 0
    fi
    git commit -m "${msg}" >/dev/null
    sha="$(git rev-parse HEAD)"
    echo "Committed: ${sha} (${msg})"
    append_summary "- committed \`${sha}\` (\`${msg}\`)"
}

get_thread_id() {
    local pid="$1"
    local f="${FEATURE_DIR_ABS}/logs/pws/${pid}/last_thread_id.txt"
    [[ -f "${f}" ]] || die "missing last_thread_id.txt for ${pid} (expected: ${FEATURE_DIR_REL}/logs/pws/${pid}/last_thread_id.txt)"
    tid="$(tr -d '[:space:]' < "${f}" || true)"
    [[ -n "${tid}" ]] || die "empty last_thread_id.txt for ${pid}: ${FEATURE_DIR_REL}/logs/pws/${pid}/last_thread_id.txt"
    printf '%s\n' "${tid}"
}

write_resume_message() {
    local pid="$1"
    local attempt="$2"
    local label="$3"
    local body="$4"
    local dir="${WRAPPER_DIR_ABS}/resume_messages"
    mkdir -p "${dir}"
    local path="${dir}/${pid}.attempt${attempt}.${label}.md"
    {
        printf 'Dispatcher resume (%s): `%s`\n\n' "${label}" "${pid}"
        printf '%s\n' "${body}"
    } >"${path}"
    printf '%s\n' "${path}"
}

run_pws_fresh() {
    local pid="$1"
    local -a args=("${RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --pws-id "${pid}" --codex-jsonl)
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    PM_PLANNING_ORCHESTRATED=1 "${args[@]}"
}

run_pws_resume() {
    local pid="$1"
    local thread_id="$2"
    local resume_message_path="$3"
    local -a args=("${RUNNER}" --feature-dir "${FEATURE_DIR_ABS}" --pws-id "${pid}" --codex-jsonl --resume-thread-id "${thread_id}" --resume-message "${resume_message_path}")
    if [[ -n "${CODEX_PROFILE}" ]]; then args+=(--codex-profile "${CODEX_PROFILE}"); fi
    if [[ -n "${CODEX_MODEL}" ]]; then args+=(--codex-model "${CODEX_MODEL}"); fi
    PM_PLANNING_ORCHESTRATED=1 "${args[@]}"
}

process_allowlist_request() {
    local pid="$1"
    local attempt="$2"
    local req_path="${FEATURE_DIR_ABS}/logs/pws/${pid}/allowlist_request.json"
    [[ -f "${req_path}" ]] || return 1

    local reason
    reason="$(jq -r '.reason // ""' "${req_path}" 2>/dev/null || echo "")"
    local -a raw_paths=()
    while IFS= read -r p; do
        [[ -n "${p}" ]] || continue
        raw_paths+=("${p}")
    done < <(jq -r '.requested_tracked_paths[]?' "${req_path}" 2>/dev/null | sed '/^$/d')

    if [[ "${#raw_paths[@]}" -eq 0 ]]; then
        die "allowlist_request.json exists but requested_tracked_paths is empty: ${FEATURE_DIR_REL}/logs/pws/${pid}/allowlist_request.json"
    fi

    local -a pack_paths=()
    for raw in "${raw_paths[@]}"; do
        pack_rel="$(normalize_requested_path "${raw}")" || die "invalid requested_tracked_path (must be pack-relative or inside feature dir): ${raw}"
        pack_paths+=("${pack_rel}")
    done

    local -a grant_paths=()
    local owner_pairs_file
    owner_pairs_file="$(mktemp "${WRAPPER_DIR_ABS}/.allowlist_owner_pairs.${pid}.attempt${attempt}.XXXXXX")"
    for pack_rel in "${pack_paths[@]}"; do
        owner="$(find_owner_pws_for_pack_rel "${pack_rel}" || true)"
        if [[ -z "${owner}" ]]; then
            grant_paths+=("${pack_rel}")
            continue
        fi
        if [[ "${owner}" == "${pid}" ]]; then
            continue
        fi
        printf '%s\t%s\n' "${owner}" "${pack_rel}" >> "${owner_pairs_file}"
    done

    if [[ "${#grant_paths[@]}" -gt 0 ]]; then
        echo "Auto-granting allowlist for ${pid}: ${grant_paths[*]}"
        grant_owns_in_triage "${pid}" "${grant_paths[@]}"

        triage_rel="$(python3 - "${REPO_ROOT}" "${TRIAGE_PATH_ABS}" <<'PY'
import sys
from pathlib import Path
repo = Path(sys.argv[1]).resolve()
p = Path(sys.argv[2]).resolve()
print(p.relative_to(repo).as_posix())
PY
)"
        commit_tracked_path "${triage_rel}" "docs: pws allowlist grant (${pid})"

        compute_plan_json
        TRIAGE_PATH_ABS="$(jq -r '.triage_path' "${PLAN_JSON_ABS}")"
    fi

    local -a owners=()
    if [[ -s "${owner_pairs_file}" ]]; then
        while IFS= read -r o; do
            [[ -n "${o}" ]] || continue
            owners+=("${o}")
        done < <(cut -f1 "${owner_pairs_file}" | sort -u)
    fi

    for owner in ${owners[@]+"${owners[@]}"}; do
        if ! is_pws_done "${owner}"; then
            die "allowlist_request from ${pid} targets path(s) owned by ${owner}, but ${owner} has not run yet (fix PM_PWS_INDEX depends_on ordering)"
        fi

        owner_role="$(jq -r --arg id "${owner}" '.pws[$id].role' "${PLAN_JSON_ABS}")"
        owner_paths="$(awk -F'\t' -v o="${owner}" '$1==o{print $2}' "${owner_pairs_file}" | paste -sd ' ' - 2>/dev/null | xargs)"

        owner_body=$(
            cat <<EOF
Another PWS is blocked and needs changes inside your owned outputs.

- Requestor PWS: \`${pid}\`
- Requested path(s): \`${owner_paths}\`
- Reason: ${reason}

Draft locations (preferred first):
- \`${FEATURE_DIR_REL}/logs/pws/${pid}/draft/<pack-relative path>\`
- \`${FEATURE_DIR_REL}/logs/pws/${pid}/draft.patch\`

Do the minimal tracked edits needed inside your allowlist to resolve the issue, then rerun:
\`make planning-micro-lint FEATURE_DIR="${FEATURE_DIR_REL}" OWNED_PATHS="<your owned paths>"\`
EOF
        )
        owner_msg_path="$(write_resume_message "${owner}" "${attempt}" "owner_fix_for_${pid}" "${owner_body}")"

        echo "== Auto-heal routing: resuming owner PWS ${owner} (requested by ${pid}) =="
        append_summary "- Routed allowlist request from \`${pid}\` to owner \`${owner}\` (paths=\`${owner_paths}\`)"

        # Resume owner until it passes runner + micro-lint.
        run_pws_until_done "${owner}" "${owner_role}" "${owner_msg_path}"
    done

    grant_csv="$(printf '%s\n' "${grant_paths[@]+"${grant_paths[@]}"}" | paste -sd ',' - 2>/dev/null || true)"
    owners_csv="$(printf '%s\n' "${owners[@]+"${owners[@]}"}" | sort | paste -sd ',' - 2>/dev/null || true)"
    req_body=$(
        cat <<EOF
Your allowlist_request.json was processed automatically.

- Granted new owns (if any): ${grant_csv:-"(none)"}
- Routed fixes to owner PWS (if any): ${owners_csv:-"(none)"}

Continue and complete this PWS. If still blocked, rewrite \`${FEATURE_DIR_REL}/logs/pws/${pid}/allowlist_request.json\` with the next required paths + drafts.
EOF
    )
    resume_msg_path="$(write_resume_message "${pid}" "${attempt}" "after_allowlist" "${req_body}")"
    printf '%s\n' "${resume_msg_path}"
    return 0
}

run_pws_until_done() {
    local pid="$1"
    local role="$2"
    local first_resume_msg="${3:-}"

    local attempt=0
    local resume_msg=""
    local first_is_resume=0
    if [[ -n "${first_resume_msg}" ]]; then
        first_is_resume=1
        resume_msg="${first_resume_msg}"
    fi

    while true; do
        if [[ "${attempt}" -ge "${MAX_RESUMES}" ]]; then
            die "exhausted resume attempts for ${pid} (MAX=${MAX_RESUMES}); see ${FEATURE_DIR_REL}/logs/pws/${pid}/"
        fi

        echo "-- Attempt ${attempt} for ${pid}"

        set +e
        if [[ "${attempt}" -eq 0 && "${first_is_resume}" -ne 1 ]]; then
            run_pws_fresh "${pid}"
            rc=$?
        else
            thread_id="$(get_thread_id "${pid}")"
            [[ -n "${resume_msg}" ]] || resume_msg="$(write_resume_message "${pid}" "${attempt}" "resume" "Resume and continue work for this PWS until all gates pass.")"
            run_pws_resume "${pid}" "${thread_id}" "${resume_msg}"
            rc=$?
        fi
        set -e

        # Always commit allowlisted outputs after each Codex run to keep the checkout clean for resumes.
        commit_pws_outputs "${pid}"

        if [[ "${rc}" -ne 0 ]]; then
            append_summary "- \`${pid}\`: runner failed (exit=${rc}); attempting auto-heal"
            if resume_after_allowlist="$(process_allowlist_request "${pid}" "${attempt}")"; then
                resume_msg="${resume_after_allowlist}"
            else
                body=$(
                    cat <<EOF
Runner failed for \`${pid}\` (role=\`${role}\`, exit=${rc}).

Inspect:
- Step logs: \`${FEATURE_DIR_REL}/logs/pws/${pid}\`
- Stable stderr: \`${FEATURE_DIR_REL}/logs/pws/${pid}/stderr.log\`

Fix the issue within the output allowlist and ensure self-checks pass. If you need additional tracked writes, write allowlist_request.json + drafts under \`${FEATURE_DIR_REL}/logs/pws/${pid}/\`.
EOF
                )
                resume_msg="$(write_resume_message "${pid}" "${attempt}" "runner_failed" "${body}")"
            fi
            attempt=$((attempt + 1))
            continue
        fi

        echo "-- Micro-lint (scoped): ${pid}"
        set +e
        micro_lint_pws "${pid}"
        ml_rc=$?
        set -e
        if [[ "${ml_rc}" -ne 0 ]]; then
            append_summary "- \`${pid}\`: micro-lint failed; attempting auto-heal"
            if resume_after_allowlist="$(process_allowlist_request "${pid}" "${attempt}")"; then
                resume_msg="${resume_after_allowlist}"
            else
                body=$(
                    cat <<EOF
planning-micro-lint failed for \`${pid}\` (role=\`${role}\`).

Fix hard-ban / ambiguity matches (and slice spec v2 structural rules when applicable), then rerun:
\`make planning-micro-lint FEATURE_DIR="${FEATURE_DIR_REL}" OWNED_PATHS="<owned paths>"\`

Step logs: \`${FEATURE_DIR_REL}/logs/pws/${pid}\`
EOF
                )
                resume_msg="$(write_resume_message "${pid}" "${attempt}" "micro_lint_failed" "${body}")"
            fi
            attempt=$((attempt + 1))
            continue
        fi

        # Final commit after micro-lint passes (usually no-op if already committed).
        commit_pws_outputs "${pid}"
        mark_pws_done "${pid}"
        return 0
    done
}

append_summary "## Execution"
append_summary ""

while IFS= read -r pid; do
    [[ -n "${pid}" ]] || continue
    role="$(jq -r --arg id "${pid}" '.pws[$id].role' "${PLAN_JSON_ABS}")"

    echo "== Running PWS: ${pid} (role=${role}) =="
    append_summary "- Started: \`${pid}\` (role=\`${role}\`)"

    run_pws_until_done "${pid}" "${role}" ""

    echo ""
done < <(jq -r '.run_order[]' "${PLAN_JSON_ABS}")

append_summary ""
append_summary "OK: full planning orchestration completed"
echo "OK: full planning orchestration completed"
echo "Summary: ${FEATURE_DIR_REL}/logs/full_planning_orchestrator/${RUN_TS}/summary.md"
