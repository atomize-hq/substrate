#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/task_finish.sh --task-id <id> [options]

Required:
  --task-id <id>         Task id (must match .taskmeta.json in this worktree)

Options:
  --verify-only          Do not run checks; only validate invariants and print summary
  --no-commit            Do not create a commit (for investigation-only runs)
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

guard_no_planning_doc_edits() {
    if git diff --name-only | rg -q "^docs/project_management/next/"; then
        die "Worktree contains changes under docs/project_management/next/ (do not edit planning docs inside the worktree; move these edits to the orchestration branch)"
    fi
    if git diff --name-only --cached | rg -q "^docs/project_management/next/"; then
        die "Worktree contains staged changes under docs/project_management/next/ (do not edit planning docs inside the worktree)"
    fi
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

merge_to_orchestration_ff_only() {
    if [[ "${TASK_TYPE}" != "integration" ]]; then
        return 0
    fi
    if [[ "${MERGE_TO_ORCH}" != "true" ]]; then
        log "merge_to_orchestration is not true; skipping merge back to ${ORCH_BRANCH}"
        return 0
    fi

    mapfile -t orch_matches < <(find_orch_worktree)
    if [[ "${#orch_matches[@]}" -gt 1 ]]; then
        die "Multiple worktrees have orchestration branch checked out (${ORCH_BRANCH}); cannot safely merge"
    fi
    if [[ "${#orch_matches[@]}" -eq 0 ]]; then
        die "Could not find an orchestration worktree for branch ${ORCH_BRANCH} (run from a repo where ${ORCH_BRANCH} is checked out)"
    fi
    orch_wt="${orch_matches[0]}"

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
printf 'CHECKS=%s\n' "${checks_summary}"
printf 'SMOKE_RUN=%s\n' "${smoke_run_id}"
printf 'MERGED_TO_ORCH=%s\n' "${MERGE_TO_ORCH}"
