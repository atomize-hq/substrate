#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || {
    echo "ERROR: failed to locate repo root via git" >&2
    exit 2
}

PM_SYSTEM_ROOT="${PM_SYSTEM_ROOT:-docs/project_management/system}"
if [[ "${PM_SYSTEM_ROOT}" != /* ]]; then
    PM_SYSTEM_ROOT="${REPO_ROOT}/${PM_SYSTEM_ROOT}"
fi

exec "${PM_SYSTEM_ROOT}/scripts/triad/task_start_pair.sh" "$@"
