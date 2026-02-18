#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'USAGE'
Usage:
  codex_pidfiles.sh --feature-dir <path> [options]

Required:
  --feature-dir <path>   Feature Planning Pack dir (docs/project_management/packs/active/<feature> or equivalent)

Options:
  --kill                 Send a signal to any live PIDs referenced by codex.pid files
  --signal <sig>         Signal for --kill (default: TERM)
  --cleanup-dead         Remove codex.pid files whose PIDs are not running

Behavior:
  - Scans <feature_dir>/logs/**/codex.pid
  - Prints one line per pidfile:
      PIDFILE=<path> PID=<pid> LIVE=0|1 CMD=<cmdline or empty>
USAGE
}

die() {
    echo "ERROR: $*" >&2
    exit 2
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
DO_KILL=0
SIGNAL="TERM"
CLEANUP_DEAD=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --feature-dir)
            FEATURE_DIR="${2:-}"
            shift 2
            ;;
        --kill)
            DO_KILL=1
            shift 1
            ;;
        --signal)
            SIGNAL="${2:-}"
            shift 2
            ;;
        --cleanup-dead)
            CLEANUP_DEAD=1
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

require_cmd python3
require_cmd git
require_cmd ps
require_cmd find

REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)" || die "Not in a git repo"
cd "${REPO_ROOT}"

FEATURE_DIR_ABS="$(python_abs_path "${FEATURE_DIR}")"
LOGS_DIR="${FEATURE_DIR_ABS}/logs"
if [[ ! -d "${LOGS_DIR}" ]]; then
    die "Missing logs dir: ${LOGS_DIR}"
fi

while IFS= read -r pidfile; do
    pid="$(tr -d '[:space:]' < "${pidfile}" || true)"
    live=0
    cmd=""
    if [[ -n "${pid}" ]] && kill -0 "${pid}" 2>/dev/null; then
        live=1
        cmd="$(ps -p "${pid}" -o cmd= 2>/dev/null || true)"
    else
        if [[ "${CLEANUP_DEAD}" -eq 1 ]]; then
            rm -f "${pidfile}" >/dev/null 2>&1 || true
        fi
    fi

    printf 'PIDFILE=%s PID=%s LIVE=%s CMD=%s\n' "${pidfile}" "${pid:-}" "${live}" "${cmd}"

    if [[ "${DO_KILL}" -eq 1 && "${live}" -eq 1 ]]; then
        kill "-${SIGNAL}" "${pid}" 2>/dev/null || true
    fi
done < <(find "${LOGS_DIR}" -type f -name codex.pid 2>/dev/null | sort)
