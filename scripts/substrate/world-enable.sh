#!/usr/bin/env bash
set -euo pipefail

# shellcheck disable=SC2034
INSTALLER_NAME="substrate-world-enable"
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
RELEASE_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)

if [[ ! -f "${SCRIPT_DIR}/install-substrate.sh" ]]; then
  echo "fatal: install-substrate.sh not found next to world-enable helper" >&2
  exit 1
fi

# Reuse installer helpers (log/run_cmd/provision functions)
source "${SCRIPT_DIR}/install-substrate.sh"

PREFIX="${HOME}/.substrate"
PROFILE="release"
DRY_RUN=0
VERBOSE=0
FORCE=0
SYNC_DEPS=1

usage() {
  cat <<'USAGE'
Substrate World Enable Helper

Usage: world-enable.sh [options]

Options:
  --prefix <path>    Installation prefix to update (default: ~/.substrate)
  --profile <name>   Provisioning profile label for logging (default: release)
  --dry-run          Show the provisioning commands without executing
  --verbose          Print verbose execution details
  --force            Rerun provisioning even if metadata reports enabled
  --no-sync-deps     Skip 'substrate world deps sync' after provisioning
  -h, --help         Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -lt 2 ]] && fatal "Missing value for --prefix"
      PREFIX="$2"
      shift 2
      ;;
    --profile)
      [[ $# -lt 2 ]] && fatal "Missing value for --profile"
      PROFILE="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    --verbose)
      VERBOSE=1
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    --no-sync-deps)
      SYNC_DEPS=0
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fatal "Unknown option: $1"
      ;;
  esac
done

if [[ ${VERBOSE} -eq 1 ]]; then
  set -x
fi

sanitize_env_path
ORIGINAL_PATH="${PATH}"
detect_platform
prepare_tmpdir

log "world-enable: prefix=${PREFIX} profile=${PROFILE} force=${FORCE} dry_run=${DRY_RUN}"
log "world-enable: release root located at ${RELEASE_ROOT}"

case "${PLATFORM}" in
  macos)
    ensure_macos_prereqs
    provision_macos_world "${RELEASE_ROOT}"
    ;;
  linux)
    ensure_linux_prereqs
    provision_linux_world "${RELEASE_ROOT}"
    ;;
  windows)
    fatal "substrate world enable is not yet supported on Windows"
    ;;
  *)
    fatal "Unsupported platform: ${PLATFORM}"
    ;;
 esac

if [[ ${DRY_RUN} -eq 1 ]]; then
  log "world-enable dry run complete"
  exit 0
fi

bin_suffix=""
if [[ "${PLATFORM}" == "windows" ]]; then
  bin_suffix=".exe"
fi

substrate_bin="${PREFIX}/bin/substrate${bin_suffix}"
if [[ ! -x "${substrate_bin}" ]]; then
  fatal "substrate binary not found at ${substrate_bin}. Did you install to ${PREFIX}?"
fi

doctor_path="${PREFIX}/bin:${ORIGINAL_PATH}"
PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"

PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" sync_world_deps "${substrate_bin}"

log "World provisioning complete via world-enable helper"
