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
  --cleanup-state        Remove installer-recorded group membership/lingering (opt-in)
  --help                 Show this message

If neither --profile nor --bin is provided the script will look for
`target/release/substrate` first, then `target/debug/substrate`.
USAGE
}

run_privileged() {
  if [[ ${EUID} -eq 0 ]]; then
    "$@"
    return $?
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo "$@"
    return $?
  fi
  warn "Command requires elevated privileges but sudo is unavailable: $*"
  return 1
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
  local in_group=0
  if [[ -n "${target_user}" ]]; then
    if user_in_group "${target_user}" substrate; then
      in_group=1
    fi
  fi
  if [[ -n "${target_user}" && "${target_user}" != "root" && "${in_group}" -eq 1 ]]; then
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

load_host_state_metadata() {
  HOST_STATE_METADATA_LOADED=0
  RECORDED_GROUP_PREEXISTING=""
  RECORDED_GROUP_CREATED=""
  RECORDED_MEMBERS_ADDED=()
  RECORDED_LINGER_USERS=()

  local path="$1"
  if [[ -z "${path}" || ! -f "${path}" ]]; then
    return 1
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    warn "python3 not available; skipping host state metadata read (${path})."
    return 1
  fi

  local output
  if ! output="$(python3 - "${path}" <<'PY'
import json
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
try:
    data = json.loads(path.read_text())
except Exception as exc:  # noqa: BLE001
    sys.stderr.write(f"[dev-uninstall-substrate] warning: unable to parse {path}: {exc}\n")
    sys.exit(1)

if data.get("schema_version") != 1:
    sys.stderr.write(f"[dev-uninstall-substrate] warning: unsupported host state schema in {path}\n")
    sys.exit(2)

host = data.get("host_state") or {}
group = host.get("group") or {}
pre = group.get("existed_before")
if isinstance(pre, bool):
    print(f"group_preexisting:{str(pre).lower()}")
created = group.get("created_by_installer")
if isinstance(created, bool):
    print(f"group_created:{str(created).lower()}")
members = group.get("members_added") or []
for member in members:
    if isinstance(member, str):
        print(f"user_added:{member}")

linger = host.get("linger") or {}
for user, info in (linger.get("users") or {}).items():
    if not isinstance(info, dict):
        continue
    if info.get("enabled_by_substrate"):
        print(f"linger_enabled:{user}")
PY
)"; then
    return 1
  fi

  while IFS= read -r line; do
    case "${line}" in
      group_preexisting:*)
        RECORDED_GROUP_PREEXISTING="${line#*:}"
        ;;
      group_created:*)
        RECORDED_GROUP_CREATED="${line#*:}"
        ;;
      user_added:*)
        RECORDED_MEMBERS_ADDED+=("${line#*:}")
        ;;
      linger_enabled:*)
        RECORDED_LINGER_USERS+=("${line#*:}")
        ;;
    esac
  done <<< "${output}"

  HOST_STATE_METADATA_LOADED=1
  return 0
}

cleanup_recorded_group() {
  local removed=0
  if [[ "${IS_LINUX}" -ne 1 ]]; then
    return 0
  fi
  if ! command -v getent >/dev/null 2>&1; then
    warn "getent not available; cannot verify substrate group membership for cleanup."
    return 0
  fi
  if ! getent group substrate >/dev/null 2>&1; then
    return 0
  fi

  for user in "${RECORDED_MEMBERS_ADDED[@]:-}"; do
    if [[ -z "${user}" || "${user}" == "root" ]]; then
      continue
    fi
    if user_in_group "${user}" "substrate"; then
      if run_privileged gpasswd -d "${user}" substrate; then
        log "Removed ${user} from substrate group (recorded during install)."
        removed=1
      else
        warn "Failed to remove ${user} from substrate group; remove manually if desired."
      fi
    fi
  done

  local allow_group_delete=0
  if [[ "${RECORDED_GROUP_PREEXISTING}" == "false" || "${RECORDED_GROUP_CREATED}" == "true" ]]; then
    allow_group_delete=1
  fi
  if [[ "${allow_group_delete}" -eq 1 ]]; then
    local members
    members="$(getent group substrate | cut -d: -f4 || true)"
    if [[ -z "${members}" ]]; then
      if run_privileged groupdel substrate; then
        log "Deleted substrate group (created by installer, no remaining members)."
        removed=1
      else
        warn "Unable to delete substrate group; remove it manually if it is no longer needed."
      fi
    else
      warn "substrate group still has members (${members}); skipping deletion."
    fi
  fi

  return ${removed}
}

cleanup_recorded_linger() {
  local changed=0
  if [[ "${IS_LINUX}" -ne 1 ]]; then
    return 0
  fi
  if [[ ${#RECORDED_LINGER_USERS[@]:-} -eq 0 ]]; then
    return 0
  fi
  if ! command -v loginctl >/dev/null 2>&1; then
    warn "loginctl not available; cannot disable lingering automatically."
    return 0
  fi

  for user in "${RECORDED_LINGER_USERS[@]}"; do
    if [[ -z "${user}" || "${user}" == "root" ]]; then
      continue
    fi
    local linger_state
    linger_state="$(loginctl show-user "${user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
    if [[ "${linger_state}" != "yes" ]]; then
      continue
    fi
    if run_privileged loginctl disable-linger "${user}"; then
      log "Disabled lingering for ${user} based on installer metadata."
      changed=1
    else
      warn "Failed to disable lingering for ${user}; run 'loginctl disable-linger ${user}' manually if needed."
    fi
  done

  return ${changed}
}

perform_auto_cleanup() {
  local cleanup_user="$1"

  if [[ "${AUTO_CLEANUP}" -ne 1 ]]; then
    print_linger_cleanup_notice "${cleanup_user}"
    print_group_cleanup_notice "${cleanup_user}"
    return
  fi

  if [[ "${IS_LINUX}" -ne 1 ]]; then
    warn "Host-state cleanup is only supported on Linux; showing manual guidance."
    print_linger_cleanup_notice "${cleanup_user}"
    print_group_cleanup_notice "${cleanup_user}"
    return
  fi

  if [[ "${HOST_STATE_METADATA_LOADED}" -ne 1 ]]; then
    warn "Host-state metadata missing or unreadable; falling back to manual cleanup guidance."
    print_linger_cleanup_notice "${cleanup_user}"
    print_group_cleanup_notice "${cleanup_user}"
    return
  fi

  local actions=0
  if ! cleanup_recorded_group; then
    actions=1
  fi
  if ! cleanup_recorded_linger; then
    actions=1
  fi

  if [[ "${actions}" -eq 0 ]]; then
    log "No recorded host-state changes required automatic cleanup. Showing guidance instead."
    print_linger_cleanup_notice "${cleanup_user}"
    print_group_cleanup_notice "${cleanup_user}"
  fi
}

PREFIX="${HOME}/.substrate"
PROFILE=""
SUBSTRATE_BIN=""
VERSION_LABEL="dev"
REMOVE_WORLD_SERVICE=0
AUTO_CLEANUP=0
HOST_STATE_PATH=""
HOST_STATE_METADATA_LOADED=0
RECORDED_GROUP_PREEXISTING=""
RECORDED_GROUP_CREATED=""
RECORDED_MEMBERS_ADDED=()
RECORDED_LINGER_USERS=()
IS_LINUX=0
IS_MAC=0
if [[ "$(uname -s)" == "Linux" ]]; then
  IS_LINUX=1
fi
if [[ "$(uname -s)" == "Darwin" ]]; then
  IS_MAC=1
fi

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
    --cleanup-state|--auto-cleanup)
      AUTO_CLEANUP=1
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
HOST_STATE_PATH="${PREFIX%/}/install_state.json"
SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"
TRACE_LOG_PATH="${PREFIX%/}/trace.jsonl"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ "${AUTO_CLEANUP}" -eq 1 ]]; then
  load_host_state_metadata "${HOST_STATE_PATH}" || true
fi

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

if [[ "${REMOVE_WORLD_SERVICE}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
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
elif [[ "${REMOVE_WORLD_SERVICE}" -eq 1 && "${IS_MAC}" -eq 1 ]]; then
  log "Attempting to remove Lima world-agent service from VM 'substrate'"
  if ! command -v limactl >/dev/null 2>&1; then
    warn "limactl not available; cannot modify Lima world-agent service. Remove manually if desired."
  else
    limactl shell substrate sudo systemctl disable --now substrate-world-agent.socket substrate-world-agent.service >/dev/null 2>&1 || warn "Failed to disable agent units inside Lima VM"
    limactl shell substrate sudo rm -f /etc/systemd/system/substrate-world-agent.service /etc/systemd/system/substrate-world-agent.socket >/dev/null 2>&1 || true
    limactl shell substrate sudo systemctl daemon-reload >/dev/null 2>&1 || true
    limactl shell substrate sudo rm -f /usr/local/bin/substrate-world-agent >/dev/null 2>&1 || true
    limactl shell substrate sudo rm -rf /var/lib/substrate /run/substrate >/dev/null 2>&1 || true
    limactl shell substrate sudo rm -f /run/substrate.sock >/dev/null 2>&1 || true
    # Clean up host-forwarded socket if present
    host_sock="${HOME}/.substrate/sock/agent.sock"
    if [[ -S "${host_sock}" || -f "${host_sock}" ]]; then
      rm -f "${host_sock}" || warn "Unable to remove host agent socket at ${host_sock}"
    fi
  fi
fi

cleanup_user="$(detect_invoking_user)"
perform_auto_cleanup "${cleanup_user}"

cat <<'MSG'

Dev shims removed. Open a new shell (or run `hash -r`) to clear cached commands.
Built artifacts under target/ are left untouched.
MSG
