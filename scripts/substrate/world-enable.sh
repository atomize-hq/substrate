#!/usr/bin/env bash
if [[ $- == *x* ]]; then
  set +x
fi
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

PREFIX=""
PREFIX_DECLARED=0
PROFILE="release"
DRY_RUN=0
VERBOSE=0
FORCE=0
SYNC_DEPS=1
NO_SYNC_DEPS_REQUESTED=0
HELP_REQUESTED=0
INTERNAL_CHILD_OPTION_PRESENT=0
for arg in "$@"; do
  if [[ "${arg}" == "--install-bootstrap-context-v1" ]]; then
    INTERNAL_CHILD_OPTION_PRESENT=1
    break
  fi
done

usage() {
  cat <<'USAGE'
Substrate World Enable Helper

Usage: world-enable.sh [options]

Options:
  --home <path>      Substrate home to update (default: current account-database home/.substrate)
  --profile <name>   Provisioning profile label for logging (default: release)
  --dry-run          Show the provisioning commands without executing
  --verbose          Print verbose execution details
  --force            Rerun provisioning even if metadata reports enabled
  --provision-agent-runtime <runtime_family>
                     Enable a world runtime globally (codex only in this slice), then run
                     'substrate world deps current sync' before the helper exits
  --no-sync-deps     Skip 'substrate world deps current sync' after provisioning
  -h, --help         Show this help message
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --home)
      [[ $# -lt 2 ]] && fatal "Missing value for --home"
      [[ "${PREFIX_DECLARED}" -eq 1 ]] && fatal "Duplicate --home"
      [[ -z "$2" ]] && fatal "Empty value for --home"
      PREFIX="$2"
      PREFIX_DECLARED=1
      shift 2
      ;;
    --install-bootstrap-context-v1)
      [[ $# -lt 2 ]] && fatal "Missing value for --install-bootstrap-context-v1"
      [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 1 ]] && fatal "Duplicate --install-bootstrap-context-v1"
      [[ -z "$2" ]] && fatal "Empty value for --install-bootstrap-context-v1"
      INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
      INSTALL_BOOTSTRAP_CONTEXT_DECLARED=1
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
    --provision-agent-runtime)
      [[ $# -lt 2 ]] && fatal "Missing value for --provision-agent-runtime"
      PROVISION_AGENT_RUNTIME="$2"
      shift 2
      ;;
    --no-sync-deps)
      SYNC_DEPS=0
      NO_SYNC_DEPS_REQUESTED=1
      shift
      ;;
    -h|--help)
      if [[ "${INTERNAL_CHILD_OPTION_PRESENT}" -eq 0 ]]; then
        usage
        exit 0
      fi
      HELP_REQUESTED=1
      shift
      ;;
    *)
      fatal "Unknown option: $1"
      ;;
  esac
done

if [[ "${INTERNAL_CHILD_OPTION_PRESENT}" -eq 1 && "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -ne 1 ]]; then
  fatal "Missing value for --install-bootstrap-context-v1"
fi
if [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 1 && "${PREFIX_DECLARED}" -ne 1 ]]; then
  fatal "Internal world-enable child requires --home"
fi
resolve_install_bootstrap_context \
  "${PREFIX_DECLARED}" \
  "${PREFIX}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}"

if [[ "${HELP_REQUESTED}" -eq 1 ]]; then
  usage
  exit 0
fi

validate_agent_runtime_provision_request

if [[ -n "${PROVISION_AGENT_RUNTIME}" && "${NO_SYNC_DEPS_REQUESTED}" -eq 1 ]]; then
  fatal_with_code 2 "--provision-agent-runtime requires 'substrate world deps current sync'. Remove --no-sync-deps or omit the runtime flag."
fi

if [[ ${VERBOSE} -eq 1 ]]; then
  set -x
fi

sanitize_env_path
ORIGINAL_PATH="${PATH}"
detect_platform
prepare_tmpdir

log "world-enable: home=${PREFIX} profile=${PROFILE} force=${FORCE} dry_run=${DRY_RUN}"
log "world-enable: release root located at ${RELEASE_ROOT}"

bin_suffix=""
if [[ "${PLATFORM}" == "windows" ]]; then
  bin_suffix=".exe"
fi

substrate_bin="${PREFIX}/bin/substrate${bin_suffix}"

if [[ ${DRY_RUN} -eq 1 && -n "${PROVISION_AGENT_RUNTIME}" ]]; then
  doctor_path="${PREFIX}/bin:${ORIGINAL_PATH}"
  PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" provision_agent_runtime_world_deps "${substrate_bin}"
  PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" sync_world_deps "${substrate_bin}"
fi

if [[ ${DRY_RUN} -eq 0 ]]; then
  if [[ ! -x "${substrate_bin}" ]]; then
    fatal "substrate binary not found at ${substrate_bin}. Did you install to ${PREFIX}?"
  fi
  primary_user="${INSTALL_BOOTSTRAP_ACCOUNT}"
  bootstrap_private_substrate_home "${substrate_bin}" "${primary_user}"
fi

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

doctor_path="${PREFIX}/bin:${ORIGINAL_PATH}"
PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" run_world_checks "${substrate_bin}"
PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" provision_agent_runtime_world_deps "${substrate_bin}"

if [[ ${SYNC_DEPS} -eq 1 ]]; then
  PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" sync_world_deps "${substrate_bin}"
else
  log "Skipping world deps sync (--no-sync-deps)"
fi

log "World provisioning complete via world-enable helper"
