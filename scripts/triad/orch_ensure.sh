#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  scripts/triad/orch_ensure.sh --feature-dir <path> [--from-branch <branch>] [--dry-run]

Behavior:
  - Ensures the feature orchestration branch exists locally and is checked out.
  - If missing locally but present on remote: fetches and sets upstream tracking.
  - If missing both locally and remote: creates the branch from <branch> (default: testing) and pushes (-u).

Constraints:
  - Requires an automation-enabled planning pack (tasks.json meta.schema_version >= 3 and meta.automation.enabled=true).
  - Refuses to operate if the current worktree has uncommitted changes (unless --dry-run).

Stdout:
  ORCH_BRANCH=<branch>
  REMOTE=<remote>
  ACTION=<what happened>
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

select_remote() {
    if git remote get-url origin >/dev/null 2>&1; then
        echo "origin"
        return 0
    fi
    first="$(git remote | sort | head -n 1)"
    if [[ -n "${first}" ]]; then
        echo "${first}"
        return 0
    fi
    echo ""
}

remote_branch_exists() {
    local remote="$1"
    local branch="$2"
    git ls-remote --exit-code --heads "${remote}" "${branch}" >/dev/null 2>&1
}

branch_checked_out_elsewhere() {
    local branch="$1"
    local current_wt
    current_wt="$(pwd)"
    git worktree list --porcelain | awk -v b="refs/heads/${branch}" -v cwd="${current_wt}" '
        $1=="worktree"{wt=$2}
        $1=="branch" && $2==b && wt!=cwd {print wt}
    ' | head -n 1
}

FEATURE_DIR=""
DRY_RUN=0
FROM_BRANCH="testing"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --from-branch)
            FROM_BRANCH="${2:-}"
            shift 2
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
if [[ -z "${FROM_BRANCH}" ]]; then
    usage >&2
    die "Missing --from-branch value"
fi

require_cmd git
require_cmd jq
require_cmd python3

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

if [[ "${DRY_RUN}" -ne 1 ]]; then
    if ! git diff --quiet || ! git diff --cached --quiet; then
        die "Working tree is not clean; commit/stash before orchestration setup"
    fi
fi

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
TASKS_JSON="${FEATURE_DIR_ABS}/tasks.json"
if [[ ! -f "${TASKS_JSON}" ]]; then
    die "Missing tasks.json: ${TASKS_JSON}"
fi

SCHEMA_VERSION="$(jq -r '.meta.schema_version // 1' "${TASKS_JSON}")"
AUTOMATION_ENABLED="$(jq -r '.meta.automation.enabled // false' "${TASKS_JSON}")"
ORCH_BRANCH="$(jq -r '.meta.automation.orchestration_branch // empty' "${TASKS_JSON}")"
if [[ "${SCHEMA_VERSION}" -lt 3 || "${AUTOMATION_ENABLED}" != "true" ]]; then
    die "orch_ensure requires tasks.json meta.schema_version>=3 and meta.automation.enabled=true"
fi
if [[ -z "${ORCH_BRANCH}" ]]; then
    die "tasks.json meta.automation.orchestration_branch must be set"
fi

other_wt="$(branch_checked_out_elsewhere "${ORCH_BRANCH}")"
if [[ -n "${other_wt}" ]]; then
    die "Orchestration branch ${ORCH_BRANCH} is already checked out in another worktree: ${other_wt}"
fi

remote="$(select_remote)"
if [[ -z "${remote}" ]]; then
    die "No git remotes configured; cannot set up orchestration branch ${ORCH_BRANCH}"
fi

resolve_from_ref() {
    local remote="$1"
    local from="$2"

    if git rev-parse --verify --quiet "${remote}/${from}^{commit}" >/dev/null 2>&1; then
        echo "${remote}/${from}"
        return 0
    fi
    if git rev-parse --verify --quiet "${from}^{commit}" >/dev/null 2>&1; then
        echo "${from}"
        return 0
    fi
    return 1
}

action="noop"
if [[ "$(git rev-parse --abbrev-ref HEAD)" != "${ORCH_BRANCH}" ]]; then
    if git show-ref --verify --quiet "refs/heads/${ORCH_BRANCH}"; then
        log "Checking out orchestration branch: ${ORCH_BRANCH}"
        action="checkout_local"
        if [[ "${DRY_RUN}" -ne 1 ]]; then
            git checkout "${ORCH_BRANCH}" >/dev/null
        fi
    elif remote_branch_exists "${remote}" "${ORCH_BRANCH}"; then
        log "Fetching orchestration branch from ${remote}: ${ORCH_BRANCH}"
        action="fetch_checkout_remote"
        if [[ "${DRY_RUN}" -ne 1 ]]; then
            git fetch "${remote}" "${ORCH_BRANCH}:${ORCH_BRANCH}" >/dev/null
            git branch --set-upstream-to "${remote}/${ORCH_BRANCH}" "${ORCH_BRANCH}" >/dev/null 2>&1 || true
            git checkout "${ORCH_BRANCH}" >/dev/null
        fi
    else
        log "Creating orchestration branch from ${FROM_BRANCH} and pushing: ${ORCH_BRANCH}"
        action="create_from_branch_push"
        if [[ "${DRY_RUN}" -ne 1 ]]; then
            git fetch "${remote}" --prune >/dev/null
            from_ref="$(resolve_from_ref "${remote}" "${FROM_BRANCH}")" || die "Base branch not found (local or ${remote}): ${FROM_BRANCH}"
            if [[ "${from_ref}" == "${remote}/"* ]]; then
                git fetch "${remote}" "${FROM_BRANCH}" >/dev/null
            fi
            git checkout -b "${ORCH_BRANCH}" "${from_ref}" >/dev/null
            git push -u "${remote}" "${ORCH_BRANCH}" >/dev/null
        fi
    fi
fi

if [[ "${DRY_RUN}" -ne 1 ]]; then
    if git rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
        git pull --ff-only >/dev/null || die "Failed ff-only pull on ${ORCH_BRANCH}"
    fi
fi

printf 'ORCH_BRANCH=%s\n' "${ORCH_BRANCH}"
printf 'REMOTE=%s\n' "${remote}"
printf 'ACTION=%s\n' "${action}"
