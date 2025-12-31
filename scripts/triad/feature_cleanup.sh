#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/feature_cleanup.sh --feature-dir <path> [options]

Required:
  --feature-dir <path>               Feature Planning Pack dir (docs/project_management/next/<feature>)

Options:
  --remove-worktrees                Remove all registered task worktrees (git worktree remove)
  --prune-local-branches            Delete registered task branches locally
  --prune-remote-branches <remote>  Delete registered task branches on the remote
  --force                           Required to delete dirty worktrees or branches that are unmerged/unpushed
  --dry-run                          Print what would happen; do not mutate git/worktrees

Stdout contract (machine-parseable):
  REMOVED_WORKTREES=<count>
  PRUNED_LOCAL_BRANCHES=<count>
  PRUNED_REMOTE_BRANCHES=<count>

Notes:
  - This script consumes the deterministic registry created by task_start:
    <git-common-dir>/triad/features/<feature>/worktrees.json
  - This is the only place worktrees should be removed (worktrees are retained throughout the feature).
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

FEATURE_DIR=""
REMOVE_WT=0
PRUNE_LOCAL=0
PRUNE_REMOTE=0
REMOTE=""
FORCE=0
DRY_RUN=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --remove-worktrees)
            REMOVE_WT=1
            shift 1
            ;;
        --prune-local-branches)
            PRUNE_LOCAL=1
            shift 1
            ;;
        --prune-remote-branches)
            PRUNE_REMOTE=1
            REMOTE="${2:-}"
            shift 2
            ;;
        --force)
            FORCE=1
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

if [[ -z "${FEATURE_DIR}" ]]; then
    usage >&2
    die "Missing --feature-dir"
fi
if [[ "${PRUNE_REMOTE}" -eq 1 && -z "${REMOTE}" ]]; then
    die "--prune-remote-branches requires a remote name"
fi

require_cmd git
require_cmd jq
require_cmd python3
require_cmd rg

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi

SCHEMA_VERSION="$(jq -r '.meta.schema_version // 1' "${TASKS_JSON}")"
AUTOMATION_ENABLED="$(jq -r '.meta.automation.enabled // false' "${TASKS_JSON}")"
ORCH_BRANCH="$(jq -r '.meta.automation.orchestration_branch // empty' "${TASKS_JSON}")"
FEATURE_NAME="$(jq -r '.meta.feature // empty' "${TASKS_JSON}")"
if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "feature_cleanup requires automation-enabled planning pack (meta.schema_version>=3 and meta.automation.enabled=true)"
fi
if [[ -z "${ORCH_BRANCH}" || -z "${FEATURE_NAME}" ]]; then
    die "tasks.json meta.automation must include orchestration_branch, and meta.feature must be set"
fi

current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${current_branch}" != "${ORCH_BRANCH}" ]]; then
    die "Run feature_cleanup from the orchestration branch: ${ORCH_BRANCH} (current: ${current_branch})"
fi
if [[ "${DRY_RUN}" -ne 1 ]]; then
    if ! git diff --quiet || ! git diff --cached --quiet; then
        die "Orchestration worktree is not clean; commit/stash before cleanup"
    fi
fi

GIT_COMMON_DIR="$(python_abs_path "$(git rev-parse --git-common-dir)")"
REGISTRY_ABS="${GIT_COMMON_DIR}/triad/features/${FEATURE_NAME}/worktrees.json"

mapfile -t worktrees < <(
    jq -r '.tasks[] | select(.git_worktree? and (.git_worktree | length) > 0) | .git_worktree' "${TASKS_JSON}" | sort -u
)
mapfile -t branches < <(
    jq -r '.tasks[] | select(.git_branch? and (.git_branch | length) > 0) | .git_branch' "${TASKS_JSON}" | sort -u
)

# Best-effort: merge in registry entries (if present + valid) to catch any extra spawned worktrees.
if [[ -f "${REGISTRY_ABS}" ]]; then
    if jq -e . >/dev/null 2>&1 <"${REGISTRY_ABS}"; then
        mapfile -t reg_worktrees < <(jq -r '.entries[]?.worktree // empty' "${REGISTRY_ABS}" | sort -u)
        mapfile -t reg_branches < <(jq -r '.entries[]?.task_branch // empty' "${REGISTRY_ABS}" | sort -u)
        worktrees+=("${reg_worktrees[@]}")
        branches+=("${reg_branches[@]}")
        mapfile -t worktrees < <(printf '%s\n' "${worktrees[@]}" | rg -v '^[[:space:]]*$' | sort -u)
        mapfile -t branches < <(printf '%s\n' "${branches[@]}" | rg -v '^[[:space:]]*$' | sort -u)
    else
        log "Warning: registry exists but is not valid JSON; ignoring registry: ${REGISTRY_ABS}"
    fi
fi

removed_worktrees=0
pruned_local=0
pruned_remote=0

ensure_worktree_clean_or_force() {
    local wt="$1"
    if [[ ! -d "${wt}" ]]; then
        return 0
    fi
    if ! git -C "${wt}" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
        die "Registry worktree is not a git worktree: ${wt}"
    fi
    if git -C "${wt}" status --porcelain | rg -q '.'; then
        if [[ "${FORCE}" -ne 1 ]]; then
            die "Dirty worktree (use --force to remove): ${wt}"
        fi
    fi
}

branch_safe_to_delete_or_force() {
    local branch="$1"
    if [[ "${branch}" == "${ORCH_BRANCH}" ]]; then
        die "Refusing to prune orchestration branch: ${branch}"
    fi

    # Must not be checked out by any worktree.
    if git worktree list --porcelain | awk '$1=="branch"{print $2}' | grep -Fxq "refs/heads/${branch}"; then
        if [[ "${FORCE}" -ne 1 ]]; then
            die "Branch is still checked out by a worktree (remove worktrees first): ${branch}"
        fi
    fi

    # If not merged into orchestration, require --force.
    if ! git merge-base --is-ancestor "${branch}" "${ORCH_BRANCH}" >/dev/null 2>&1; then
        if [[ "${FORCE}" -ne 1 ]]; then
            die "Branch is not merged into ${ORCH_BRANCH} (use --force to delete): ${branch}"
        fi
    fi

    # If upstream exists and local has commits not on upstream, require --force.
    if git rev-parse --verify --quiet "refs/heads/${branch}" >/dev/null 2>&1; then
        if git rev-parse --abbrev-ref --symbolic-full-name "${branch}@{u}" >/dev/null 2>&1; then
            ahead_count="$(git rev-list --right-only --count "${branch}@{u}...${branch}" 2>/dev/null || echo 0)"
            if [[ "${ahead_count}" -gt 0 && "${FORCE}" -ne 1 ]]; then
                    die "Branch has unpushed commits (use --force to delete): ${branch}"
            fi
        else
            # No upstream configured; treat as potentially unpushed.
            if [[ "${FORCE}" -ne 1 ]]; then
                die "Branch has no upstream configured (use --force to delete): ${branch}"
            fi
        fi
    fi
}

if [[ "${REMOVE_WT}" -eq 1 ]]; then
    for wt_rel in "${worktrees[@]}"; do
        [[ -z "${wt_rel}" ]] && continue
        wt_abs="${wt_rel}"
        if [[ "${wt_abs}" != /* ]]; then
            wt_abs="${REPO_ROOT}/${wt_abs}"
        fi
        if [[ ! -d "${wt_abs}" ]]; then
            log "Skipping missing worktree path: ${wt_abs}"
            continue
        fi
        ensure_worktree_clean_or_force "${wt_abs}"
        log "Removing worktree: ${wt_abs}"
        if [[ "${DRY_RUN}" -eq 0 ]]; then
            if [[ "${FORCE}" -eq 1 ]]; then
                git worktree remove --force "${wt_abs}" >/dev/null
            else
                git worktree remove "${wt_abs}" >/dev/null
            fi
        fi
        removed_worktrees=$((removed_worktrees + 1))
    done
fi

if [[ "${PRUNE_LOCAL}" -eq 1 ]]; then
    for br in "${branches[@]}"; do
        [[ -z "${br}" ]] && continue
        if ! git show-ref --verify --quiet "refs/heads/${br}"; then
            continue
        fi
        branch_safe_to_delete_or_force "${br}"
        log "Pruning local branch: ${br}"
        if [[ "${DRY_RUN}" -eq 0 ]]; then
            git branch -D "${br}" >/dev/null
        fi
        pruned_local=$((pruned_local + 1))
    done
fi

if [[ "${PRUNE_REMOTE}" -eq 1 ]]; then
    for br in "${branches[@]}"; do
        [[ -z "${br}" ]] && continue
        # Remote prune is always destructive; require --force if local branch isn't merged.
        if ! git merge-base --is-ancestor "${br}" "${ORCH_BRANCH}" >/dev/null 2>&1; then
            if [[ "${FORCE}" -ne 1 ]]; then
                die "Branch is not merged into ${ORCH_BRANCH}; refusing to delete remote branch without --force: ${br}"
            fi
        fi
        log "Pruning remote branch: ${REMOTE} ${br}"
        if [[ "${DRY_RUN}" -eq 0 ]]; then
            git push "${REMOTE}" ":${br}" >/dev/null
        fi
        pruned_remote=$((pruned_remote + 1))
    done
fi

printf 'REMOVED_WORKTREES=%s\n' "${removed_worktrees}"
printf 'PRUNED_LOCAL_BRANCHES=%s\n' "${pruned_local}"
printf 'PRUNED_REMOTE_BRANCHES=%s\n' "${pruned_remote}"
