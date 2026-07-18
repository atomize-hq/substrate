#!/usr/bin/env bash
set -euo pipefail

log() { printf '[substrate-uninstall] %s\n' "$1"; }
fatal() { printf '[substrate-uninstall][ERROR] %s\n' "$1" >&2; exit 1; }

resolve_install_bootstrap_context() {
  local declared="$1"
  local raw_prefix="$2"
  local supplied_carrier="$3"
  local carrier_declared="$4"
  local context_fd

  exec {context_fd}< <(python3 - "${declared}" "${raw_prefix}" "${supplied_carrier}" "${carrier_declared}" <<'PY'
import base64
import hashlib
import os
import pwd
import re
import sys

DOMAIN = "substrate.install_bootstrap_context"
KEYS = (
    "domain", "version", "selected_host_prefix", "host_substrate_home",
    "host_substrate_root", "principal_kind", "principal_account",
    "principal_uid", "host_context_commitment",
)


def fail():
    raise ValueError("invalid install bootstrap context")


def normalize_path(raw):
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw:
        fail()
    parts = []
    for part in raw[1:].split("/"):
        if not part:
            continue
        if part in (".", ".."):
            fail()
        parts.append(part)
    if not parts:
        fail()
    return "/" + "/".join(parts)


def b64_encode(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=")


def b64_decode(value):
    if not value or not re.fullmatch(rb"[A-Za-z0-9_-]+", value):
        fail()
    decoded = base64.urlsafe_b64decode(value + b"=" * ((-len(value)) % 4))
    if b64_encode(decoded) != value:
        fail()
    return decoded.decode("utf-8")


def entry_by_uid(uid):
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    entry = pwd.getpwuid(uid)
    if not entry.pw_name or pwd.getpwnam(entry.pw_name).pw_uid != uid:
        fail()
    if any(ch in entry.pw_name for ch in "\0\r\n"):
        fail()
    return entry


def entry_by_name(account):
    if not account or any(ch in account for ch in "\0\r\n"):
        fail()
    entry = pwd.getpwnam(account)
    if entry.pw_uid == 0 or pwd.getpwuid(entry.pw_uid).pw_name != entry.pw_name:
        fail()
    return entry


def public_principal():
    effective_uid = os.geteuid()
    if effective_uid != 0:
        return entry_by_uid(effective_uid)
    explicit = os.environ.get("SUBSTRATE_INSTALL_PRIMARY_USER", "")
    if explicit:
        return entry_by_name(explicit)
    sudo_user = os.environ.get("SUDO_USER", "")
    sudo_uid = os.environ.get("SUDO_UID", "")
    if not sudo_user or not sudo_uid or not re.fullmatch(r"0|[1-9][0-9]*", sudo_uid):
        fail()
    entry = entry_by_name(sudo_user)
    if str(entry.pw_uid) != sudo_uid:
        fail()
    return entry


def internal_principal():
    effective_uid = os.geteuid()
    if effective_uid != 0:
        return entry_by_uid(effective_uid)
    sudo_user = os.environ.get("SUDO_USER", "")
    sudo_uid = os.environ.get("SUDO_UID", "")
    if not sudo_user or not sudo_uid or not re.fullmatch(r"0|[1-9][0-9]*", sudo_uid):
        fail()
    entry = entry_by_name(sudo_user)
    if str(entry.pw_uid) != sudo_uid:
        fail()
    return entry


def frame(prefix, account, uid):
    encoded_prefix = b64_encode(prefix.encode("utf-8")).decode("ascii")
    return (
        f"domain={DOMAIN}\nversion=1\nselected_host_prefix={encoded_prefix}\n"
        f"host_substrate_home={encoded_prefix}\nhost_substrate_root={encoded_prefix}\n"
        f"principal_kind=unix\nprincipal_account={b64_encode(account.encode('utf-8')).decode('ascii')}\n"
        f"principal_uid={uid}\n"
    ).encode("ascii")


def decode_carrier(encoded):
    raw_encoded = encoded.encode("ascii")
    if not raw_encoded or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw_encoded):
        fail()
    record = base64.urlsafe_b64decode(raw_encoded + b"=" * ((-len(raw_encoded)) % 4))
    if b64_encode(record) != raw_encoded or not record.endswith(b"\n") or b"\r" in record or b"\0" in record:
        fail()
    lines = record[:-1].split(b"\n")
    if len(lines) != len(KEYS):
        fail()
    values = {}
    for expected, line in zip(KEYS, lines):
        if line.count(b"=") != 1:
            fail()
        key, value = line.split(b"=", 1)
        if key.decode("ascii") != expected:
            fail()
        values[expected] = value
    if values["domain"] != DOMAIN.encode("ascii") or values["version"] != b"1" or values["principal_kind"] != b"unix":
        fail()
    prefix = normalize_path(b64_decode(values["selected_host_prefix"]))
    if b64_decode(values["host_substrate_home"]) != prefix or b64_decode(values["host_substrate_root"]) != prefix:
        fail()
    account = b64_decode(values["principal_account"])
    raw_uid = values["principal_uid"]
    if not re.fullmatch(rb"0|[1-9][0-9]*", raw_uid):
        fail()
    uid = int(raw_uid)
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    commitment = values["host_context_commitment"].decode("ascii")
    if not re.fullmatch(r"[0-9a-f]{64}", commitment):
        fail()
    expected_frame = frame(prefix, account, uid)
    if record != expected_frame + b"host_context_commitment=" + commitment.encode("ascii") + b"\n":
        fail()
    if hashlib.sha256(expected_frame).hexdigest() != commitment:
        fail()
    return prefix, account, uid, commitment


try:
    declared = sys.argv[1] == "1"
    raw_prefix = sys.argv[2]
    supplied = sys.argv[3]
    internal = sys.argv[4] == "1"
    if internal:
        prefix, account, uid, commitment = decode_carrier(supplied)
        entry = internal_principal()
        if account != entry.pw_name or uid != entry.pw_uid:
            fail()
        if declared and normalize_path(raw_prefix) != prefix:
            fail()
        expected = {
            "SUBSTRATE_HOME": prefix,
            "SUBSTRATE_ROOT": prefix,
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT": commitment,
            "SUBSTRATE_INSTALL_PRIMARY_USER": account,
            "SUBSTRATE_INSTALL_PRIMARY_UID": str(uid),
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1": supplied,
        }
        for key, value in expected.items():
            if key in os.environ and os.environ[key] != value:
                fail()
        carrier = supplied
    else:
        entry = public_principal()
        account = entry.pw_name
        uid = entry.pw_uid
        prefix = normalize_path(raw_prefix if declared else entry.pw_dir.rstrip("/") + "/.substrate")
        commitment_input = frame(prefix, account, uid)
        commitment = hashlib.sha256(commitment_input).hexdigest()
        carrier = b64_encode(
            commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")
        ).decode("ascii")
    account_home = normalize_path(entry.pw_dir)
    for value in (prefix, carrier, commitment, account, str(uid), account_home):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid install bootstrap context", file=sys.stderr)
    raise SystemExit(2)
PY
  )
  IFS= read -r -d '' PREFIX <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1 <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_UID <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT_HOME <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  exec {context_fd}<&-

  export SUBSTRATE_HOME="${PREFIX}"
  export SUBSTRATE_ROOT="${PREFIX}"
  export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
  export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
  export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

PATH_SNIPPET_START="# >>> substrate >>>"
PATH_SNIPPET_END="# <<< substrate <<<"

remove_path_snippet() {
  local target="$1"
  if [[ -z "${target}" || ! -f "${target}" ]]; then
    return 0
  fi
  if ! grep -Fq "${PATH_SNIPPET_START}" "${target}" || ! grep -Fq "${PATH_SNIPPET_END}" "${target}"; then
    return 0
  fi

  local tmp
  tmp="$(mktemp)"
  sed "\#^${PATH_SNIPPET_START}\$#,\#^${PATH_SNIPPET_END}\$#d" "${target}" > "${tmp}"
  mv "${tmp}" "${target}"
}

remove_shell_path_snippets() {
  remove_path_snippet "${HOME}/.bashrc"
  remove_path_snippet "${HOME}/.bash_profile"
  remove_path_snippet "${HOME}/.profile"
  remove_path_snippet "${HOME}/.zshrc"
  remove_path_snippet "${HOME}/.zprofile"
  remove_path_snippet "${HOME}/.config/fish/config.fish"
}

usage() {
  cat <<'USAGE'
Substrate Uninstaller

Usage:
  uninstall-substrate.sh [--prefix <path>] [--cleanup-state] [--auto-cleanup] [-h|--help]

Options:
  --prefix <path>                  Select the installed Substrate host prefix
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
      if [[ -t 0 || -t 1 || -t 2 ]]; then
        log "sudo password required for '$*'; prompting..."
        sudo "$@"
        return $?
      fi
      log "sudo password required for '$*'; rerun uninstall with sudo to complete this step."
      return ${status}
    fi
    log "sudo failed running '$*' (exit ${status})."
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
PREFIX=""
PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
INSTALL_BOOTSTRAP_CONTEXT_DECLARED=0
INSTALL_BOOTSTRAP_COMMITMENT=""
INSTALL_BOOTSTRAP_ACCOUNT=""
INSTALL_BOOTSTRAP_UID=""
INSTALL_BOOTSTRAP_ACCOUNT_HOME=""
SUBSTRATE_HOME=""
HOST_STATE_PATH=""
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
    --prefix)
      if [[ $# -lt 2 ]]; then
        fatal "Missing value for --prefix"
      fi
      PREFIX="$2"
      PREFIX_DECLARED=1
      shift 2
      ;;
    --prefix=*)
      PREFIX="${1#--prefix=}"
      PREFIX_DECLARED=1
      shift
      ;;
    --install-bootstrap-context-v1)
      if [[ $# -lt 2 ]]; then
        fatal "Missing value for --install-bootstrap-context-v1"
      fi
      if [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 1 ]]; then
        fatal "Duplicate --install-bootstrap-context-v1"
      fi
      if [[ -z "$2" ]]; then
        fatal "Empty value for --install-bootstrap-context-v1"
      fi
      INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
      INSTALL_BOOTSTRAP_CONTEXT_DECLARED=1
      shift 2
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
      log "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

resolve_install_bootstrap_context \
  "${PREFIX_DECLARED}" \
  "${PREFIX}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
  "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}"
SUBSTRATE_HOME="${PREFIX}"
HOST_STATE_PATH="${SUBSTRATE_HOME}/install_state.json"

if [[ "${AUTO_CLEANUP}" -eq 1 ]]; then
  load_host_state_metadata "${HOST_STATE_PATH}" || true
fi

log "Stopping substrate processes (if any)..."
pgrep -fl substrate || true
pkill -x substrate || true
pkill -f '/substrate/bin/substrate-shim' || true
pkill -f '/substrate-forwarder' || true
pkill -f '/substrate-world-service' || true

log "Removing PATH snippet from shell rc files (if present)..."
remove_shell_path_snippets || true

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
    log "Stopping substrate-world-service service..."
    maybe_sudo systemctl stop substrate-world-service.service 2>/dev/null || true
    maybe_sudo systemctl stop substrate-world-service.socket 2>/dev/null || true
    maybe_sudo systemctl disable substrate-world-service.service 2>/dev/null || true
    maybe_sudo systemctl disable substrate-world-service.socket 2>/dev/null || true

    log "Removing systemd unit + runtime directories..."
    maybe_sudo rm -f /etc/systemd/system/substrate-world-service.service || true
    maybe_sudo rm -f /etc/systemd/system/substrate-world-service.socket || true
    maybe_sudo rm -rf /var/lib/substrate || true
    maybe_sudo rm -rf /run/substrate || true
    maybe_sudo rm -f /run/substrate.sock || true
    maybe_sudo systemctl daemon-reload 2>/dev/null || true

    log "Verifying substrate-world-service units are absent after uninstall (missing is ok)..."
    maybe_sudo systemctl status substrate-world-service.service 2>/dev/null || true
    maybe_sudo systemctl status substrate-world-service.socket 2>/dev/null || true
fi

log "Removing world-service binary from /usr/local/bin (if present)..."
maybe_sudo rm -f /usr/local/bin/substrate-world-service || true

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

host_agent_socket="${HOME}/.substrate/sock/agent.sock"
if [[ -S "${host_agent_socket}" || -f "${host_agent_socket}" ]]; then
  log "Removing host-forwarded agent socket at ${host_agent_socket}..."
  rm -f "${host_agent_socket}" || log "Unable to remove agent socket at ${host_agent_socket}; delete it manually."
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
