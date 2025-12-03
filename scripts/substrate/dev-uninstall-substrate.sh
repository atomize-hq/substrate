#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-uninstall-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Dev Uninstaller

Removes development shims and helper files produced by dev-install-substrate.sh.

Usage:
  dev-uninstall-substrate.sh [--prefix <path>] [--profile <debug|release>] [--bin <path>] [--version-label <name>]
  dev-uninstall-substrate.sh --help

Options:
  --prefix <path>        Installation prefix that was used during dev install (default: ~/.substrate)
  --profile <name>       Cargo profile whose binary should be used for shim removal
  --bin <path>           Explicit path to substrate binary to invoke for shim removal
  --version-label <name> Version directory label used during dev install (default: dev)
  --remove-world-service Remove the Linux world-agent systemd service (requires sudo)
  --help                 Show this message

If neither --profile nor --bin is provided the script will look for
`target/release/substrate` first, then `target/debug/substrate`.
USAGE
}

detect_invoking_user() {
  if [[ -n "${SUDO_USER:-}" ]]; then
    printf '%s\n' "${SUDO_USER}"
    return
  fi
  if [[ -n "${USER:-}" ]]; then
    printf '%s\n' "${USER}"
    return
  fi
  if command -v id >/dev/null 2>&1; then
    id -un 2>/dev/null || true
    return
  fi
  printf ''
}

user_in_group() {
  local target_user="$1"
  local target_group="$2"
  if [[ -z "${target_user}" || -z "${target_group}" ]]; then
    return 1
  fi
  if id -nG "${target_user}" 2>/dev/null | tr ' ' '\n' | grep -qx "${target_group}"; then
    return 0
  fi
  return 1
}

print_linger_cleanup_notice() {
  local target_user="$1"
  if [[ "$(uname -s)" != "Linux" ]]; then
    return
  fi
  if [[ -z "${target_user}" || "${target_user}" == "root" ]]; then
    cat <<'MSG'
[dev-uninstall-substrate] loginctl: Unable to detect which user enabled lingering.
Disable lingering manually if socket activation is no longer needed:
  loginctl disable-linger <user>
MSG
    return
  fi
  if ! command -v loginctl >/dev/null 2>&1; then
    return
  fi

  local linger_state
  linger_state="$(loginctl show-user "${target_user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
  if [[ "${linger_state}" == "yes" ]]; then
    cat <<MSG
[dev-uninstall-substrate] loginctl reports lingering is still enabled for ${target_user}.
Disable it if you no longer need socket-activated services:
  loginctl disable-linger ${target_user}
MSG
  fi
}

print_group_cleanup_notice() {
  local target_user="$1"
  if [[ "$(uname -s)" != "Linux" ]]; then
    return
  fi
  if ! getent group substrate >/dev/null 2>&1; then
    return
  fi
  if [[ -n "${target_user}" && "${target_user}" != "root" && user_in_group "${target_user}" substrate ]]; then
    cat <<MSG
[dev-uninstall-substrate] ${target_user} still belongs to the 'substrate' group.
If you are done debugging, remove the membership and delete the group if unused:
  sudo gpasswd -d ${target_user} substrate
  sudo groupdel substrate    # when no members remain
MSG
  else
    cat <<'MSG'
[dev-uninstall-substrate] The 'substrate' group still exists. Remove it via
'sudo groupdel substrate' once all members have been detached.
MSG
  fi
}

PREFIX="${HOME}/.substrate"
PROFILE=""
SUBSTRATE_BIN=""
VERSION_LABEL="dev"
REMOVE_WORLD_SERVICE=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      shift 2
      ;;
    --profile)
      [[ $# -ge 2 ]] || fatal "--profile requires a value"
      PROFILE="$2"
      shift 2
      ;;
    --bin)
      [[ $# -ge 2 ]] || fatal "--bin requires a value"
      SUBSTRATE_BIN="$2"
      shift 2
      ;;
    --version-label)
      [[ $# -ge 2 ]] || fatal "--version-label requires a value"
      VERSION_LABEL="$2"
      shift 2
      ;;
    --remove-world-service)
      REMOVE_WORLD_SERVICE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fatal "Unknown argument: $1"
      ;;
  esac
done

BIN_DIR="${PREFIX%/}/bin"
VERSION_DIR="${PREFIX%/}/versions/${VERSION_LABEL}"
VERSIONS_ROOT="${PREFIX%/}/versions"
MANAGER_ENV_PATH="${PREFIX%/}/manager_env.sh"
MANAGER_INIT_PATH="${PREFIX%/}/manager_init.sh"
INSTALL_CONFIG_PATH="${PREFIX%/}/config.toml"
SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"
TRACE_LOG_PATH="${PREFIX%/}/trace.jsonl"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ -z "${SUBSTRATE_BIN}" ]]; then
  case "${PROFILE}" in
    release)
      SUBSTRATE_BIN="${REPO_ROOT}/target/release/substrate"
      ;;
    debug)
      SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
      ;;
    "")
      if [[ -x "${REPO_ROOT}/target/release/substrate" ]]; then
        SUBSTRATE_BIN="${REPO_ROOT}/target/release/substrate"
      elif [[ -x "${REPO_ROOT}/target/debug/substrate" ]]; then
        SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
      fi
      ;;
    *)
      fatal "Unsupported profile '${PROFILE}'. Use 'debug' or 'release'."
      ;;
  esac
fi

if [[ -n "${SUBSTRATE_BIN}" && ! -x "${SUBSTRATE_BIN}" ]]; then
  warn "Specified substrate binary (${SUBSTRATE_BIN}) is not executable; shim removal may be incomplete."
  SUBSTRATE_BIN=""
fi

if [[ -n "${SUBSTRATE_BIN}" ]]; then
  log "Removing shims via ${SUBSTRATE_BIN}"
  if ! SHIM_ORIGINAL_PATH="${PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 "${SUBSTRATE_BIN}" --shim-remove; then
    warn "substrate --shim-remove returned an error"
  fi
else
  warn "No substrate binary found; skipping shim-remove invocation."
fi

if [[ -d "${SHIMS_DIR}" ]]; then
  log "Deleting ${SHIMS_DIR}"
  rm -rf "${SHIMS_DIR}"
fi

if [[ -f "${ENV_FILE}" ]]; then
  log "Removing ${ENV_FILE}"
  rm -f "${ENV_FILE}"
fi

if [[ -d "${VERSION_DIR}" ]]; then
  log "Deleting ${VERSION_DIR}"
  rm -rf "${VERSION_DIR}"
fi

if [[ -f "${INSTALL_CONFIG_PATH}" ]]; then
  log "Removing ${INSTALL_CONFIG_PATH}"
  rm -f "${INSTALL_CONFIG_PATH}"
fi

if [[ -f "${MANAGER_ENV_PATH}" ]]; then
  log "Removing ${MANAGER_ENV_PATH}"
  rm -f "${MANAGER_ENV_PATH}"
fi

if [[ -f "${MANAGER_INIT_PATH}" ]]; then
  log "Removing ${MANAGER_INIT_PATH}"
  rm -f "${MANAGER_INIT_PATH}"
fi

if [[ -f "${TRACE_LOG_PATH}" ]]; then
  log "Removing ${TRACE_LOG_PATH}"
  rm -f "${TRACE_LOG_PATH}"
fi

if [[ -d "${BIN_DIR}" ]]; then
  log "Cleaning dev symlinks in ${BIN_DIR}"
  for binary in substrate substrate-shim substrate-forwarder host-proxy world-agent substrate-world-agent; do
    for candidate in "${binary}" "${binary}.exe"; do
      target_path="${BIN_DIR}/${candidate}"
      if [[ -L "${target_path}" ]]; then
        target="$(readlink "${target_path}")"
        if [[ -n "${target}" && "${target}" == "${REPO_ROOT}"/* ]]; then
          rm -f "${target_path}"
        fi
      fi
    done
  done
  rmdir "${BIN_DIR}" 2>/dev/null || true
fi

if [[ -d "${VERSIONS_ROOT}" ]]; then
  rmdir "${VERSIONS_ROOT}" 2>/dev/null || true
fi

if [[ -d "${PREFIX}" ]]; then
  rmdir "${PREFIX}" 2>/dev/null && log "Removed empty prefix ${PREFIX}"
fi

if [[ "${REMOVE_WORLD_SERVICE}" -eq 1 && "$(uname -s)" == "Linux" ]]; then
  log "Attempting to remove substrate-world-agent service (sudo may prompt)"
  if ! command -v sudo >/dev/null 2>&1; then
    warn "sudo not available; cannot modify substrate-world-agent service."
  else
    sudo systemctl disable --now substrate-world-agent.socket substrate-world-agent.service >/dev/null 2>&1 || warn "Failed to disable substrate-world-agent socket/service units"
    sudo rm -f /etc/systemd/system/substrate-world-agent.service || true
    sudo rm -f /etc/systemd/system/substrate-world-agent.socket || true
    sudo systemctl daemon-reload || true
    sudo rm -f /usr/local/bin/substrate-world-agent || true
    sudo rm -rf /var/lib/substrate || true
    sudo rm -rf /run/substrate || true
    sudo rm -f /run/substrate.sock || true
  fi
fi

cleanup_user="$(detect_invoking_user)"
print_linger_cleanup_notice "${cleanup_user}"
print_group_cleanup_notice "${cleanup_user}"

cat <<'MSG'

Dev shims removed. Open a new shell (or run `hash -r`) to clear cached commands.
Built artifacts under target/ are left untouched.
MSG
