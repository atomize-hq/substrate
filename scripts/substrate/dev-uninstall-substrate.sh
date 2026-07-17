#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-uninstall-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

resolve_install_bootstrap_context() {
  local declared="$1"
  local raw_prefix="$2"
  local supplied_carrier="$3"
  local context_fd

  exec {context_fd}< <(python3 - "${declared}" "${raw_prefix}" "${supplied_carrier}" <<'PY'
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


def frame(prefix, account, uid):
    encoded = b64_encode(prefix.encode("utf-8")).decode("ascii")
    return (
        f"domain={DOMAIN}\nversion=1\nselected_host_prefix={encoded}\n"
        f"host_substrate_home={encoded}\nhost_substrate_root={encoded}\n"
        f"principal_kind=unix\nprincipal_account={b64_encode(account.encode('utf-8')).decode('ascii')}\n"
        f"principal_uid={uid}\n"
    ).encode("ascii")


def current_principal():
    uid = os.geteuid()
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    entry = pwd.getpwuid(uid)
    if not entry.pw_name or pwd.getpwnam(entry.pw_name).pw_uid != uid:
        fail()
    if any(ch in entry.pw_name for ch in "\0\r\n"):
        fail()
    return entry, uid


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
    entry, current_uid = current_principal()
    if supplied:
        prefix, account, uid, commitment = decode_carrier(supplied)
        if account != entry.pw_name or uid != current_uid:
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
        prefix = normalize_path(raw_prefix if declared else entry.pw_dir.rstrip("/") + "/.substrate")
        account = entry.pw_name
        uid = current_uid
        commitment_input = frame(prefix, account, uid)
        commitment = hashlib.sha256(commitment_input).hexdigest()
        carrier = b64_encode(commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")).decode("ascii")
    for value in (prefix, carrier, commitment, account, str(uid)):
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
  exec {context_fd}<&-

  export SUBSTRATE_HOME="${PREFIX}"
  export SUBSTRATE_ROOT="${PREFIX}"
  export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
  export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
  export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

usage() {
  cat <<'USAGE'
Substrate Dev Uninstaller

Removes development shims and helper files produced by dev-install-substrate.sh.

Usage:
  dev-uninstall-substrate.sh [--prefix <path>] [--profile <debug|release>] [--bin <path>] [--version-label <name>] [--kill-live-processes]
  dev-uninstall-substrate.sh --help

Options:
  --prefix <path>        Installation prefix that was used during dev install (default: ~/.substrate)
  --profile <name>       Cargo profile whose binary should be used for shim removal
  --bin <path>           Explicit path to substrate binary to invoke for shim removal
  --version-label <name> Version directory label used during dev install (default: dev)
  --kill-live-processes  Kill matching live dev owner-helper processes before removing files
  --remove-world-service Remove the Linux world-service systemd service (requires sudo)
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

remove_managed_symlink() {
  local path="$1"
  if [[ ! -L "${path}" ]]; then
    return 1
  fi
  local target
  target="$(readlink "${path}" || true)"
  if [[ -z "${target}" ]]; then
    return 1
  fi
  case "${target}" in
    "${REPO_ROOT}"/*)
      rm -f "${path}"
      log "Removing managed symlink ${path}"
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

remove_managed_prefix_linux_binary_copies() {
  local manifest_path="$1"
  if [[ -z "${manifest_path}" || ! -f "${manifest_path}" ]]; then
    return 0
  fi

  while IFS= read -r cached_path; do
    case "${cached_path}" in
      "${BIN_DIR}/linux/substrate"|\
      "${BIN_DIR}/linux/world-service")
        if [[ -f "${cached_path}" && ! -L "${cached_path}" ]]; then
          rm -f "${cached_path}"
          log "Removing managed copied Linux binary ${cached_path}"
        fi
        ;;
      *)
        ;;
    esac
  done < "${manifest_path}"

  rm -f "${manifest_path}"
}

record_protected_path() {
  local path="$1"
  if [[ -z "${path}" ]]; then
    return
  fi
  if [[ -n "${PROTECTED_PATHS_SEEN["${path}"]:-}" ]]; then
    return
  fi
  PROTECTED_PATHS_SEEN["${path}"]=1
  PROTECTED_PATHS+=("${path}")
}

collect_protected_paths() {
  local path
  for path in "$@"; do
    if [[ -e "${path}" || -L "${path}" ]]; then
      record_protected_path "${path}"
    fi
  done
}

report_protected_paths() {
  if [[ "${#PROTECTED_PATHS[@]}" -eq 0 ]]; then
    return 0
  fi

  warn "Protected paths preserved during uninstall:"
  while IFS= read -r path; do
    [[ -n "${path}" ]] || continue
    warn "Preserving protected path ${path}"
  done < <(printf '%s\n' "${PROTECTED_PATHS[@]}" | LC_ALL=C sort)

  return 0
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
  if [[ ${#RECORDED_LINGER_USERS[@]} -eq 0 ]]; then
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

kill_live_dev_owner_helpers() {
  if [[ "${KILL_LIVE_PROCESSES}" -ne 1 ]]; then
    return 0
  fi
  if ! command -v ps >/dev/null 2>&1; then
    warn "ps not available; cannot discover live dev owner-helper processes."
    return 0
  fi

  local -a candidate_pids=()
  local line=""
  local pid=""
  local cmd=""
  while IFS= read -r line; do
    [[ "${line}" =~ ^[[:space:]]*([0-9]+)[[:space:]]+(.*)$ ]] || continue
    pid="${BASH_REMATCH[1]}"
    cmd="${BASH_REMATCH[2]}"

    [[ "${cmd}" == *" agent __owner-helper "* ]] || continue

    case "${cmd}" in
      "${REPO_ROOT}/target/debug/substrate "*|\
      "${REPO_ROOT}/target/release/substrate "*|\
      "${PREFIX%/}/bin/substrate "*|\
      "${VERSION_DIR}/substrate "*)
        candidate_pids+=("${pid}")
        ;;
      *)
        if [[ "${cmd}" == *" --plan-file ${PREFIX%/}/run/agent-hub/handles/owner-helper/"* ]]; then
          candidate_pids+=("${pid}")
        fi
        ;;
    esac
  done < <(ps -eo pid=,args=)

  if [[ "${#candidate_pids[@]}" -eq 0 ]]; then
    log "No matching live dev owner-helper processes found."
    return 0
  fi

  log "Stopping ${#candidate_pids[@]} matching live dev owner-helper process(es): ${candidate_pids[*]}"
  kill "${candidate_pids[@]}" 2>/dev/null || true
  sleep 1

  local -a remaining_pids=()
  for pid in "${candidate_pids[@]}"; do
    if kill -0 "${pid}" 2>/dev/null; then
      remaining_pids+=("${pid}")
    fi
  done

  if [[ "${#remaining_pids[@]}" -eq 0 ]]; then
    return 0
  fi

  warn "Escalating to SIGKILL for surviving dev owner-helper process(es): ${remaining_pids[*]}"
  kill -9 "${remaining_pids[@]}" 2>/dev/null || true
  sleep 1

  for pid in "${remaining_pids[@]}"; do
    if kill -0 "${pid}" 2>/dev/null; then
      warn "Failed to terminate dev owner-helper process ${pid}"
    fi
  done
}

PREFIX=""
PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
PROFILE=""
SUBSTRATE_BIN=""
VERSION_LABEL="dev"
KILL_LIVE_PROCESSES=0
REMOVE_WORLD_SERVICE=0
AUTO_CLEANUP=0
HOST_STATE_PATH=""
HOST_STATE_METADATA_LOADED=0
RECORDED_GROUP_PREEXISTING=""
RECORDED_GROUP_CREATED=""
RECORDED_MEMBERS_ADDED=()
RECORDED_LINGER_USERS=()
PROTECTED_PATHS=()
declare -A PROTECTED_PATHS_SEEN=()
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
      PREFIX_DECLARED=1
      shift 2
      ;;
    --install-bootstrap-context-v1)
      [[ $# -ge 2 ]] || fatal "--install-bootstrap-context-v1 requires a value"
      INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
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
    --kill-live-processes)
      KILL_LIVE_PROCESSES=1
      shift
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

resolve_install_bootstrap_context "${PREFIX_DECLARED}" "${PREFIX}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}"

BIN_DIR="${PREFIX%/}/bin"
VERSION_DIR="${PREFIX%/}/versions/${VERSION_LABEL}"
VERSIONS_ROOT="${PREFIX%/}/versions"
MANAGER_ENV_PATH="${PREFIX%/}/manager_env.sh"
MANAGER_INIT_PATH="${PREFIX%/}/manager_init.sh"
INSTALL_CONFIG_PATH="${PREFIX%/}/config.yaml"
LEGACY_INSTALL_CONFIG_PATH="${PREFIX%/}/config.toml"
HOST_STATE_PATH="${PREFIX%/}/install_state.json"
SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"
TRACE_LOG_PATH="${PREFIX%/}/trace.jsonl"
RUN_DIR="${PREFIX%/}/run"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
MANAGED_STATE_DIR="${PREFIX%/}/.dev-install-managed"
MANAGED_MAC_LINUX_BINARIES_PATH="${MANAGED_STATE_DIR}/mac-linux-binaries.txt"

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

kill_live_dev_owner_helpers

if [[ -n "${SUBSTRATE_BIN}" ]]; then
  log "Removing shims via ${SUBSTRATE_BIN}"
  if ! SHIM_ORIGINAL_PATH="${PATH}" "${SUBSTRATE_BIN}" --no-world \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    --shim-remove; then
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

if [[ -f "${LEGACY_INSTALL_CONFIG_PATH}" ]]; then
  log "Removing ${LEGACY_INSTALL_CONFIG_PATH}"
  rm -f "${LEGACY_INSTALL_CONFIG_PATH}"
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

if [[ -d "${RUN_DIR}" ]]; then
  log "Removing ${RUN_DIR}"
  rm -rf "${RUN_DIR}"
fi

if [[ -d "${BIN_DIR}" ]]; then
  log "Cleaning dev symlinks in ${BIN_DIR}"
  for binary in substrate substrate-shim substrate-forwarder host-proxy world-service substrate-world-service; do
    for candidate in "${binary}" "${binary}.exe"; do
      target_path="${BIN_DIR}/${candidate}"
      if remove_managed_symlink "${target_path}"; then
        continue
      fi
    done
  done
  rmdir "${BIN_DIR}" 2>/dev/null || true
fi

RUNTIME_SCRIPTS_DIR="${PREFIX%/}/scripts"
if [[ -d "${RUNTIME_SCRIPTS_DIR}" ]]; then
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/substrate/world-enable.sh" || true
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/substrate/install-substrate.sh" || true
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/substrate/world-deps.yaml" || true
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/mac/lima-warm.sh" || true
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/mac/lima/substrate.yaml" || true
  remove_managed_symlink "${RUNTIME_SCRIPTS_DIR}/mac/lima/substrate-dev.yaml" || true
  rmdir "${RUNTIME_SCRIPTS_DIR}/substrate" 2>/dev/null || true
  rmdir "${RUNTIME_SCRIPTS_DIR}/mac/lima" 2>/dev/null || true
  rmdir "${RUNTIME_SCRIPTS_DIR}/mac" 2>/dev/null || true
  rmdir "${RUNTIME_SCRIPTS_DIR}" 2>/dev/null || true
fi

BIN_LINUX_DIR="${BIN_DIR}/linux"
if [[ -d "${BIN_LINUX_DIR}" ]]; then
  remove_managed_prefix_linux_binary_copies "${MANAGED_MAC_LINUX_BINARIES_PATH}"
  remove_managed_symlink "${BIN_LINUX_DIR}/substrate" || true
  remove_managed_symlink "${BIN_LINUX_DIR}/world-service" || true
  rmdir "${BIN_LINUX_DIR}" 2>/dev/null || true
fi
rmdir "${BIN_DIR}" 2>/dev/null || true
rmdir "${MANAGED_STATE_DIR}" 2>/dev/null || true

if [[ -d "${VERSIONS_ROOT}" ]]; then
  rmdir "${VERSIONS_ROOT}" 2>/dev/null || true
fi

if [[ -d "${PREFIX}" ]]; then
  rmdir "${PREFIX}" 2>/dev/null && log "Removed empty prefix ${PREFIX}"
fi

if [[ "${REMOVE_WORLD_SERVICE}" -eq 1 && "${IS_LINUX}" -eq 1 ]]; then
  log "Attempting to remove substrate-world-service service (sudo may prompt)"
  if ! command -v sudo >/dev/null 2>&1; then
    warn "sudo not available; cannot modify substrate-world-service service."
  else
    sudo systemctl disable --now substrate-world-service.socket substrate-world-service.service >/dev/null 2>&1 || warn "Failed to disable substrate-world-service socket/service units"
    sudo rm -f /etc/systemd/system/substrate-world-service.service || true
    sudo rm -f /etc/systemd/system/substrate-world-service.socket || true
    sudo systemctl daemon-reload || true
    sudo rm -f /usr/local/bin/substrate-world-service || true
    sudo rm -rf /var/lib/substrate || true
    sudo rm -rf /run/substrate || true
    sudo rm -f /run/substrate.sock || true
  fi
elif [[ "${REMOVE_WORLD_SERVICE}" -eq 1 && "${IS_MAC}" -eq 1 ]]; then
  log "Attempting to remove Lima world-service service from VM 'substrate'"
  if ! command -v limactl >/dev/null 2>&1; then
    warn "limactl not available; cannot modify Lima world-service service. Remove manually if desired."
  elif [[ ! -d "${HOME}/.lima/substrate" ]]; then
    warn "Lima VM 'substrate' not found under ${HOME}/.lima/substrate; skipping guest cleanup."
  else
    # Use sudo -n to avoid hanging on password prompts. If this fails, instruct the user to run the
    # commands interactively via `limactl shell substrate`.
    out="$(
      limactl shell substrate -- bash -lc \
        'sudo -n systemctl disable --now substrate-world-service.socket substrate-world-service.service' 2>&1
    )" || warn "Failed to disable agent units inside Lima VM. Try: limactl shell substrate, then run: sudo systemctl disable --now substrate-world-service.socket substrate-world-service.service. Error: ${out}"

    limactl shell substrate -- bash -lc \
      'sudo -n rm -f /etc/systemd/system/substrate-world-service.service /etc/systemd/system/substrate-world-service.socket' >/dev/null 2>&1 || true
    limactl shell substrate -- bash -lc \
      'sudo -n systemctl daemon-reload' >/dev/null 2>&1 || true
    limactl shell substrate -- bash -lc \
      'sudo -n rm -f /usr/local/bin/substrate-world-service' >/dev/null 2>&1 || true
    limactl shell substrate -- bash -lc \
      'sudo -n rm -f /usr/local/bin/substrate /usr/local/bin/world' >/dev/null 2>&1 || true
    limactl shell substrate -- bash -lc \
      'sudo -n rm -rf /var/lib/substrate /run/substrate' >/dev/null 2>&1 || true
    limactl shell substrate -- bash -lc \
      'sudo -n rm -f /run/substrate.sock' >/dev/null 2>&1 || true
    # Clean up host-forwarded socket if present
    host_sock="${HOME}/.substrate/sock/agent.sock"
    if [[ -S "${host_sock}" || -f "${host_sock}" ]]; then
      rm -f "${host_sock}" || warn "Unable to remove host agent socket at ${host_sock}"
    fi
  fi
fi

cleanup_user="$(detect_invoking_user)"
perform_auto_cleanup "${cleanup_user}"

collect_protected_paths \
  "${BIN_DIR}/substrate" \
  "${BIN_DIR}/substrate.exe" \
  "${BIN_DIR}/substrate-shim" \
  "${BIN_DIR}/substrate-shim.exe" \
  "${BIN_DIR}/substrate-forwarder" \
  "${BIN_DIR}/substrate-forwarder.exe" \
  "${BIN_DIR}/host-proxy" \
  "${BIN_DIR}/host-proxy.exe" \
  "${BIN_DIR}/world-service" \
  "${BIN_DIR}/world-service.exe" \
  "${BIN_DIR}/substrate-world-service" \
  "${BIN_DIR}/substrate-world-service.exe" \
  "${RUNTIME_SCRIPTS_DIR}/substrate/world-enable.sh" \
  "${RUNTIME_SCRIPTS_DIR}/substrate/install-substrate.sh" \
  "${RUNTIME_SCRIPTS_DIR}/substrate/world-deps.yaml" \
  "${RUNTIME_SCRIPTS_DIR}/mac/lima-warm.sh" \
  "${RUNTIME_SCRIPTS_DIR}/mac/lima/substrate.yaml" \
  "${RUNTIME_SCRIPTS_DIR}/mac/lima/substrate-dev.yaml" \
  "${BIN_LINUX_DIR}/substrate" \
  "${BIN_LINUX_DIR}/world-service"

if [[ "${#PROTECTED_PATHS[@]}" -gt 0 ]]; then
  report_protected_paths
  warn "Protected-path refusal class exit 5"
  exit 5
fi

cat <<'MSG'

Dev shims removed. Open a new shell (or run `hash -r`) to clear cached commands.
Built artifacts under target/ are left untouched.
MSG

exit 0
