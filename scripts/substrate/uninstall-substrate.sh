#!/usr/bin/env bash
set -euo pipefail

log() { printf '[substrate-uninstall] %s\n' "$1"; }

usage() {
  cat <<'USAGE'
Substrate Uninstaller

Usage:
  uninstall-substrate.sh [--cleanup-state] [--auto-cleanup] [-h|--help]

Options:
  --cleanup-state, --auto-cleanup  Remove installer-recorded group membership/lingering (opt-in)
  -h, --help                       Show this message
USAGE
}

maybe_sudo() {
  if [[ ${EUID} -eq 0 ]]; then
    "$@"
    return
  fi

  if command -v sudo >/dev/null 2>&1; then
    sudo -n "$@"
    local status=$?
    if [[ ${status} -eq 0 ]]; then
      return
    fi
    if [[ ${status} -eq 1 ]]; then
      log "sudo password required for '$*'; rerun uninstall with sudo to complete this step."
    else
      log "sudo failed running '$*' (exit ${status})."
    fi
    return ${status}
  fi

  log "sudo not available; attempting '$*' without elevation"
  "$@"
}

run_python() {
  local clean_path
  clean_path="${SHIM_ORIGINAL_PATH:-/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin}"
  env -i PATH="${clean_path}" HOME="${HOME}" python3 "$@"
}

detect_primary_user() {
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
[substrate-uninstall] loginctl: Unable to determine which user enabled lingering.
Disable lingering manually if socket activation is no longer required:
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
[substrate-uninstall] loginctl reports lingering enabled for ${target_user}.
Disable it if Substrate is fully removed:
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
[substrate-uninstall] ${target_user} is still a member of the 'substrate' group.
Remove the membership (and the group when empty) if you no longer need socket access:
  sudo gpasswd -d ${target_user} substrate
  sudo groupdel substrate    # when no members remain
MSG
  else
    cat <<'MSG'
[substrate-uninstall] The 'substrate' group remains on this host. Delete it with
'sudo groupdel substrate' once all members have been removed.
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
    log "python3 not available; skipping host state metadata read (${path})."
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
    sys.stderr.write(f"[substrate-uninstall] warning: unable to parse {path}: {exc}\n")
    sys.exit(1)

if data.get("schema_version") != 1:
    sys.stderr.write(f"[substrate-uninstall] warning: unsupported host state schema in {path}\n")
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
    log "getent not available; cannot verify substrate group membership for cleanup."
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
      if maybe_sudo gpasswd -d "${user}" substrate; then
        log "Removed ${user} from substrate group (recorded during install)."
        removed=1
      else
        log "Unable to remove ${user} from substrate group automatically."
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
      if maybe_sudo groupdel substrate; then
        log "Deleted substrate group (created by installer, no remaining members)."
        removed=1
      else
        log "Unable to delete substrate group; remove it manually if desired."
      fi
    else
      log "substrate group still has members (${members}); skipping deletion."
    fi
  fi

  return ${removed}
}

cleanup_recorded_linger() {
  local changed=0
  if [[ "${IS_LINUX}" -ne 1 ]]; then
    return 0
  fi
  if [[ ${#RECORDED_LINGER_USERS[@]} -eq 0 ]]; then
    return 0
  fi
  if ! command -v loginctl >/dev/null 2>&1; then
    log "loginctl not available; cannot disable lingering automatically."
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
    if maybe_sudo loginctl disable-linger "${user}"; then
      log "Disabled lingering for ${user} based on installer metadata."
      changed=1
    else
      log "Failed to disable lingering for ${user}; run 'loginctl disable-linger ${user}' manually if needed."
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
    log "Host-state cleanup is only supported on Linux; showing manual guidance."
    print_linger_cleanup_notice "${cleanup_user}"
    print_group_cleanup_notice "${cleanup_user}"
    return
  fi

  if [[ "${HOST_STATE_METADATA_LOADED}" -ne 1 ]]; then
    log "Host-state metadata missing or unreadable; falling back to manual cleanup guidance."
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

AUTO_CLEANUP=0
HOST_STATE_PATH="${HOME}/.substrate/install_state.json"
HOST_STATE_METADATA_LOADED=0
RECORDED_GROUP_PREEXISTING=""
RECORDED_GROUP_CREATED=""
RECORDED_MEMBERS_ADDED=()
RECORDED_LINGER_USERS=()
IS_LINUX=0
if [[ "$(uname -s)" == "Linux" ]]; then
  IS_LINUX=1
fi

while [[ $# -gt 0 ]]; do
  case "$1" in
    --cleanup-state|--auto-cleanup)
      AUTO_CLEANUP=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      log "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

if [[ "${AUTO_CLEANUP}" -eq 1 ]]; then
  load_host_state_metadata "${HOST_STATE_PATH}" || true
fi

log "Stopping substrate processes (if any)..."
pgrep -fl substrate || true
pkill -x substrate || true
pkill -f '/substrate/bin/substrate-shim' || true
pkill -f '/substrate-forwarder' || true
pkill -f '/substrate-world-agent' || true

log "Removing substrate directories..."
run_python - <<'PY'
import pathlib, shutil
home = pathlib.Path.home()
for target in [
    '.substrate',
    '.substrate/config',
    '.substrate/versions',
    '.substrate_bashenv',
    '.substrate_bashenv_trampoline',
    '.substrate_preexec',
    '.substrate_history',
    '.substrate.lock',
]:
    path = home / target
    if path.is_dir():
        shutil.rmtree(path, ignore_errors=True)
    elif path.exists():
        path.unlink()
PY

if command -v systemctl >/dev/null 2>&1; then
    log "Stopping substrate-world-agent service..."
    maybe_sudo systemctl stop substrate-world-agent.service 2>/dev/null || true
    maybe_sudo systemctl stop substrate-world-agent.socket 2>/dev/null || true
    maybe_sudo systemctl disable substrate-world-agent.service 2>/dev/null || true
    maybe_sudo systemctl disable substrate-world-agent.socket 2>/dev/null || true

    log "Removing systemd unit + runtime directories..."
    maybe_sudo rm -f /etc/systemd/system/substrate-world-agent.service || true
    maybe_sudo rm -f /etc/systemd/system/substrate-world-agent.socket || true
    maybe_sudo rm -rf /var/lib/substrate || true
    maybe_sudo rm -rf /run/substrate || true
    maybe_sudo rm -f /run/substrate.sock || true
    maybe_sudo systemctl daemon-reload 2>/dev/null || true

    log "Verifying substrate-world-agent units are absent after uninstall (missing is ok)..."
    maybe_sudo systemctl status substrate-world-agent.service 2>/dev/null || true
    maybe_sudo systemctl status substrate-world-agent.socket 2>/dev/null || true
fi

log "Removing world-agent binary from /usr/local/bin (if present)..."
maybe_sudo rm -f /usr/local/bin/substrate-world-agent || true

if command -v limactl >/dev/null 2>&1; then
  # Only relevant on macOS hosts where Lima is installed.
  if [[ "$(uname -s)" == "Darwin" ]]; then
    log "Removing Lima VM..."
    if limactl list 2>/dev/null | grep -q substrate; then
      limactl stop substrate || true
      limactl delete substrate || true
    fi
  fi
fi

log "Checking for host symlinks..."
for target in /usr/local/bin/substrate*; do
  if [[ -e "${target}" ]]; then
    ls -l "${target}"
  fi
done
if [[ -d "${HOME}/bin" ]]; then
  for target in "${HOME}"/bin/substrate*; do
    if [[ -e "${target}" ]]; then
      ls -l "${target}"
    fi
  done
fi

log "Clearing shell command cache..."
hash -r || true

cleanup_user="$(detect_primary_user)"
perform_auto_cleanup "${cleanup_user}"

log "Done. Open a new shell to pick up changes."
