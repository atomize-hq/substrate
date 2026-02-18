#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  make triad-task-finish TASK_ID=<id> [VERIFY_ONLY=1] [SMOKE=1] [...]

Required:
  --task-id <id>         Task id (must match .taskmeta.json in this worktree)

Options:
  --verify-only          Do not run checks; only validate invariants and print summary
  --no-commit            Do not create a commit (for investigation-only runs)
  --allow-unplanned-touch Allow completion even if unplanned touches exist (STRICT packs only; requires --reason)
  --reason "<text>"      Required when --allow-unplanned-touch is set; auditable justification
  --platform <p>         linux|macos|windows|wsl (optional; used for smoke if requested)
  --smoke                Run `make feature-smoke` for platform-fix tasks (requires gh auth)
  --dry-run              Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  TASK_BRANCH=<branch>
  WORKTREE=<path>
  HEAD=<sha>
  COMMITS=<count>
  CHECKS=<what ran / verified>
  SMOKE_RUN=<run id/url if executed>

Notes:
  - Run this script from inside the task worktree.
  - This script does NOT delete the worktree (feature_cleanup removes worktrees at feature end).
  - Integration tasks merge back to orchestration only when tasks.json sets `merge_to_orchestration=true`.
    - If the orchestration branch is behind the integration branch, this will fast-forward.
    - If the orchestration branch has advanced (typically due to docs/status commits), this will create a merge commit while preserving the orchestration branch’s Planning Pack files under the feature dir.
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
}

log() {
    echo "== $*" >&2
}

require_cmd() {
    if ! command -v "$1" >/dev/null 2>&1; then
        die "Missing dependency: $1"
    fi
}

python_read_json() {
    python3 - "$1" <<'PY'
import json
import sys

with open(sys.argv[1], "r", encoding="utf-8") as f:
    print(json.dumps(json.load(f)))
PY
}

python_abs_path() {
    python3 - "$1" <<'PY'
import os
import sys

p = sys.argv[1]
if os.path.isabs(p):
    print(os.path.realpath(p))
else:
    print(os.path.realpath(os.path.join(os.getcwd(), p)))
PY
}

TASK_ID=""
VERIFY_ONLY=0
NO_COMMIT=0
ALLOW_UNPLANNED_TOUCH=0
OVERRIDE_REASON=""
PLATFORM=""
SMOKE=0
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --task-id)
            TASK_ID="${2:-}"
            shift 2
            ;;
        --verify-only)
            VERIFY_ONLY=1
            shift 1
            ;;
        --no-commit)
            NO_COMMIT=1
            shift 1
            ;;
        --allow-unplanned-touch)
            ALLOW_UNPLANNED_TOUCH=1
            shift 1
            ;;
        --reason)
            OVERRIDE_REASON="${2:-}"
            shift 2
            ;;
        --platform)
            PLATFORM="${2:-}"
            shift 2
            ;;
        --smoke)
            SMOKE=1
            shift 1
            ;;
        --dry-run)
            DRY_RUN=1
            shift 1
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            die "Unknown arg: $1"
            ;;
    esac
done

if [[ -z "${TASK_ID}" ]]; then
    usage >&2
    die "Missing --task-id"
fi

if [[ "${ALLOW_UNPLANNED_TOUCH}" -eq 1 ]]; then
    if [[ -z "${OVERRIDE_REASON}" ]]; then
        die "--allow-unplanned-touch requires --reason \"<text>\""
    fi
    if rg -nq '[\r\n]' <<<"${OVERRIDE_REASON}"; then
        die "--reason must be a single line (no newlines)"
    fi
fi

case "${PLATFORM}" in
    ""|linux|macos|windows|wsl) ;;
    *)
        die "Invalid --platform: ${PLATFORM}"
        ;;
esac

require_cmd git
require_cmd jq
require_cmd python3
require_cmd rg

WORKTREE_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not inside a git worktree"
cd "${WORKTREE_ROOT}"

TASKMETA_PATH="${WORKTREE_ROOT}/.taskmeta.json"
if [[ ! -f "${TASKMETA_PATH}" ]]; then
    die "Missing .taskmeta.json in worktree root: ${TASKMETA_PATH} (run task_start first)"
fi

TASKMETA_JSON="$(python_read_json "${TASKMETA_PATH}")"

META_TASK_ID="$(jq -r '.task_id // empty' <<<"${TASKMETA_JSON}")"
if [[ "${META_TASK_ID}" != "${TASK_ID}" ]]; then
    die "Task id mismatch: --task-id=${TASK_ID} but .taskmeta.json has task_id=${META_TASK_ID}"
fi

REPO_ROOT="$(git rev-parse --show-superproject-working-tree 2>/dev/null || true)"
if [[ -z "${REPO_ROOT}" ]]; then
    # For a normal (non-submodule) worktree, the git root is the worktree root; locate common repo root via git.
    REPO_ROOT="$(git rev-parse --show-toplevel)"
fi

PM_SYSTEM_SCRIPTS_DIR="${REPO_ROOT}/docs/project_management/system/scripts"
PLANNING_SCRIPTS_DIR="${PM_SYSTEM_SCRIPTS_DIR}/planning"

FEATURE_DIR_RELPATH="$(jq -r '.feature_dir // empty' <<<"${TASKMETA_JSON}")"
ORCH_BRANCH="$(jq -r '.orchestration_branch // empty' <<<"${TASKMETA_JSON}")"
TASK_BRANCH="$(jq -r '.task_branch // empty' <<<"${TASKMETA_JSON}")"
CREATED_FROM_SHA="$(jq -r '.created_from_sha // empty' <<<"${TASKMETA_JSON}")"

if [[ -z "${FEATURE_DIR_RELPATH}" || -z "${ORCH_BRANCH}" || -z "${TASK_BRANCH}" || -z "${CREATED_FROM_SHA}" ]]; then
    die ".taskmeta.json is missing required fields (feature_dir/orchestration_branch/task_branch/created_from_sha)"
fi

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR_RELPATH}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json referenced by .taskmeta.json: ${TASKS_JSON}"
fi

SCHEMA_VERSION="$(jq -r '.meta.schema_version // 1' "${TASKS_JSON}")"
AUTOMATION_ENABLED="$(jq -r '.meta.automation.enabled // false' "${TASKS_JSON}")"
if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "task_finish requires automation-enabled planning pack (meta.schema_version>=3 and meta.automation.enabled=true)"
fi

SLICE_SPEC_VERSION="$(jq -r '.meta.slice_spec_version | tonumber? // 0' "${TASKS_JSON}")"

TASK_JSON="$(jq -c --arg id "${TASK_ID}" '.tasks[] | select(.id==$id)' "${TASKS_JSON}")" || true
if [[ -z "${TASK_JSON}" ]]; then
    die "Task not found in tasks.json: ${TASK_ID}"
fi

TASK_TYPE="$(jq -r '.type' <<<"${TASK_JSON}")"
REQUIRED_TARGETS="$(jq -r '.required_make_targets // [] | join(" ")' <<<"${TASK_JSON}")"
TASK_PLATFORM="$(jq -r '.platform // empty' <<<"${TASK_JSON}")"
MERGE_TO_ORCH="$(jq -r 'if has("merge_to_orchestration") then .merge_to_orchestration else empty end' <<<"${TASK_JSON}")"
FEATURE_NAME="$(jq -r '.meta.feature // empty' "${TASKS_JSON}")"
if [[ -z "${FEATURE_NAME}" ]]; then
    die "tasks.json meta.feature is required for automation packs"
fi

if [[ "${TASK_TYPE}" == "integration" ]]; then
    case "${MERGE_TO_ORCH}" in
        true|false) ;;
        *)
            die "tasks.json integration tasks must set merge_to_orchestration to true/false (task ${TASK_ID})"
            ;;
    esac
fi

GIT_COMMON_DIR="$(python_abs_path "$(git rev-parse --git-common-dir)")"
REGISTRY_ABS="${GIT_COMMON_DIR}/triad/features/${FEATURE_NAME}/worktrees.json"

if [[ -n "${TASK_PLATFORM}" && -n "${PLATFORM}" && "${TASK_PLATFORM}" != "${PLATFORM}" ]]; then
    die "Platform mismatch: task.platform=${TASK_PLATFORM} but --platform=${PLATFORM}"
fi
if [[ -z "${PLATFORM}" && -n "${TASK_PLATFORM}" ]]; then
    PLATFORM="${TASK_PLATFORM}"
fi

current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${current_branch}" != "${TASK_BRANCH}" ]]; then
    die "Expected to be on task branch ${TASK_BRANCH}, but current branch is ${current_branch}"
fi

pm_roots_json="$(python3 "${PLANNING_SCRIPTS_DIR}/pm_paths.py" print-roots 2>/dev/null)" || die "Failed to resolve PM roots (pm_paths.py print-roots)"
PM_ROOT="$(jq -r '.pm_root' <<<"${pm_roots_json}")"
PM_PACKS_ROOT="$(jq -r '.pm_packs_root' <<<"${pm_roots_json}")"

PM_PACKS_PREFIX="${PM_PACKS_ROOT%/}/"

is_planning_path() {
    local p="$1"
    [[ -z "${p}" ]] && return 1
    if [[ "${p}" == "${PM_PACKS_PREFIX}"* ]]; then
        return 0
    fi
    return 1
}

guard_no_planning_doc_edits() {
    local p
    while IFS= read -r p; do
        if is_planning_path "${p}"; then
            die "Worktree contains changes under planning roots (${PM_PACKS_PREFIX}): ${p} (do not edit planning docs inside the worktree; move these edits to the orchestration branch)"
        fi
    done < <(git diff --name-only)

    while IFS= read -r p; do
        if is_planning_path "${p}"; then
            die "Worktree contains staged changes under planning roots (${PM_PACKS_PREFIX}): ${p} (do not edit planning docs inside the worktree)"
        fi
    done < <(git diff --name-only --cached)

    while IFS= read -r p; do
        if is_planning_path "${p}"; then
            die "Worktree contains untracked files under planning roots (${PM_NEXT_PREFIX} or ${PM_PACKS_PREFIX}): ${p} (do not edit planning docs inside the worktree; move these edits to the orchestration branch)"
        fi
    done < <(git ls-files --others --exclude-standard)
}

impact_map_touchset_status="unknown"
impact_map_source="unknown"
impact_map_source_worktree=""
impact_map_override_reason=""

collect_touched_paths() {
    local base_sha="$1"
    local mb
    mb="$(git merge-base "${base_sha}" HEAD 2>/dev/null)" || die "Could not compute merge-base for created_from_sha=${base_sha}"

    python3 - "${mb}" <<'PY'
from __future__ import annotations

import subprocess
import sys

mb = sys.argv[1]


def run_bytes(args: list[str]) -> bytes:
    p = subprocess.run(args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    if p.returncode != 0:
        msg = p.stderr.decode("utf-8", errors="replace").strip()
        raise SystemExit(f"FAIL: git command failed: {' '.join(args)}: {msg}")
    return p.stdout


def parse_name_status_z(payload: bytes) -> set[str]:
    tokens = payload.split(b"\0")
    if tokens and tokens[-1] == b"":
        tokens.pop()

    touched: set[str] = set()
    i = 0
    while i < len(tokens):
        status = tokens[i].decode("utf-8", errors="replace")
        i += 1

        if not status:
            continue

        if status.startswith("R"):
            if i + 1 >= len(tokens):
                raise SystemExit("FAIL: unexpected rename record shape in git --name-status -z output")
            old = tokens[i].decode("utf-8", errors="replace")
            new = tokens[i + 1].decode("utf-8", errors="replace")
            i += 2
            if old:
                touched.add(old)
            if new:
                touched.add(new)
            continue

        if status.startswith("C"):
            if i + 1 >= len(tokens):
                raise SystemExit("FAIL: unexpected copy record shape in git --name-status -z output")
            _old = tokens[i].decode("utf-8", errors="replace")
            new = tokens[i + 1].decode("utf-8", errors="replace")
            i += 2
            if new:
                touched.add(new)
            continue

        if i >= len(tokens):
            raise SystemExit("FAIL: unexpected record shape in git --name-status -z output")
        path = tokens[i].decode("utf-8", errors="replace")
        i += 1
        if path:
            touched.add(path)

    return touched


def parse_paths_z(payload: bytes) -> set[str]:
    tokens = payload.split(b"\0")
    if tokens and tokens[-1] == b"":
        tokens.pop()
    return {t.decode("utf-8", errors="replace") for t in tokens if t}


touched: set[str] = set()
touched |= parse_name_status_z(run_bytes(["git", "diff", "--name-status", "-z", "-M", f"{mb}..HEAD"]))
touched |= parse_name_status_z(run_bytes(["git", "diff", "--name-status", "-z", "-M", "--cached"]))
touched |= parse_name_status_z(run_bytes(["git", "diff", "--name-status", "-z", "-M"]))
touched |= parse_paths_z(run_bytes(["git", "ls-files", "-z", "--others", "--exclude-standard"]))

touched.discard(".taskmeta.json")

for p in sorted(touched):
    print(p)
PY
}

enforce_impact_map_touchset() {
    if [[ "${SLICE_SPEC_VERSION}" -lt 2 ]]; then
        impact_map_touchset_status="skipped"
        impact_map_source="legacy"
        impact_map_source_worktree=""
        log "WARN: impact_map touch-set enforcement disabled (meta.slice_spec_version < 2)."
        return 0
    fi

    orch_wt="$(find_single_orch_worktree_or_die)"
    impact_map_source="orchestration"
    impact_map_source_worktree="${orch_wt}"

    local allow_json
    if ! allow_json="$(cd "${orch_wt}" && python3 "${PLANNING_SCRIPTS_DIR}/validate_impact_map.py" --feature-dir "${FEATURE_DIR_RELPATH}" --emit-json)"; then
        die "impact_map Touch Set validation failed on orchestration branch; fix ${FEATURE_DIR_RELPATH}/impact_map.md in ${ORCH_BRANCH} before completing the task"
    fi

    allow_exact_text="$(jq -r '.create[]?, .edit[]?, .deprecate[]?, .delete[]?' <<<"${allow_json}")"

    allow_prefixes=()
    while IFS= read -r prefix; do
        [[ -z "${prefix}" ]] && continue
        allow_prefixes+=("${prefix}")
    done < <(jq -r '.dir_prefixes[]?' <<<"${allow_json}")

    touched_paths=()
    while IFS= read -r p; do
        [[ -z "${p}" ]] && continue
        touched_paths+=("${p}")
    done < <(collect_touched_paths "${CREATED_FROM_SHA}")

    if [[ "${SUBSTRATE_TASK_FINISH_TOUCH_DEBUG:-0}" == "1" ]]; then
        log "DEBUG: touched paths since created_from_sha=${CREATED_FROM_SHA} (deduped):"
        for p in "${touched_paths[@]}"; do
            printf '%s\n' "${p}" >&2
        done
    fi

    allow_path() {
        local p="$1"
        if grep -Fxq "${p}" <<<"${allow_exact_text}"; then
            return 0
        fi
        local prefix
        for prefix in "${allow_prefixes[@]}"; do
            [[ -z "${prefix}" ]] && continue
            if [[ "${p}" == "${prefix}"* ]]; then
                return 0
            fi
        done
        return 1
    }

    unplanned=()
    local p
    for p in "${touched_paths[@]}"; do
        if ! allow_path "${p}"; then
            unplanned+=("${p}")
        fi
    done

    if [[ "${#unplanned[@]}" -eq 0 ]]; then
        impact_map_touchset_status="enforced"
        return 0
    fi

    unplanned_sorted=("${unplanned[@]}")

    echo "FAIL: unplanned file touches detected (${#unplanned_sorted[@]})" >&2
    for p in "${unplanned_sorted[@]}"; do
        echo "- ${p}" >&2
    done
    echo "" >&2
    echo "To proceed:" >&2
    echo "1) In orchestration worktree (${orch_wt}) on branch ${ORCH_BRANCH}: update ${FEATURE_DIR_RELPATH}/impact_map.md to include these paths (Create/Edit/Deprecate/Delete)." >&2
    echo "2) Commit planning docs update." >&2
    echo "3) Re-run: make triad-task-finish TASK_ID=\"${TASK_ID}\"" >&2

    if [[ "${ALLOW_UNPLANNED_TOUCH}" -eq 1 ]]; then
        impact_map_touchset_status="overridden"
        impact_map_override_reason="${OVERRIDE_REASON}"
        log "WARN: proceeding due to --allow-unplanned-touch (reason: ${OVERRIDE_REASON})"
        return 0
    fi

    exit 1
}

run_make_targets() {
    if [[ -z "${REQUIRED_TARGETS}" ]]; then
        log "No required_make_targets configured for ${TASK_ID}; skipping checks"
        return 0
    fi
    for t in ${REQUIRED_TARGETS}; do
        log "Running: make ${t}"
        if [[ "${DRY_RUN}" -eq 1 ]]; then
            continue
        fi
        make "${t}" 1>&2
    done
}

commit_changes() {
    if [[ "${NO_COMMIT}" -eq 1 ]]; then
        log "--no-commit set; skipping commit"
        return 0
    fi

    guard_no_planning_doc_edits

    if git status --porcelain | rg -q '.'; then
        log "Committing worktree changes"
        if [[ "${DRY_RUN}" -eq 1 ]]; then
            return 0
        fi
        git add -A
        if git diff --cached --quiet; then
            log "No staged changes to commit"
            return 0
        fi
        git commit -m "task: ${TASK_ID}" 1>&2
    else
        log "No changes to commit"
    fi
}

find_orch_worktree() {
    git worktree list --porcelain | awk -v b="refs/heads/${ORCH_BRANCH}" '
        $1=="worktree" { wt=$2 }
        $1=="branch" && $2==b { print wt }
    '
}

find_single_orch_worktree_or_die() {
    local orch_wt=""
    local count=0
    while IFS= read -r wt; do
        [[ -z "${wt}" ]] && continue
        count=$((count + 1))
        if [[ "${count}" -eq 1 ]]; then
            orch_wt="${wt}"
        else
            die "Multiple worktrees have orchestration branch checked out (${ORCH_BRANCH}); cannot safely determine orchestration worktree"
        fi
    done < <(find_orch_worktree)
    if [[ "${count}" -eq 0 ]]; then
        die "Could not find an orchestration worktree with branch ${ORCH_BRANCH} checked out (run triad-orch-ensure or check out ${ORCH_BRANCH} in a worktree)"
    fi
    printf '%s\n' "${orch_wt}"
}

merge_to_orchestration_ff_only() {
    if [[ "${TASK_TYPE}" != "integration" ]]; then
        return 0
    fi
    if [[ "${MERGE_TO_ORCH}" != "true" ]]; then
        log "merge_to_orchestration is not true; skipping merge back to ${ORCH_BRANCH}"
        return 0
    fi

    orch_wt="$(find_single_orch_worktree_or_die)"

    log "Merging ${TASK_BRANCH} -> ${ORCH_BRANCH} in orchestration worktree: ${orch_wt}"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        return 0
    fi

    if ! git -C "${orch_wt}" diff --quiet; then
        die "Orchestration worktree has unstaged changes; expected clean: ${orch_wt}"
    fi
    if ! git -C "${orch_wt}" diff --cached --quiet; then
        die "Orchestration worktree has staged changes; expected clean: ${orch_wt}"
    fi

    git -C "${orch_wt}" checkout "${ORCH_BRANCH}" >/dev/null
    if git -C "${orch_wt}" rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
        git -C "${orch_wt}" pull --ff-only >/dev/null
    fi

    # Fast path: allow a clean FF when possible.
    if git -C "${orch_wt}" merge --ff-only "${TASK_BRANCH}" 1>&2; then
        return 0
    fi

    # Non-FF path: create a merge commit, but always preserve the orchestration branch's Planning Pack
    # files under FEATURE_DIR_RELPATH to avoid conflicts/accidental drift. This is intentionally narrow:
    # code conflicts are not auto-resolved.
    log "Non-FF merge required; creating merge commit while preserving Planning Pack files from ${ORCH_BRANCH}: ${FEATURE_DIR_RELPATH}"
    set +e
    git -C "${orch_wt}" merge --no-ff --no-commit "${TASK_BRANCH}" 1>&2
    merge_rc="$?"
    set -e

    # Always restore feature-dir planning pack files from the orchestration branch (HEAD in orch_wt).
    # This resolves tasks.json/session_log.md conflicts and prevents task branches from overwriting them.
    git -C "${orch_wt}" checkout -q HEAD -- "${FEATURE_DIR_RELPATH}" >/dev/null 2>&1 || true
    git -C "${orch_wt}" add -A -- "${FEATURE_DIR_RELPATH}" >/dev/null 2>&1 || true

    # If conflicts remain, they must be resolved manually (we do not auto-resolve code conflicts).
    conflicts="$(git -C "${orch_wt}" diff --name-only --diff-filter=U || true)"
    if [[ -n "${conflicts}" ]]; then
        git -C "${orch_wt}" merge --abort >/dev/null 2>&1 || true
        echo "Merge back to ${ORCH_BRANCH} has non-planning conflicts and requires human resolution. Conflicts:" >&2
        printf '%s\n' "${conflicts}" >&2
        die "Resolve conflicts on ${ORCH_BRANCH} and re-run task_finish."
    fi

    # If the initial merge failed only due to planning-pack conflicts, we should now be in a clean
    # merge state ready to commit. If it failed for other reasons, committing will fail and surface
    # the underlying problem.
    if [[ "${merge_rc}" -ne 0 ]]; then
        log "Merge had conflicts; Planning Pack restored from ${ORCH_BRANCH}, proceeding to commit merge"
    fi
    git -C "${orch_wt}" commit -m "merge: ${TASK_ID} (${TASK_BRANCH}) -> ${ORCH_BRANCH}" 1>&2
}

update_registry() {
    local registry_abs="${REGISTRY_ABS}"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        return 0
    fi
    python3 - "${registry_abs}" "${TASK_ID}" "${HEAD_SHA}" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone

registry_abs = sys.argv[1]
task_id = sys.argv[2]
head = sys.argv[3]

if not os.path.exists(registry_abs):
    # Registry is best-effort; task_finish still succeeds without it.
    raise SystemExit(0)

with open(registry_abs, "r", encoding="utf-8") as f:
    data = json.load(f)

entries = data.get("entries", [])
now = datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")

for e in entries:
    if e.get("task_id") == task_id:
        e["last_finished_at_utc"] = now
        e["last_head_sha"] = head
        break

data["updated_at_utc"] = now

tmp = registry_abs + ".tmp"
with open(tmp, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2, sort_keys=True)
    f.write("\n")
os.replace(tmp, registry_abs)
PY
}

smoke_run_id=""
run_smoke_if_requested() {
    if [[ "${SMOKE}" -eq 0 ]]; then
        return 0
    fi
    if [[ -z "${PLATFORM}" ]]; then
        die "--smoke requires --platform or tasks.json task.platform"
    fi

    require_cmd gh
    if ! gh auth status -h github.com >/dev/null 2>&1; then
        die "GitHub CLI is not authenticated for github.com (run: gh auth login -h github.com) or provide a token via GH_TOKEN for non-interactive runs"
    fi

    feature_dir_ci="$(python3 -c 'import os,sys; print(os.path.relpath(sys.argv[1], sys.argv[2]))' "${FEATURE_DIR_ABS}" "${REPO_ROOT}")"
    log "Running feature smoke via CI: PLATFORM=${PLATFORM}"
    if [[ "${DRY_RUN}" -eq 1 ]]; then
        smoke_run_id="dry-run"
        return 0
    fi

    out="$(mktemp)"
    smoke_ok=0
    smoke_args=(make feature-smoke FEATURE_DIR="${feature_dir_ci}" PLATFORM="${PLATFORM}" WORKFLOW_REF="${ORCH_BRANCH}" CLEANUP=1)
    if [[ "${PLATFORM}" == "linux" ]]; then
        wsl_required="$(jq -r '.meta.wsl_required // false' "${TASKS_JSON}")"
        wsl_mode="$(jq -r '.meta.wsl_task_mode // "bundled"' "${TASKS_JSON}")"
        if [[ "${wsl_required}" == "true" && "${wsl_mode}" == "bundled" ]]; then
            smoke_args+=(RUN_WSL=1)
        fi
    fi

    if ("${smoke_args[@]}" 2>&1 | tee "${out}" 1>&2); then
        smoke_ok=1
    fi

    # Best-effort parse: "Run: <id>"
    if rg -n '^Run: ' "${out}" >/dev/null 2>&1; then
        smoke_run_id="$(rg -n '^Run: ' "${out}" | tail -n 1 | sed -E 's/^Run: *//')"
    else
        smoke_run_id="unknown"
    fi
    rm -f "${out}"

    if [[ "${smoke_ok}" -ne 1 ]]; then
        die "feature-smoke failed for PLATFORM=${PLATFORM} (SMOKE_RUN=${smoke_run_id})"
    fi
}

if [[ "${VERIFY_ONLY}" -eq 1 ]]; then
    checks_summary="verify-only"
else
    if [[ -z "${REQUIRED_TARGETS}" ]]; then
        checks_summary="none"
    else
        checks_summary="make ${REQUIRED_TARGETS}"
    fi
    run_make_targets
fi

guard_no_planning_doc_edits
enforce_impact_map_touchset

    if [[ "${VERIFY_ONLY}" -eq 1 ]]; then
        HEAD_SHA="$(git rev-parse HEAD)"
        COMMITS_COUNT="$(git rev-list --count "${CREATED_FROM_SHA}..HEAD" || echo 0)"
        smoke_run_id="skipped"

        printf 'TASK_BRANCH=%s\n' "${TASK_BRANCH}"
        printf 'WORKTREE=%s\n' "${WORKTREE_ROOT}"
        printf 'HEAD=%s\n' "${HEAD_SHA}"
        printf 'COMMITS=%s\n' "${COMMITS_COUNT}"
        printf 'CHECKS=%s; impact_map_touchset:%s; impact_map_source:%s\n' "${checks_summary}" "${impact_map_touchset_status}" "${impact_map_source}"
        printf 'IMPACT_MAP_SOURCE=%s\n' "${impact_map_source}"
        printf 'IMPACT_MAP_SOURCE_WORKTREE=%s\n' "${impact_map_source_worktree}"
        if [[ "${impact_map_touchset_status}" == "overridden" ]]; then
            printf 'IMPACT_MAP_OVERRIDE_REASON=%s\n' "${impact_map_override_reason}"
        fi
        printf 'SMOKE_RUN=%s\n' "${smoke_run_id}"
        printf 'MERGED_TO_ORCH=%s\n' "${MERGE_TO_ORCH}"
    exit 0
fi

commit_changes
merge_to_orchestration_ff_only

HEAD_SHA="$(git rev-parse HEAD)"
COMMITS_COUNT="$(git rev-list --count "${CREATED_FROM_SHA}..HEAD" || echo 0)"

run_smoke_if_requested

update_registry

printf 'TASK_BRANCH=%s\n' "${TASK_BRANCH}"
printf 'WORKTREE=%s\n' "${WORKTREE_ROOT}"
printf 'HEAD=%s\n' "${HEAD_SHA}"
printf 'COMMITS=%s\n' "${COMMITS_COUNT}"
printf 'CHECKS=%s; impact_map_touchset:%s; impact_map_source:%s\n' "${checks_summary}" "${impact_map_touchset_status}" "${impact_map_source}"
printf 'IMPACT_MAP_SOURCE=%s\n' "${impact_map_source}"
printf 'IMPACT_MAP_SOURCE_WORKTREE=%s\n' "${impact_map_source_worktree}"
if [[ "${impact_map_touchset_status}" == "overridden" ]]; then
    printf 'IMPACT_MAP_OVERRIDE_REASON=%s\n' "${impact_map_override_reason}"
fi
printf 'SMOKE_RUN=%s\n' "${smoke_run_id}"
printf 'MERGED_TO_ORCH=%s\n' "${MERGE_TO_ORCH}"
