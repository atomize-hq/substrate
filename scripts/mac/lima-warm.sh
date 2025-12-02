#!/usr/bin/env bash
set -euo pipefail

PROFILE=scripts/mac/lima/substrate.yaml
PROJECT_PATH=""
CHECK_ONLY=0

usage() {
    cat <<'USAGE'
Usage: scripts/mac/lima-warm.sh [options] [<project-path>]

Options:
  --check-only    Report the current Lima VM status without creating or starting it
  -h, --help      Show this help text

Arguments:
  <project-path>  Repository path to mount inside the Lima VM (default: current directory)
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            if [[ -z "${PROJECT_PATH}" ]]; then
                PROJECT_PATH="$1"
            else
                echo "Unexpected argument: $1" >&2
                usage
                exit 1
            fi
            shift
            ;;
    esac
 done

if [[ -z "${PROJECT_PATH}" ]]; then
    PROJECT_PATH=$(pwd)
fi

if [[ ${CHECK_ONLY} -eq 1 ]]; then
    host_os=$(uname -s)
    if [[ "${host_os}" != "Darwin" ]]; then
        echo "[check-only] Host ${host_os} does not support Lima; skipping warm check."
        exit 0
    fi
    if ! command -v limactl >/dev/null 2>&1; then
        echo "[check-only] limactl not found; skipping warm check."
        exit 0
    fi
    if ! command -v jq >/dev/null 2>&1; then
        echo "[check-only] jq not found; skipping warm check."
        exit 0
    fi
    if limactl list substrate >/dev/null 2>&1; then
        status=$(limactl list substrate --json | jq -r '.status // "unknown"')
        echo "[check-only] Lima VM 'substrate' status: ${status}"
    else
        echo "[check-only] Lima VM 'substrate' not found."
    fi
    exit 0
fi

TMP_PROFILE=$(mktemp)
trap 'rm -f "$TMP_PROFILE"' EXIT

PROJECT="${PROJECT_PATH}" envsubst < "${PROFILE}" > "${TMP_PROFILE}"

if limactl list substrate >/dev/null 2>&1; then
    status=$(limactl list substrate --json | jq -r '.status // ""')
    if [ "$status" = "Running" ]; then
        echo "Lima VM 'substrate' is already running."
        exit 0
    else
        echo "Starting existing Lima VM 'substrate'..."
        limactl start substrate
    fi
else
    echo "Creating new Lima VM 'substrate' from profile..."
    limactl start --tty=false --name substrate "$TMP_PROFILE"
fi

# Wait until running
timeout=120
echo "Waiting for VM to reach Running state..."
while [ $timeout -gt 0 ]; do
    status=$(limactl list substrate --json | jq -r '.status // ""')
    if [ "$status" = "Running" ]; then
        echo "Lima VM 'substrate' is running."
        exit 0
    fi
    sleep 2
    timeout=$((timeout-2))
done

echo "ERROR: Lima VM did not reach Running state within 120 seconds" >&2
exit 1
