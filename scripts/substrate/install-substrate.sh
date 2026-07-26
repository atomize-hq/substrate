#!/usr/bin/env bash
RELEASE_INSTALL_INHERITED_XTRACE=0
if [[ $- == *x* ]]; then
  RELEASE_INSTALL_INHERITED_XTRACE=1
  set +x
fi
set -euo pipefail

if [[ -z "${INSTALLER_NAME:-}" ]]; then
  INSTALLER_NAME="substrate-install"
fi
readonly INSTALLER_NAME
# shellcheck disable=SC2034 # used for release metadata
readonly INSTALLER_VERSION="0.1.0-dev"
readonly DEFAULT_FALLBACK_VERSION="0.2.2"
readonly LATEST_RELEASE_API="${SUBSTRATE_INSTALL_LATEST_API:-https://api.github.com/repos/atomize-hq/substrate/releases/latest}"
readonly DEFAULT_PREFIX=""
readonly DEFAULT_BASE_URL="https://github.com/atomize-hq/substrate/releases/download"
readonly PRIVILEGED_TOOL_PATH="/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"

VERSION_RAW=""
VERSION=""
VERSION_TAG=""
PREFIX="$DEFAULT_PREFIX"
PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
INSTALL_BOOTSTRAP_CONTEXT_DECLARED=0
INSTALL_BOOTSTRAP_COMMITMENT=""
INSTALL_BOOTSTRAP_ACCOUNT=""
INSTALL_BOOTSTRAP_UID=""
INSTALL_BOOTSTRAP_ACCOUNT_HOME=""
HELP_REQUESTED=0
NO_WORLD=0
NO_SHIMS=0
DRY_RUN=0
SYNC_DEPS=0
ENABLE_WORLD_NETFILTER=0
PROVISION_AGENT_RUNTIME=""
PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
ARTIFACT_DIR="${SUBSTRATE_INSTALL_ARTIFACT_DIR:-${SUBSTRATE_INSTALL_ARCHIVE:-}}"
BASE_URL="${SUBSTRATE_INSTALL_BASE_URL:-$DEFAULT_BASE_URL}"
TMPDIR=""
PLATFORM=""
ARCH=""
IS_WSL=0
ORIGINAL_PATH="${PATH}"
PKG_MANAGER_ENV_OVERRIDE="${PKG_MANAGER:-}"
PKG_MANAGER=""
PKG_MANAGER_SOURCE=""
PKG_MANAGER_FLAG_OVERRIDE=""
PKG_MANAGER_DECISION_LINE_EMITTED=0
APT_UPDATED=0
SUDO_CMD=()
MANAGER_ENV_PATH=""
MANAGER_INIT_PATH=""
INSTALL_CONFIG_PATH=""
HOST_STATE_PATH=""
HOST_STATE_GROUP_EXISTED=""
HOST_STATE_GROUP_CREATED=0
HOST_STATE_ADDED_USERS=()
HOST_STATE_LINGER_ENTRIES=()
readonly DISTRO_UNKNOWN_SENTINEL="<unknown>"
OS_RELEASE_SELECTED_PATH=""
OS_RELEASE_INPUT_STATE="unavailable"
DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"
readonly SUPPORTED_PKG_MANAGERS=(apt-get dnf yum pacman zypper)
PATH_PROBE_DETECTED_MANAGERS=()
PKG_MANAGER_PATH_PROBE_WARNING_EMITTED=0

log() {
  printf '[%s] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

warn() {
  printf '[%s][WARN] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

fatal() {
  printf '[%s][ERROR] %s\n' "${INSTALLER_NAME}" "$*" >&2
  exit 1
}

fatal_with_code() {
  local code="$1"
  shift
  printf '[%s][ERROR] %s\n' "${INSTALLER_NAME}" "$*" >&2
  exit "${code}"
}

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
    "domain",
    "version",
    "selected_host_prefix",
    "host_substrate_home",
    "host_substrate_root",
    "principal_kind",
    "principal_account",
    "principal_uid",
    "host_context_commitment",
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
        f"domain={DOMAIN}\n"
        "version=1\n"
        f"selected_host_prefix={encoded_prefix}\n"
        f"host_substrate_home={encoded_prefix}\n"
        f"host_substrate_root={encoded_prefix}\n"
        "principal_kind=unix\n"
        f"principal_account={b64_encode(account.encode('utf-8')).decode('ascii')}\n"
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

print_usage() {
  cat <<'EOF'
Substrate Installer
Usage:
  curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash
  # (Windows host) powershell -ExecutionPolicy Bypass -File install-substrate.ps1

Options:
  --version <semver>   Install a specific release (default: latest GitHub release)
  --prefix <path>      Installation prefix (default: ~/.substrate)
  --pkg-manager <mgr>  Force Linux package manager: apt-get|dnf|yum|pacman|zypper
  --no-world           Skip world backend provisioning
  --no-shims           Skip shim deployment
  --sync-deps          Run 'substrate world deps current sync' after provisioning completes
  --provision-agent-runtime <runtime_family>
                       Enable a world runtime globally (codex only in this slice), then run
                       'substrate world deps current sync' so the guest install step is impossible to miss
  --world-netfilter    Enable Linux nftables egress scoping (sets WORLD_NETFILTER_ENABLE=1 for substrate-world-service.service)
  --dry-run            Print actions without executing
  --artifact-dir <dir> Use pre-downloaded host bundle + SHA256SUMS
  --archive <dir>      Alias for --artifact-dir (deprecated)
  -h, --help           Show this message
EOF
}

cleanup() {
  if [[ -n "${TMPDIR}" && -d "${TMPDIR}" && "${DRY_RUN}" -eq 0 ]]; then
    rm -rf "${TMPDIR}"
  fi
}

run_cmd() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s\n' "${INSTALLER_NAME}" "$*" >&2
    return 0
  fi
  "$@"
}

run_cmd_with_redacted_install_bootstrap_carrier() {
  local carrier_flag="--install-bootstrap-context-v1"
  local placeholder="<redacted-authenticated-bootstrap-carrier-v1>"
  local -a argv=("$@")
  local argc="${#argv[@]}"
  local carrier_count=0
  local after_end_of_options=0
  local invalid=0
  local index
  local argument
  local carrier_value

  for ((index = 0; index < argc; index += 1)); do
    argument="${argv[index]}"
    if [[ "${argument}" == "--" ]]; then
      after_end_of_options=1
      continue
    fi
    if [[ "${argument}" == "${carrier_flag}" ]]; then
      if [[ "${after_end_of_options}" -eq 1 ]]; then
        invalid=1
        continue
      fi
      carrier_count=$((carrier_count + 1))
      if [[ "${carrier_count}" -gt 1 || $((index + 1)) -ge argc ]]; then
        invalid=1
        continue
      fi
      carrier_value="${argv[index + 1]}"
      if [[ -z "${carrier_value}" || "${carrier_value}" == -* ]]; then
        invalid=1
        continue
      fi
      index=$((index + 1))
      continue
    fi
    if [[ "${argument}" == "${carrier_flag}="* ]]; then
      if [[ "${after_end_of_options}" -eq 1 ]]; then
        invalid=1
        continue
      fi
      carrier_count=$((carrier_count + 1))
      carrier_value="${argument#*=}"
      if [[ "${carrier_count}" -gt 1 || -z "${carrier_value}" ]]; then
        invalid=1
      fi
      continue
    fi
    if [[ "${argument}" == "${carrier_flag}"* ]]; then
      invalid=1
    fi
  done

  if [[ "${invalid}" -eq 1 || "${carrier_count}" -ne 1 ]]; then
    printf '%s\n' \
      '[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments' >&2
    return 2
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    local -a display_argv=()
    local -a display_is_redacted=()
    for ((index = 0; index < argc; index += 1)); do
      argument="${argv[index]}"
      if [[ "${argument}" == "${carrier_flag}" ]]; then
        display_argv+=("${carrier_flag}" "${placeholder}")
        display_is_redacted+=(0 1)
        index=$((index + 1))
      elif [[ "${argument}" == "${carrier_flag}="* ]]; then
        display_argv+=("${carrier_flag}=${placeholder}")
        display_is_redacted+=(1)
      else
        display_argv+=("${argument}")
        display_is_redacted+=(0)
      fi
    done

    printf '[%s][dry-run]' "${INSTALLER_NAME}" >&2
    for index in "${!display_argv[@]}"; do
      if [[ "${display_is_redacted[index]}" -eq 1 ]]; then
        printf ' %s' "${display_argv[index]}" >&2
      else
        printf ' %q' "${display_argv[index]}" >&2
      fi
    done
    printf '\n' >&2
    return 0
  fi

  "$@"
}

command_exists() {
  local cmd="$1"
  if command -v "${cmd}" >/dev/null 2>&1; then
    return 0
  fi

  local fallback=""
  case "${cmd}" in
    nft|ip)
      fallback="/usr/sbin/${cmd}"
      ;;
    systemctl)
      fallback="/usr/bin/systemctl"
      ;;
  esac

  if [[ -n "${fallback}" && -x "${fallback}" ]]; then
    return 0
  fi

  return 1
}

require_cmd() {
  local cmd="$1"
  command_exists "${cmd}" || fatal "Required command '${cmd}' not found. Please install it and re-run."
}

is_supported_pkg_manager() {
  local candidate="$1"
  local manager=""

  for manager in "${SUPPORTED_PKG_MANAGERS[@]}"; do
    if [[ "${candidate}" == "${manager}" ]]; then
      return 0
    fi
  done

  return 1
}

supported_pkg_manager_list() {
  local manager=""
  local first=1

  for manager in "${SUPPORTED_PKG_MANAGERS[@]}"; do
    if [[ "${first}" -eq 1 ]]; then
      printf '%s' "${manager}"
      first=0
    else
      printf ', %s' "${manager}"
    fi
  done
}

path_probe_detected_manager_list() {
  local manager=""
  local first=1

  for manager in "${PATH_PROBE_DETECTED_MANAGERS[@]}"; do
    if [[ "${first}" -eq 1 ]]; then
      printf '%s' "${manager}"
      first=0
    else
      printf ', %s' "${manager}"
    fi
  done
}

fail_invalid_explicit_pkg_manager() {
  local source_name="$1"
  local invalid_value="$2"

  printf '[%s][ERROR] Invalid %s value %q. Allowed values: %s. Re-run with one of the allowed values or remove the invalid override.\n' \
    "${INSTALLER_NAME}" \
    "${source_name}" \
    "${invalid_value}" \
    "$(supported_pkg_manager_list)" >&2
  exit 2
}

fail_missing_explicit_pkg_manager() {
  local source_name="$1"
  local selected_manager="$2"

  printf '[%s][ERROR] Selected package manager %q from %s was not found in PATH. Install that manager or rerun with another allowed manager (%s).\n' \
    "${INSTALLER_NAME}" \
    "${selected_manager}" \
    "${source_name}" \
    "$(supported_pkg_manager_list)" >&2
  exit 3
}

fail_no_supported_pkg_manager() {
  local missing_cmds=("$@")

  printf '[%s][ERROR] No supported package manager was detected. Missing prerequisite commands for this installer branch: %s. Install them manually and rerun. You may also rerun with --pkg-manager <%s> or PKG_MANAGER=<%s>.\n' \
    "${INSTALLER_NAME}" \
    "${missing_cmds[*]}" \
    'apt-get|dnf|yum|pacman|zypper' \
    'apt-get|dnf|yum|pacman|zypper' >&2
  exit 4
}

detect_primary_user() {
  if [[ -n "${INSTALL_BOOTSTRAP_ACCOUNT:-}" ]]; then
    printf '%s\n' "${INSTALL_BOOTSTRAP_ACCOUNT}"
    return
  fi
  if [[ -n "${SUBSTRATE_INSTALL_PRIMARY_USER:-}" ]]; then
    printf '%s\n' "${SUBSTRATE_INSTALL_PRIMARY_USER}"
    return
  fi
  if [[ -n "${SUDO_USER:-}" ]]; then
    printf '%s\n' "${SUDO_USER}"
    return
  fi
  if [[ "${EUID}" -ne 0 ]]; then
    if command -v id >/dev/null 2>&1; then
      id -un 2>/dev/null || true
      return
    fi
    if [[ -n "${USER:-}" ]]; then
      printf '%s\n' "${USER}"
      return
    fi
  fi
  printf ''
}

bootstrap_private_substrate_home() {
  local substrate_bin="$1"
  local _primary_user="${2:-}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] SUBSTRATE_HOME=%s SUBSTRATE_ROOT=%s %s --install-bootstrap-context-v1 <carrier> --install-bootstrap-home-v1\n' \
      "${INSTALLER_NAME}" "${PREFIX}" "${PREFIX}" "${substrate_bin}" >&2
    return 0
  fi
  if [[ ! -x "${substrate_bin}" ]]; then
    fatal "Substrate bootstrap binary not found at ${substrate_bin}."
  fi

  local bootstrap_rc=0
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  env \
    "SUBSTRATE_HOME=${PREFIX}" \
    "SUBSTRATE_ROOT=${PREFIX}" \
    "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${INSTALL_BOOTSTRAP_COMMITMENT}" \
    "SUBSTRATE_INSTALL_PRIMARY_USER=${INSTALL_BOOTSTRAP_ACCOUNT}" \
    "SUBSTRATE_INSTALL_PRIMARY_UID=${INSTALL_BOOTSTRAP_UID}" \
    "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    "${substrate_bin}" \
      --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
      --install-bootstrap-home-v1 >/dev/null || bootstrap_rc=$?
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${bootstrap_rc}" -ne 0 ]]; then
    fatal "Private SUBSTRATE_HOME bootstrap rejected ${PREFIX}; no existing root was repaired."
  fi
}

user_in_group_linux() {
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

record_group_existence() {
  if [[ "${PLATFORM:-}" != "linux" ]]; then
    return
  fi
  if [[ -n "${HOST_STATE_GROUP_EXISTED}" ]]; then
    return
  fi
  if command_exists getent; then
    if getent group substrate >/dev/null 2>&1; then
      HOST_STATE_GROUP_EXISTED="true"
    else
      HOST_STATE_GROUP_EXISTED="false"
    fi
  else
    HOST_STATE_GROUP_EXISTED="unknown"
  fi
}

record_group_created() {
  HOST_STATE_GROUP_CREATED=1
}

record_user_added() {
  local user="$1"
  if [[ -z "${user}" ]]; then
    return
  fi
  for existing in "${HOST_STATE_ADDED_USERS[@]:-}"; do
    if [[ "${existing}" == "${user}" ]]; then
      return
    fi
  done
  HOST_STATE_ADDED_USERS+=("${user}")
}

record_linger_state() {
  local user="$1"
  local state="$2"
  local enabled="${3:-0}"
  if [[ -z "${user}" ]]; then
    return
  fi
  local updated=0
  for idx in "${!HOST_STATE_LINGER_ENTRIES[@]}"; do
    IFS=':' read -r existing_user existing_state existing_enabled <<<"${HOST_STATE_LINGER_ENTRIES[$idx]}"
    if [[ "${existing_user}" == "${user}" ]]; then
      local new_state="${existing_state:-unknown}"
      if [[ -n "${state}" ]]; then
        new_state="${state}"
      fi
      local new_enabled="${existing_enabled:-0}"
      if [[ "${enabled}" -eq 1 ]]; then
        new_enabled="1"
      fi
      HOST_STATE_LINGER_ENTRIES[idx]="${user}:${new_state}:${new_enabled}"
      updated=1
      break
    fi
  done
  if [[ "${updated}" -eq 0 ]]; then
    local normalized_state="${state:-unknown}"
    local normalized_enabled=0
    if [[ "${enabled}" -eq 1 ]]; then
      normalized_enabled=1
    fi
    HOST_STATE_LINGER_ENTRIES+=("${user}:${normalized_state}:${normalized_enabled}")
  fi
}

write_host_state_metadata() {
  local world_enabled="${1:-1}"
  if [[ "${PLATFORM:-}" != "linux" ]]; then
    return
  fi
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "Skipping host state metadata during dry-run (${HOST_STATE_PATH:-unset})."
    return
  fi
  if [[ -z "${HOST_STATE_PATH}" ]]; then
    return
  fi
  if ! command_exists python3; then
    warn "python3 not found; skipping host state metadata recording (${HOST_STATE_PATH})."
    return
  fi
  detect_platform_metadata || true

  local events=()
  if [[ -n "${HOST_STATE_GROUP_EXISTED}" ]]; then
    events+=("group_preexisting:${HOST_STATE_GROUP_EXISTED}")
  fi
  if [[ "${HOST_STATE_GROUP_CREATED}" -eq 1 ]]; then
    events+=("group_created:true")
  fi
  for user in "${HOST_STATE_ADDED_USERS[@]:-}"; do
    events+=("user_added:${user}")
  done
  for entry in "${HOST_STATE_LINGER_ENTRIES[@]:-}"; do
    events+=("linger:${entry}")
  done
  if [[ -n "${PKG_MANAGER}" && -n "${PKG_MANAGER_SOURCE}" ]]; then
    events+=("platform_os_release_id:${DETECTED_DISTRO_ID:-${DISTRO_UNKNOWN_SENTINEL}}")
    events+=("platform_os_release_id_like:${DETECTED_DISTRO_ID_LIKE:-${DISTRO_UNKNOWN_SENTINEL}}")
    events+=("platform_pkg_manager_selected:${PKG_MANAGER}")
    events+=("platform_pkg_manager_source:${PKG_MANAGER_SOURCE}")
  fi

  local event_payload
  event_payload="$(printf '%s\n' "${events[@]}")"
  mkdir -p "$(dirname "${HOST_STATE_PATH}")" || true
  local tmp="${HOST_STATE_PATH}.tmp"
  if ! STATE_EVENTS="${event_payload}" python3 - "${HOST_STATE_PATH}" > "${tmp}" <<'PY'
import datetime
import json
import os
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
events = [line.strip() for line in os.environ.get("STATE_EVENTS", "").splitlines() if line.strip()]
schema_version = 1
timestamp = datetime.datetime.now(datetime.timezone.utc).isoformat().replace("+00:00", "Z")

base = {}
parsed_existing = False
if path.exists():
    try:
        with path.open() as f:
            base = json.load(f)
        parsed_existing = True
    except Exception as exc:  # noqa: BLE001
        sys.stderr.write(f"[substrate-install] warning: unable to parse {path}: {exc}\n")
        base = {}

if parsed_existing and base.get("schema_version") != schema_version:
    sys.stderr.write(
        f"[substrate-install] warning: unsupported schema_version {base.get('schema_version')} at {path}; rebuilding metadata\n"
    )
    base = {}

base["schema_version"] = schema_version
base.setdefault("created_at", timestamp)
base["updated_at"] = timestamp

host = base.setdefault("host_state", {})
group = host.setdefault("group", {"name": "substrate", "members_added": []})
group.setdefault("name", "substrate")
members = {m for m in group.get("members_added", []) if isinstance(m, str)}
linger = host.setdefault("linger", {})
linger_users = linger.setdefault("users", {})
platform = host.get("platform") or {}
os_release = platform.get("os_release") or {}
pkg_manager = platform.get("pkg_manager") or {}


def parse_bool(raw: str):
    lowered = raw.lower()
    if lowered in ("true", "1", "yes"):
        return True
    if lowered in ("false", "0", "no"):
        return False
    return None


for raw_event in events:
    parts = raw_event.split(":", 3)
    if not parts:
        continue
    kind = parts[0]
    if kind == "group_preexisting" and len(parts) >= 2:
        val = parse_bool(parts[1])
        if val is not None:
            group["existed_before"] = val
        elif "existed_before" not in group:
            group["existed_before"] = None
    elif kind == "group_created" and len(parts) >= 2:
        val = parse_bool(parts[1])
        if val is not None:
            group["created_by_installer"] = val
    elif kind == "user_added" and len(parts) >= 2:
        user = parts[1]
        if user:
            members.add(user)
    elif kind == "linger" and len(parts) >= 4:
        user, state, enabled_flag = parts[1], parts[2], parts[3]
        if not user:
            continue
        entry = linger_users.setdefault(user, {})
        if state:
            entry.setdefault("state_at_install", state)
            entry["state_at_install"] = state
        enabled_val = parse_bool(enabled_flag)
        if enabled_val is not None:
            entry["enabled_by_substrate"] = enabled_val
        elif "enabled_by_substrate" not in entry:
            entry["enabled_by_substrate"] = False
    elif kind == "platform_os_release_id" and len(parts) >= 2:
        os_release["id"] = parts[1]
    elif kind == "platform_os_release_id_like" and len(parts) >= 2:
        os_release["id_like"] = parts[1]
    elif kind == "platform_pkg_manager_selected" and len(parts) >= 2:
        pkg_manager["selected"] = parts[1]
    elif kind == "platform_pkg_manager_source" and len(parts) >= 2:
        pkg_manager["source"] = parts[1]

group["members_added"] = sorted(members)
if os_release:
    platform["os_release"] = os_release
if pkg_manager:
    platform["pkg_manager"] = pkg_manager
if platform:
    host["platform"] = platform
json.dump(base, sys.stdout, indent=2, sort_keys=True)
PY
  then
    warn "Failed to write host state metadata to ${HOST_STATE_PATH}; continuing without blocking install."
    rm -f "${tmp}" || true
    return
  fi

  if ! mv "${tmp}" "${HOST_STATE_PATH}"; then
    warn "Failed to replace host state metadata at ${HOST_STATE_PATH}; continuing without blocking install."
    rm -f "${tmp}" || true
    return
  fi
  chmod 0644 "${HOST_STATE_PATH}" || true
  log "Host state metadata recorded at ${HOST_STATE_PATH}"
}

ensure_linux_group_membership() {
  local target_user="$1"
  local target_group="substrate"

  initialize_sudo
  record_group_existence

  if ! getent group "${target_group}" >/dev/null 2>&1; then
    log "Creating '${target_group}' group (sudo may prompt)..."
    if ! run_with_sudo groupadd --system "${target_group}"; then
      warn "Unable to create ${target_group} group automatically. Run 'sudo groupadd --system ${target_group}' and re-run the installer."
      return
    else
      record_group_created
    fi
  fi

  if [[ -z "${target_user}" || "${target_user}" == "root" ]]; then
    warn "Could not determine which non-root user should join the '${target_group}' group. Run 'sudo usermod -aG ${target_group} <user>' manually if socket access is required."
    return
  fi

  if user_in_group_linux "${target_user}" "${target_group}"; then
    log "${target_user} already belongs to ${target_group}."
    return
  fi

  log "Adding ${target_user} to ${target_group} (sudo may prompt)..."
  if run_with_sudo usermod -aG "${target_group}" "${target_user}"; then
    warn "${target_user} added to ${target_group}. Log out/in or run 'newgrp ${target_group}' so group membership applies to new shells."
    record_user_added "${target_user}"
  else
    warn "Failed to add ${target_user} to ${target_group}; run 'sudo usermod -aG ${target_group} ${target_user}' manually."
  fi
}

print_linger_guidance_linux() {
  local target_user="$1"
  if [[ -z "${target_user}" || "${target_user}" == "root" ]]; then
    cat <<'MSG'
[substrate-install] loginctl: Unable to detect a non-root user for lingering.
Run 'loginctl enable-linger <user>' so socket-activated services start after logout.
MSG
    record_linger_state "${target_user}" "unknown" 0
    return
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    cat <<MSG
[substrate-install][dry-run] Would check lingering for ${target_user}. Ensure lingering
is enabled later via: loginctl enable-linger ${target_user}
MSG
    record_linger_state "${target_user}" "unknown" 0
    return
  fi

  if ! command_exists loginctl; then
    cat <<MSG
[substrate-install] loginctl not found. To keep the socket active after logout, run:
  loginctl enable-linger ${target_user}
MSG
    record_linger_state "${target_user}" "unknown" 0
    return
  fi

  local linger_state
  linger_state="$(loginctl show-user "${target_user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
  record_linger_state "${target_user}" "${linger_state:-unknown}" 0
  if [[ "${linger_state}" == "yes" ]]; then
    log "loginctl reports lingering already enabled for ${target_user}."
  else
    cat <<MSG
[substrate-install] loginctl status for ${target_user}: ${linger_state:-unknown}
Enable lingering to allow systemd to start the world-service socket after logout:
  loginctl enable-linger ${target_user}
MSG
  fi
}

initialize_sudo() {
  if [[ ${#SUDO_CMD[@]} -gt 0 ]]; then
    return
  fi

  if [[ "${EUID}" -ne 0 ]]; then
    if command -v sudo >/dev/null 2>&1; then
      SUDO_CMD=(sudo)
    else
      fatal "This installer requires 'sudo' when run as a non-root user. Install sudo or re-run the installer as root."
    fi
  fi
}

run_with_sudo() {
  local trace_was_active=0
  if [[ $- == *x* ]]; then
    trace_was_active=1
    set +x
  fi
  if [[ -z "${PREFIX}" \
      || "${SUBSTRATE_HOME:-}" != "${PREFIX}" \
      || "${SUBSTRATE_ROOT:-}" != "${PREFIX}" \
      || "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" != "${INSTALL_BOOTSTRAP_COMMITMENT}" \
      || "${SUBSTRATE_INSTALL_PRIMARY_USER:-}" != "${INSTALL_BOOTSTRAP_ACCOUNT}" \
      || "${SUBSTRATE_INSTALL_PRIMARY_UID:-}" != "${INSTALL_BOOTSTRAP_UID}" \
      || "${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" != "${INSTALL_BOOTSTRAP_CONTEXT_V1}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    fatal "Install bootstrap context projection changed before privileged tool dispatch."
  fi
  initialize_sudo

  local tool="$1"
  shift
  local tool_path=""
  if [[ "${tool}" == */* ]]; then
    if [[ "${tool}" != "/usr/libexec/substrate/substrate-apply-socket-acl" ]]; then
      if [[ "${trace_was_active}" -eq 1 ]]; then
        set -x
      fi
      fatal "Unsupported absolute privileged tool path: ${tool}"
    fi
    tool_path="${tool}"
  else
    tool_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- "${tool}" 2>/dev/null || true)"
  fi
  if [[ -z "${tool_path}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    fatal "Unable to resolve privileged tool: ${tool}"
  fi
  local env_path=""
  env_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- env 2>/dev/null || true)"
  if [[ -z "${env_path}" ]]; then
    if [[ "${trace_was_active}" -eq 1 ]]; then
      set -x
    fi
    fatal "Unable to resolve privileged environment scrubber: env"
  fi
  local -a scrubbed_command=(
    "${env_path}" -i
    "PATH=${PRIVILEGED_TOOL_PATH}"
    "HOME=/root"
    "USER=root"
    "LOGNAME=root"
    "${tool_path}"
    "$@"
  )
  if [[ "${trace_was_active}" -eq 1 ]]; then
    set -x
  fi

  if [[ ${#SUDO_CMD[@]} -eq 0 ]]; then
    run_cmd "${scrubbed_command[@]}"
    return $?
  fi

  local true_path
  true_path="$(PATH="${PRIVILEGED_TOOL_PATH}" type -P -- true 2>/dev/null || true)"
  if [[ -z "${true_path}" ]]; then
    fatal "Unable to resolve privileged sudo probe tool: true"
  fi
  if "${SUDO_CMD[@]}" -n -- "${env_path}" -i "PATH=${PRIVILEGED_TOOL_PATH}" "${true_path}" >/dev/null 2>&1; then
    run_cmd "${SUDO_CMD[@]}" -n -- "${scrubbed_command[@]}"
    return $?
  fi

  if [[ ! -t 0 && ! -t 1 && ! -t 2 ]]; then
    fatal "This installer requires interactive sudo for '$*', but no TTY is available. Re-run from a terminal or pre-authenticate with 'sudo -v'."
  fi

  run_cmd "${SUDO_CMD[@]}" -- "${scrubbed_command[@]}"
}

reset_os_release_input_state() {
  OS_RELEASE_SELECTED_PATH=""
  OS_RELEASE_INPUT_STATE="unavailable"
  DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
  DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"
}

resolve_selected_os_release_input() {
  local selected_path="${SUBSTRATE_INSTALL_OS_RELEASE_PATH:-}"
  local os_release_fd

  reset_os_release_input_state

  if [[ -z "${selected_path}" ]]; then
    selected_path="/etc/os-release"
  fi

  if [[ "${selected_path}" != /* ]]; then
    return 1
  fi

  if [[ ! -f "${selected_path}" || ! -r "${selected_path}" ]]; then
    return 1
  fi

  if ! exec {os_release_fd}<"${selected_path}"; then
    return 1
  fi
  exec {os_release_fd}<&-

  OS_RELEASE_SELECTED_PATH="${selected_path}"
  OS_RELEASE_INPUT_STATE="selected"
  return 0
}

trim_ascii_whitespace() {
  local value="$1"
  value="${value#"${value%%[!$' \t\r']*}"}"
  value="${value%"${value##*[!$' \t\r']}"}"
  printf '%s' "${value}"
}

strip_matching_quotes() {
  local value="$1"
  if [[ ${#value} -ge 2 ]]; then
    case "${value:0:1}${value: -1}" in
      "''"|'""')
        value="${value:1:${#value}-2}"
        ;;
    esac
  fi
  printf '%s' "${value}"
}

parse_selected_os_release_fields() {
  local line=""
  local key=""
  local raw_value=""
  local normalized_value=""

  DETECTED_DISTRO_ID="${DISTRO_UNKNOWN_SENTINEL}"
  DETECTED_DISTRO_ID_LIKE="${DISTRO_UNKNOWN_SENTINEL}"

  if [[ "${OS_RELEASE_INPUT_STATE}" != "selected" || -z "${OS_RELEASE_SELECTED_PATH}" ]]; then
    return 1
  fi

  while IFS= read -r line || [[ -n "${line}" ]]; do
    if [[ "${line}" =~ ^[[:space:]]*$ ]]; then
      continue
    fi
    if [[ "${line}" =~ ^[[:space:]]*# ]]; then
      continue
    fi

    key="${line%%=*}"
    if [[ "${key}" == "${line}" ]]; then
      continue
    fi

    raw_value="${line#*=}"
    raw_value="$(trim_ascii_whitespace "${raw_value}")"
    normalized_value="$(strip_matching_quotes "${raw_value}")"
    normalized_value="${normalized_value,,}"
    if [[ -z "${normalized_value}" ]]; then
      normalized_value="${DISTRO_UNKNOWN_SENTINEL}"
    fi

    case "${key}" in
      ID)
        DETECTED_DISTRO_ID="${normalized_value}"
        ;;
      ID_LIKE)
        DETECTED_DISTRO_ID_LIKE="${normalized_value}"
        ;;
    esac
  done < "${OS_RELEASE_SELECTED_PATH}"

  return 0
}

os_release_id_like_has_token() {
  local needle="$1"
  local token=""

  if [[ -z "${needle}" || "${DETECTED_DISTRO_ID_LIKE}" == "${DISTRO_UNKNOWN_SENTINEL}" ]]; then
    return 1
  fi

  for token in ${DETECTED_DISTRO_ID_LIKE}; do
    if [[ "${token}" == "${needle}" ]]; then
      return 0
    fi
  done

  return 1
}

os_release_matches_debian_family() {
  case "${DETECTED_DISTRO_ID}" in
    debian|ubuntu|linuxmint|pop)
      return 0
      ;;
  esac

  os_release_id_like_has_token "debian" || os_release_id_like_has_token "ubuntu"
}

os_release_matches_fedora_rhel_family() {
  case "${DETECTED_DISTRO_ID}" in
    fedora|rhel|centos|rocky|almalinux|ol|amzn)
      return 0
      ;;
  esac

  os_release_id_like_has_token "fedora" || os_release_id_like_has_token "rhel"
}

os_release_matches_arch_family() {
  case "${DETECTED_DISTRO_ID}" in
    arch|manjaro|endeavouros|arcolinux|artix|garuda)
      return 0
      ;;
  esac

  os_release_id_like_has_token "arch"
}

os_release_matches_suse_family() {
  local token=""

  case "${DETECTED_DISTRO_ID}" in
    *suse*)
      return 0
      ;;
  esac

  if [[ "${DETECTED_DISTRO_ID_LIKE}" == "${DISTRO_UNKNOWN_SENTINEL}" ]]; then
    return 1
  fi

  for token in ${DETECTED_DISTRO_ID_LIKE}; do
    case "${token}" in
      *suse*)
        return 0
        ;;
    esac
  done

  return 1
}

select_package_manager_from_os_release() {
  PKG_MANAGER_SOURCE=""

  if os_release_matches_debian_family; then
    if command -v apt-get >/dev/null 2>&1; then
      PKG_MANAGER="apt-get"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_fedora_rhel_family; then
    if command -v dnf >/dev/null 2>&1; then
      PKG_MANAGER="dnf"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    if command -v yum >/dev/null 2>&1; then
      PKG_MANAGER="yum"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_arch_family; then
    if command -v pacman >/dev/null 2>&1; then
      PKG_MANAGER="pacman"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  if os_release_matches_suse_family; then
    if command -v zypper >/dev/null 2>&1; then
      PKG_MANAGER="zypper"
      PKG_MANAGER_SOURCE="os_release"
      return 0
    fi
    return 1
  fi

  return 1
}

select_package_manager_from_path_probe() {
  local manager=""
  local selected_manager=""

  PATH_PROBE_DETECTED_MANAGERS=()
  PKG_MANAGER_PATH_PROBE_WARNING_EMITTED=0

  for manager in "${SUPPORTED_PKG_MANAGERS[@]}"; do
    if command -v "${manager}" >/dev/null 2>&1; then
      PATH_PROBE_DETECTED_MANAGERS+=("${manager}")
      if [[ -z "${selected_manager}" ]]; then
        selected_manager="${manager}"
      fi
    fi
  done

  if [[ -z "${selected_manager}" ]]; then
    return 1
  fi

  PKG_MANAGER="${selected_manager}"
  PKG_MANAGER_SOURCE="path_probe"
  maybe_emit_path_probe_multi_manager_warning
  return 0
}

detect_platform_metadata() {
  if [[ "${PLATFORM:-}" != "linux" ]]; then
    return 1
  fi

  PKG_MANAGER=""
  PKG_MANAGER_SOURCE=""
  resolve_selected_os_release_input || true
  parse_selected_os_release_fields || true

  if select_package_manager_from_os_release; then
    return 0
  fi

  if select_package_manager_from_path_probe; then
    return 0
  fi

  return 1
}

select_package_manager_from_flag() {
  if [[ -z "${PKG_MANAGER_FLAG_OVERRIDE}" ]]; then
    return 1
  fi

  if ! is_supported_pkg_manager "${PKG_MANAGER_FLAG_OVERRIDE}"; then
    fail_invalid_explicit_pkg_manager "--pkg-manager" "${PKG_MANAGER_FLAG_OVERRIDE}"
  fi

  PKG_MANAGER="${PKG_MANAGER_FLAG_OVERRIDE}"
  PKG_MANAGER_SOURCE="flag"

  if ! command -v "${PKG_MANAGER_FLAG_OVERRIDE}" >/dev/null 2>&1; then
    maybe_emit_package_manager_decision_line
    fail_missing_explicit_pkg_manager "--pkg-manager" "${PKG_MANAGER_FLAG_OVERRIDE}"
  fi

  return 0
}

select_package_manager_from_env() {
  if [[ -z "${PKG_MANAGER_ENV_OVERRIDE}" ]]; then
    return 1
  fi

  if ! is_supported_pkg_manager "${PKG_MANAGER_ENV_OVERRIDE}"; then
    fail_invalid_explicit_pkg_manager "PKG_MANAGER" "${PKG_MANAGER_ENV_OVERRIDE}"
  fi

  PKG_MANAGER="${PKG_MANAGER_ENV_OVERRIDE}"
  PKG_MANAGER_SOURCE="env"

  if ! command -v "${PKG_MANAGER_ENV_OVERRIDE}" >/dev/null 2>&1; then
    maybe_emit_package_manager_decision_line
    fail_missing_explicit_pkg_manager "PKG_MANAGER" "${PKG_MANAGER_ENV_OVERRIDE}"
  fi

  return 0
}

detect_package_manager() {
  if [[ -n "${PKG_MANAGER}" && -n "${PKG_MANAGER_SOURCE}" ]]; then
    return 0
  fi

  resolve_selected_os_release_input || true
  parse_selected_os_release_fields || true

  if select_package_manager_from_flag; then
    return 0
  fi

  if select_package_manager_from_env; then
    return 0
  fi

  if select_package_manager_from_os_release; then
    return 0
  fi

  if select_package_manager_from_path_probe; then
    return 0
  fi

  return 1
}

maybe_emit_package_manager_decision_line() {
  if [[ -z "${PKG_MANAGER}" ]]; then
    return
  fi

  case "${PKG_MANAGER_SOURCE}" in
    flag|env|os_release|path_probe)
      ;;
    *)
      return
      ;;
  esac

  if [[ "${PKG_MANAGER_DECISION_LINE_EMITTED}" -eq 1 ]]; then
    return
  fi

  printf 'Detected distro: %s (like: %s), using package manager: %s (source: %s)\n' \
    "${DETECTED_DISTRO_ID}" \
    "${DETECTED_DISTRO_ID_LIKE}" \
    "${PKG_MANAGER}" \
    "${PKG_MANAGER_SOURCE}" >&2
  PKG_MANAGER_DECISION_LINE_EMITTED=1
}

maybe_emit_path_probe_multi_manager_warning() {
  local manager_list=""

  if [[ "${PKG_MANAGER_SOURCE}" != "path_probe" ]]; then
    return
  fi

  if [[ "${PKG_MANAGER_PATH_PROBE_WARNING_EMITTED}" -eq 1 ]]; then
    return
  fi

  if [[ ${#PATH_PROBE_DETECTED_MANAGERS[@]} -le 1 ]]; then
    return
  fi

  manager_list="$(path_probe_detected_manager_list)"
  printf 'Multiple supported package managers found in PATH: %s; selecting %s by fixed probe order (apt-get -> dnf -> yum -> pacman -> zypper). Override with --pkg-manager <apt-get|dnf|yum|pacman|zypper> or PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>.\n' \
    "${manager_list}" \
    "${PKG_MANAGER}" >&2
  PKG_MANAGER_PATH_PROBE_WARNING_EMITTED=1
}

resolve_package_for_command() {
  local cmd="$1"

  case "${PKG_MANAGER}" in
    apt-get)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs fuse3" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    dnf|yum)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    pacman)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    zypper)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    *)
      echo ""
      ;;
  esac
}

resolve_package_for_runtime_library() {
  local library="$1"

  case "${PKG_MANAGER}" in
    apt-get)
      case "${library}" in
        libseccomp) echo "libseccomp2" ;;
        *) echo "" ;;
      esac
      ;;
    dnf|yum)
      case "${library}" in
        libseccomp) echo "libseccomp" ;;
        *) echo "" ;;
      esac
      ;;
    pacman)
      case "${library}" in
        libseccomp) echo "libseccomp" ;;
        *) echo "" ;;
      esac
      ;;
    zypper)
      case "${library}" in
        libseccomp) echo "libseccomp2" ;;
        *) echo "" ;;
      esac
      ;;
    *)
      echo ""
      ;;
  esac
}

seccomp_runtime_available() {
  if command -v ldconfig >/dev/null 2>&1; then
    if ldconfig -p 2>/dev/null | grep -Eq 'libseccomp\.so(\.2)?([[:space:]]|$)'; then
      return 0
    fi
  fi

  if compgen -G '/lib*/libseccomp.so*' >/dev/null; then
    return 0
  fi
  if compgen -G '/usr/lib*/libseccomp.so*' >/dev/null; then
    return 0
  fi

  return 1
}

install_packages() {
  local packages=()
  packages=("$@")
  if [[ ${#packages[@]} -eq 0 ]]; then
    return
  fi

  case "${PKG_MANAGER}" in
    apt-get)
      log "Installing packages: ${packages[*]}"
      if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf '[%s][dry-run] %s apt-get update\n' "${INSTALLER_NAME}" "${SUDO_CMD[*]:-}" >&2
        printf '[%s][dry-run] %s apt-get install -y %s\n' "${INSTALLER_NAME}" "${SUDO_CMD[*]:-}" "${packages[*]}" >&2
        return
      fi
      if [[ ${APT_UPDATED} -eq 0 ]]; then
        run_with_sudo apt-get update
        APT_UPDATED=1
      fi
      run_with_sudo apt-get install -y "${packages[@]}"
      ;;
    dnf)
      log "Installing packages: ${packages[*]}"
      run_with_sudo dnf install -y "${packages[@]}"
      ;;
    yum)
      log "Installing packages: ${packages[*]}"
      run_with_sudo yum install -y "${packages[@]}"
      ;;
    pacman)
      log "Installing packages: ${packages[*]}"
      run_with_sudo pacman -Sy --noconfirm --needed "${packages[@]}"
      ;;
    zypper)
      log "Installing packages: ${packages[*]}"
      run_with_sudo zypper --non-interactive install "${packages[@]}"
      ;;
    *)
      fatal "Unsupported package manager. Install required commands manually and re-run."
      ;;
  esac
}

ensure_linux_packages_for_commands() {
  local commands=("$@")
  local missing_cmds=()
  for cmd in "${commands[@]}"; do
    if ! command_exists "${cmd}"; then
      missing_cmds+=("${cmd}")
    fi
  done

  if [[ ${#missing_cmds[@]} -eq 0 ]]; then
    return
  fi

  if ! detect_package_manager; then
    fail_no_supported_pkg_manager "${missing_cmds[@]}"
  fi

  initialize_sudo
  maybe_emit_package_manager_decision_line

  declare -A pkg_set=()
  local cmd pkg_list
  for cmd in "${missing_cmds[@]}"; do
    pkg_list="$(resolve_package_for_command "${cmd}")"
    if [[ -z "${pkg_list}" ]]; then
      warn "No package mapping for '${cmd}' under ${PKG_MANAGER}; please install it manually."
      continue
    fi
    for pkg in ${pkg_list}; do
      pkg_set["${pkg}"]=1
    done
  done

  if [[ ${#pkg_set[@]} -eq 0 ]]; then
    return
  fi

  local packages=()
  for pkg in "${!pkg_set[@]}"; do
    packages+=("${pkg}")
  done

  install_packages "${packages[@]}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return
  fi

  # Re-check after installation.
  local remaining=()
  for cmd in "${missing_cmds[@]}"; do
    if ! command_exists "${cmd}"; then
      remaining+=("${cmd}")
    fi
  done
  if [[ ${#remaining[@]} -gt 0 ]]; then
    fatal "Unable to install required commands: ${remaining[*]}. Install them manually and re-run."
  fi
}

ensure_linux_packages_for_runtime_libraries() {
  local libraries=("$@")
  local missing_libraries=()
  local library=""

  for library in "${libraries[@]}"; do
    case "${library}" in
      libseccomp)
        if ! seccomp_runtime_available; then
          missing_libraries+=("${library}")
        fi
        ;;
      *)
        warn "No runtime library probe implemented for '${library}'; please install it manually if required."
        ;;
    esac
  done

  if [[ ${#missing_libraries[@]} -eq 0 ]]; then
    return
  fi

  if ! detect_package_manager; then
    fail_no_supported_pkg_manager "${missing_libraries[@]}"
  fi

  initialize_sudo
  maybe_emit_package_manager_decision_line

  declare -A pkg_set=()
  local pkg_list pkg
  for library in "${missing_libraries[@]}"; do
    pkg_list="$(resolve_package_for_runtime_library "${library}")"
    if [[ -z "${pkg_list}" ]]; then
      warn "No package mapping for runtime library '${library}' under ${PKG_MANAGER}; please install it manually."
      continue
    fi
    for pkg in ${pkg_list}; do
      pkg_set["${pkg}"]=1
    done
  done

  if [[ ${#pkg_set[@]} -eq 0 ]]; then
    return
  fi

  local packages=()
  for pkg in "${!pkg_set[@]}"; do
    packages+=("${pkg}")
  done

  install_packages "${packages[@]}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return
  fi

  local remaining=()
  for library in "${missing_libraries[@]}"; do
    case "${library}" in
      libseccomp)
        if ! seccomp_runtime_available; then
          remaining+=("${library}")
        fi
        ;;
    esac
  done
  if [[ ${#remaining[@]} -gt 0 ]]; then
    fatal "Unable to install required runtime libraries: ${remaining[*]}. Install them manually and re-run."
  fi
}

compute_file_sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file}" | awk '{print $1}'
  else
    fatal "Neither sha256sum nor shasum found; cannot verify checksums."
  fi
}

sanitize_env_path() {
  if [[ -n "${SHIM_ORIGINAL_PATH:-}" ]]; then
    PATH="${SHIM_ORIGINAL_PATH}"
  else
    local shim_dir="${HOME}/.substrate/shims"
    local IFS=':'
    local parts=()
    if [[ -n "${PATH}" ]]; then
      IFS=':' read -r -a parts <<< "${PATH}"
    fi
    local filtered=()
    for entry in "${parts[@]}"; do
      if [[ "${entry}" == "${shim_dir}" ]]; then
        continue
      fi
      filtered+=("${entry}")
    done
    PATH="$(IFS=':'; printf '%s' "${filtered[*]}")"
  fi
  export PATH
  ORIGINAL_PATH="${PATH}"
}

detect_platform() {
  local uname_s
  uname_s="$(uname -s)"
  ARCH="$(uname -m)"

  case "${uname_s}" in
    Darwin)
      PLATFORM="macos"
      ;;
    Linux)
      PLATFORM="linux"
      if grep -qi microsoft /proc/version 2>/dev/null; then
        IS_WSL=1
      fi
      ;;
    MINGW*|MSYS*|CYGWIN*)
      PLATFORM="windows"
      ;;
    *)
      fatal "Unsupported operating system: ${uname_s}"
      ;;
  esac
}

ensure_supported_linux_world_posture() {
  if [[ "${PLATFORM}" != "linux" || "${IS_WSL}" -ne 1 || "${NO_WORLD}" -eq 1 ]]; then
    return
  fi

  fatal_with_code 4 "WSL world provisioning is intentionally fail-closed in this slice because the WSL helper path is not aligned with the Linux/macOS placement contract. Re-run with --no-world for a CLI-only install inside WSL, or use a supported Linux host-native or macOS Lima world backend."
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)
        [[ $# -lt 2 ]] && fatal "Missing value for --version"
        VERSION_RAW="$2"
        shift 2
        ;;
      --prefix)
        [[ $# -lt 2 ]] && fatal "Missing value for --prefix"
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
        [[ $# -lt 2 ]] && fatal "Missing value for --install-bootstrap-context-v1"
        [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 1 ]] && fatal "Duplicate --install-bootstrap-context-v1"
        [[ -z "$2" ]] && fatal "Empty value for --install-bootstrap-context-v1"
        INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
        INSTALL_BOOTSTRAP_CONTEXT_DECLARED=1
        shift 2
        ;;
      --pkg-manager)
        [[ $# -lt 2 ]] && fatal "Missing value for --pkg-manager"
        PKG_MANAGER_FLAG_OVERRIDE="$2"
        shift 2
        ;;
      --no-world)
        NO_WORLD=1
        shift
        ;;
      --no-shims)
        NO_SHIMS=1
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        shift
        ;;
      --sync-deps)
        SYNC_DEPS=1
        shift
        ;;
      --provision-agent-runtime)
        [[ $# -lt 2 ]] && fatal "Missing value for --provision-agent-runtime"
        PROVISION_AGENT_RUNTIME="$2"
        shift 2
        ;;
      --world-netfilter)
        ENABLE_WORLD_NETFILTER=1
        shift
        ;;
      --artifact-dir|--archive)
        [[ $# -lt 2 ]] && fatal "Missing value for $1"
        ARTIFACT_DIR="$2"
        shift 2
        ;;
      -h|--help)
        HELP_REQUESTED=1
        shift
        ;;
      *)
        fatal "Unknown option: $1"
        ;;
    esac
  done
}

supported_agent_runtime_list() {
  printf 'codex'
}

world_deps_item_for_agent_runtime() {
  local runtime_family="$1"

  case "${runtime_family}" in
    codex)
      printf 'codex-runtime\n'
      ;;
    *)
      fatal_with_code 2 "Unsupported value for --provision-agent-runtime: '${runtime_family}'. This slice supports: $(supported_agent_runtime_list)."
      ;;
  esac
}

validate_agent_runtime_provision_request() {
  if [[ -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return
  fi

  world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}" >/dev/null

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    fatal_with_code 2 "--provision-agent-runtime requires world provisioning. Remove --no-world or omit the runtime flag."
  fi

  SYNC_DEPS=1
}

agent_runtime_retry_after_sync_failure() {
  local deps_item
  deps_item="$(world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}")"
  if [[ "${INSTALLER_NAME}" == "substrate-world-enable" ]]; then
    printf "re-run the world-enable helper with '--provision-agent-runtime %s' to re-add '%s' and retry the sync" "${PROVISION_AGENT_RUNTIME}" "${deps_item}"
    return
  fi

  printf "re-run the installer with '--provision-agent-runtime %s' to re-add '%s' and retry the sync" "${PROVISION_AGENT_RUNTIME}" "${deps_item}"
}

world_deps_global_remove_scope_note() {
  printf 'It does not remove any guest-side state that may already have been applied in the world.'
}

fetch_latest_release_tag() {
  if ! command -v curl >/dev/null 2>&1; then
    return 1
  fi
  if ! command -v jq >/dev/null 2>&1; then
    return 1
  fi

  local curl_cmd=(curl -fsSL -H "Accept: application/vnd.github+json")
  if [[ -n "${SUBSTRATE_INSTALL_GITHUB_TOKEN:-}" ]]; then
    curl_cmd+=(-H "Authorization: Bearer ${SUBSTRATE_INSTALL_GITHUB_TOKEN}")
  fi

  local response
  if ! response="$("${curl_cmd[@]}" "${LATEST_RELEASE_API}")"; then
    return 1
  fi

  jq -r '.tag_name // empty' <<<"${response}"
}

ensure_version_selected() {
  if [[ -n "${VERSION_TAG}" ]]; then
    return
  fi

  if [[ -z "${VERSION_RAW}" ]]; then
    local resolved_tag=""
    if resolved_tag="$(fetch_latest_release_tag 2>/dev/null)" && [[ -n "${resolved_tag}" ]]; then
      VERSION_RAW="${resolved_tag}"
      log "No --version provided; defaulting to latest release ${resolved_tag}."
    else
      VERSION_RAW="v${DEFAULT_FALLBACK_VERSION}"
      warn "Unable to resolve latest release tag; falling back to ${VERSION_RAW}."
    fi
  else
    log "Using requested version ${VERSION_RAW}."
  fi

  VERSION="${VERSION_RAW#v}"
  if [[ -z "${VERSION}" ]]; then
    fatal "Unable to determine version from '${VERSION_RAW}'"
  fi
  VERSION_TAG="v${VERSION}"
}

prepare_tmpdir() {
  TMPDIR="$(mktemp -d -t substrate-install.XXXXXX)"
  trap cleanup EXIT
}

normalize_prefix() {
  if [[ "${PREFIX}" != "/" ]]; then
    PREFIX="${PREFIX%/}"
    if [[ -z "${PREFIX}" ]]; then
      PREFIX="/"
    fi
  fi
}

initialize_metadata_paths() {
  ENV_SH_PATH="${PREFIX}/env.sh"
  MANAGER_ENV_PATH="${PREFIX}/manager_env.sh"
  MANAGER_INIT_PATH="${PREFIX}/manager_init.sh"
  INSTALL_CONFIG_PATH="${PREFIX}/config.yaml"
  HOST_STATE_PATH="${PREFIX}/install_state.json"
}

ensure_no_legacy_toml_install_config() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return
  fi

  local legacy="${PREFIX}/config.toml"
  if [[ -f "${legacy}" ]]; then
    fatal "Unsupported legacy TOML config detected at ${legacy}. YAML config is now required at ${INSTALL_CONFIG_PATH}. Delete the TOML file and re-run the installer."
  fi
}

ensure_manager_init_placeholder() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Create manager_init placeholder at %s\n' "${INSTALLER_NAME}" "${MANAGER_INIT_PATH}" >&2
    return
  fi

  local init_dir
  init_dir="$(dirname "${MANAGER_INIT_PATH}")"
  mkdir -p "${init_dir}"
  if [[ -f "${MANAGER_INIT_PATH}" ]]; then
    return
  fi

  cat > "${MANAGER_INIT_PATH}.tmp" <<'EOF'
# Substrate manager init placeholder – this file is replaced at runtime by `substrate`.
EOF
  mv "${MANAGER_INIT_PATH}.tmp" "${MANAGER_INIT_PATH}"
  chmod 0644 "${MANAGER_INIT_PATH}" || true
}

write_manager_env_script() {
  local enabled="$1"
  local state="disabled"
  if [[ "${enabled}" -eq 1 ]]; then
    state="enabled"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Write manager_env.sh at %s (world_enabled=%s)\n' "${INSTALLER_NAME}" "${MANAGER_ENV_PATH}" "${state}" >&2
    return
  fi

  local env_dir
  env_dir="$(dirname "${MANAGER_ENV_PATH}")"
  mkdir -p "${env_dir}"
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  local today
  today="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  local substrate_home_literal commitment_literal account_literal uid_literal carrier_literal legacy_literal
  substrate_home_literal="$(printf '%q' "${PREFIX}")"
  commitment_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")"
  account_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")"
  uid_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")"
  carrier_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")"
  legacy_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}/.substrate_bashenv")"
  cat > "${MANAGER_ENV_PATH}.tmp" <<EOF
#!/usr/bin/env bash
# Managed by ${INSTALLER_NAME} on ${today}
if [[ -n "\${SUBSTRATE_MANAGER_ENV_ACTIVE:-}" ]]; then
  return 0
fi

expected_substrate_home=${substrate_home_literal}
expected_substrate_commitment=${commitment_literal}
expected_substrate_account=${account_literal}
expected_substrate_uid=${uid_literal}
expected_substrate_carrier=${carrier_literal}

verify_projection() {
  local projection_name="\$1"
  local projection_actual="\$2"
  local projection_expected="\$3"
  if [[ -n "\${projection_actual}" && "\${projection_actual}" != "\${projection_expected}" ]]; then
    printf '[substrate-manager-env][ERROR] conflicting %s projection\n' "\${projection_name}" >&2
    return 1
  fi
}

verify_projection SUBSTRATE_HOME "\${SUBSTRATE_HOME:-}" "\${expected_substrate_home}" || { return 1 2>/dev/null || exit 1; }
verify_projection SUBSTRATE_ROOT "\${SUBSTRATE_ROOT:-}" "\${expected_substrate_home}" || { return 1 2>/dev/null || exit 1; }
verify_projection SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT "\${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" "\${expected_substrate_commitment}" || { return 1 2>/dev/null || exit 1; }
verify_projection SUBSTRATE_INSTALL_PRIMARY_USER "\${SUBSTRATE_INSTALL_PRIMARY_USER:-}" "\${expected_substrate_account}" || { return 1 2>/dev/null || exit 1; }
verify_projection SUBSTRATE_INSTALL_PRIMARY_UID "\${SUBSTRATE_INSTALL_PRIMARY_UID:-}" "\${expected_substrate_uid}" || { return 1 2>/dev/null || exit 1; }
verify_projection SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 "\${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" "\${expected_substrate_carrier}" || { return 1 2>/dev/null || exit 1; }

substrate_home="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd -P)" || {
  printf '[substrate-manager-env][ERROR] unable to resolve manager projection directory\n' >&2
  return 1 2>/dev/null || exit 1
}
if [[ "\${substrate_home}" != "\${expected_substrate_home}" ]]; then
  printf '[substrate-manager-env][ERROR] manager projection directory does not match committed SUBSTRATE_HOME\n' >&2
  return 1 2>/dev/null || exit 1
fi

substrate_env="\${substrate_home}/env.sh"
if [[ ! -f "\${substrate_env}" ]]; then
  printf '[substrate-manager-env][ERROR] missing committed environment projection at %s\n' "\${substrate_env}" >&2
  return 1 2>/dev/null || exit 1
fi
# shellcheck disable=SC1090
if ! source "\${substrate_env}"; then
  printf '[substrate-manager-env][ERROR] committed environment projection failed at %s\n' "\${substrate_env}" >&2
  return 1 2>/dev/null || exit 1
fi
if [[ "\${SUBSTRATE_HOME:-}" != "\${expected_substrate_home}" \
  || "\${SUBSTRATE_ROOT:-}" != "\${expected_substrate_home}" \
  || "\${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT:-}" != "\${expected_substrate_commitment}" \
  || "\${SUBSTRATE_INSTALL_PRIMARY_USER:-}" != "\${expected_substrate_account}" \
  || "\${SUBSTRATE_INSTALL_PRIMARY_UID:-}" != "\${expected_substrate_uid}" \
  || "\${SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1:-}" != "\${expected_substrate_carrier}" ]]; then
  printf '[substrate-manager-env][ERROR] committed environment projection mismatch at %s\n' "\${substrate_env}" >&2
  return 1 2>/dev/null || exit 1
fi
export SUBSTRATE_MANAGER_ENV_ACTIVE=1

manager_init_path="\${substrate_home}/manager_init.sh"
if [[ -f "\${manager_init_path}" ]]; then
  # shellcheck disable=SC1090
  source "\${manager_init_path}"
fi

substrate_original="\${SUBSTRATE_ORIGINAL_BASH_ENV:-}"
if [[ -n "\${substrate_original}" && -f "\${substrate_original}" ]]; then
  # shellcheck disable=SC1090
  source "\${substrate_original}"
fi

legacy_bashenv=${legacy_literal}
if [[ -f "\${legacy_bashenv}" ]]; then
  # shellcheck disable=SC1090
  source "\${legacy_bashenv}"
fi
EOF
  mv "${MANAGER_ENV_PATH}.tmp" "${MANAGER_ENV_PATH}"
  chmod 0644 "${MANAGER_ENV_PATH}" || true
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
}

write_env_sh_script() {
  local enabled="$1"
  local state="disabled"
  if [[ "${enabled}" -eq 1 ]]; then
    state="enabled"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Write env.sh at %s (world=%s)\n' "${INSTALLER_NAME}" "${ENV_SH_PATH}" "${state}" >&2
    return
  fi

  local env_dir
  env_dir="$(dirname "${ENV_SH_PATH}")"
  mkdir -p "${env_dir}"
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi

  local substrate_home_literal commitment_literal account_literal uid_literal carrier_literal anchor_mode_literal anchor_path_literal policy_mode_literal world_literal
  substrate_home_literal="$(printf '%q' "${PREFIX}")"
  commitment_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")"
  account_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")"
  uid_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")"
  carrier_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")"
  anchor_mode_literal="$(printf '%q' "workspace")"
  anchor_path_literal="$(printf '%q' "")"
  policy_mode_literal="$(printf '%q' "observe")"
  world_literal="$(printf '%q' "${state}")"
  cat > "${ENV_SH_PATH}.tmp" <<EOF
#!/usr/bin/env bash
export SUBSTRATE_HOME=${substrate_home_literal}
export SUBSTRATE_ROOT=${substrate_home_literal}
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${commitment_literal}
export SUBSTRATE_INSTALL_PRIMARY_USER=${account_literal}
export SUBSTRATE_INSTALL_PRIMARY_UID=${uid_literal}
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${carrier_literal}
export SUBSTRATE_WORLD=${world_literal}
export SUBSTRATE_CAGED=1
export SUBSTRATE_ANCHOR_MODE=${anchor_mode_literal}
export SUBSTRATE_ANCHOR_PATH=${anchor_path_literal}
export SUBSTRATE_POLICY_MODE=${policy_mode_literal}
EOF
  mv "${ENV_SH_PATH}.tmp" "${ENV_SH_PATH}"
  chmod 0644 "${ENV_SH_PATH}" || true
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
}

write_install_config() {
  local enabled="$1"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    local enabled_flag="false"
    if [[ "${enabled}" -eq 1 ]]; then
      enabled_flag="true"
    fi
    printf '[%s][dry-run] Write install metadata to %s (world_enabled=%s)\n' "${INSTALLER_NAME}" "${INSTALL_CONFIG_PATH}" "${enabled_flag}" >&2
    return
  fi

  local config_dir
  config_dir="$(dirname "${INSTALL_CONFIG_PATH}")"
  mkdir -p "${config_dir}"

  cat > "${INSTALL_CONFIG_PATH}.tmp" <<'EOF'
# Substrate global config patch (sparse overrides).
# - This file is a YAML mapping of global-scoped overrides.
# - Omitted keys inherit from defaults.
EOF
  if [[ "${enabled}" -eq 1 ]]; then
    printf '{}\n' >> "${INSTALL_CONFIG_PATH}.tmp"
  else
    printf 'world:\n  enabled: false\n' >> "${INSTALL_CONFIG_PATH}.tmp"
  fi

  mv "${INSTALL_CONFIG_PATH}.tmp" "${INSTALL_CONFIG_PATH}"
  chmod 0600 "${INSTALL_CONFIG_PATH}" || true
}

finalize_install_metadata() {
  local enabled="$1"
  ensure_no_legacy_toml_install_config
  ensure_manager_init_placeholder
  write_env_sh_script "${enabled}"
  write_manager_env_script "${enabled}"
  write_install_config "${enabled}"
}

ensure_version_config_present() {
  local version_dir="$1"
  local config_dir="${version_dir}/config"
  local manager_manifest="${config_dir}/manager_hooks.yaml"
  local world_deps="${config_dir}/world-deps.yaml"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] ensure config manifests exist under %s\n' "${INSTALLER_NAME}" "${config_dir}" >&2
    printf '[%s][dry-run] project %s to %s\n' "${INSTALLER_NAME}" "${manager_manifest}" "${PREFIX}/manager_hooks.yaml" >&2
    return
  fi

  mkdir -p "${config_dir}"

  if [[ ! -f "${manager_manifest}" ]]; then
    fatal "manager manifest missing from bundle (expected ${manager_manifest})"
  fi

  cp "${manager_manifest}" "${PREFIX}/manager_hooks.yaml.tmp"
  mv "${PREFIX}/manager_hooks.yaml.tmp" "${PREFIX}/manager_hooks.yaml"
  chmod 0644 "${PREFIX}/manager_hooks.yaml" || true

  if [[ ! -f "${world_deps}" ]]; then
    local scripts_world_deps
    scripts_world_deps="${version_dir}/scripts/substrate/world-deps.yaml"
    if [[ -f "${scripts_world_deps}" ]]; then
      cp "${scripts_world_deps}" "${world_deps}"
      log "Staged world-deps manifest under ${config_dir}"
    else
      fatal "world-deps manifest missing from bundle (expected ${world_deps})"
    fi
  fi
}

ensure_macos_prereqs() {
  require_cmd sw_vers
  require_cmd sysctl
  require_cmd curl
  require_cmd tar
  require_cmd shasum
  require_cmd jq
  require_cmd limactl
  require_cmd envsubst

  local hv_support
  hv_support="$(sysctl -n kern.hv_support 2>/dev/null || true)"
  if [[ "${hv_support}" != "1" ]]; then
    fatal "macOS virtualization not available. Enable Virtualization Framework in System Settings."
  fi

  if [[ "${ARCH}" != "arm64" ]]; then
    fatal "Only macOS arm64 is currently supported."
  fi
}

ensure_linux_prereqs() {
  ensure_linux_packages_for_commands curl tar jq
  require_cmd curl
  require_cmd tar
  require_cmd jq

  if [[ "${EUID}" -ne 0 ]]; then
    if ! command_exists sudo; then
      fatal "This installer requires 'sudo' when run as a non-root user. Install sudo or re-run the installer as root."
    fi
  fi

  if ! command_exists sha256sum && ! command_exists shasum; then
    ensure_linux_packages_for_commands sha256sum
    if ! command_exists sha256sum && ! command_exists shasum; then
      fatal "Missing sha256sum (preferred) or shasum for checksum verification. Install coreutils/perl-Digest-SHA or rerun with --dry-run."
    fi
  fi

  if [[ "${NO_WORLD}" -eq 0 ]]; then
    ensure_linux_packages_for_commands systemctl fuse-overlayfs nft ip
    ensure_linux_packages_for_runtime_libraries libseccomp
    require_cmd systemctl
    require_cmd fuse-overlayfs
    require_cmd nft
    require_cmd ip

    local init_comm
    init_comm="$(ps -p 1 -o comm= 2>/dev/null || true)"
    if [[ "${init_comm}" != "systemd" ]]; then
      if [[ "${IS_WSL}" -eq 1 ]]; then
        fatal "WSL distribution not running systemd (pid 1: ${init_comm:-unknown}). Enable systemd in /etc/wsl.conf or re-run with --no-world."
      else
        fatal "Systemd is not PID 1 (detected '${init_comm:-unknown}'). Boot into a systemd-based userland or install with --no-world."
      fi
    fi
  fi
}

download_file() {
  local source="$1"
  local destination="$2"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] download %s -> %s\n' "${INSTALLER_NAME}" "${source}" "${destination}" >&2
    return 0
  fi

  local dir
  dir="$(dirname "${destination}")"
  mkdir -p "${dir}"

  if [[ "${source}" =~ ^https?:// ]]; then
    curl -fL --progress-bar -o "${destination}" "${source}"
  else
    cp "${source}" "${destination}"
  fi
}

download_artifact() {
  local artifact_name="$1"
  local dest_path="$2"

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    local local_path="${ARTIFACT_DIR}/${artifact_name}"
    if [[ ! -f "${local_path}" ]]; then
      fatal "Expected artifact '${artifact_name}' not found in ${ARTIFACT_DIR}."
    fi
    log "Using local artifact: ${local_path}"
    download_file "${local_path}" "${dest_path}"
    return
  fi

  local url="${BASE_URL}/${VERSION_TAG}/${artifact_name}"
  log "Downloading ${artifact_name} from ${url}"
  download_file "${url}" "${dest_path}"
}

download_checksums() {
  local dest_path="$1"

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    local checksum_path="${ARTIFACT_DIR}/SHA256SUMS"
    if [[ -f "${checksum_path}" ]]; then
      download_file "${checksum_path}" "${dest_path}"
      return 0
    fi
    warn "SHA256SUMS not found in ${ARTIFACT_DIR}; skipping checksum verification."
    return 1
  fi

  local url="${BASE_URL}/${VERSION_TAG}/SHA256SUMS"
  log "Downloading SHA256SUMS from ${url}"
  if ! download_file "${url}" "${dest_path}"; then
    warn "Failed to download SHA256SUMS; skipping checksum verification."
    return 1
  fi
  return 0
}

verify_checksum() {
  local archive_path="$1"
  local checksums_path="$2"
  local artifact_name="$3"

  if [[ ! -f "${checksums_path}" ]]; then
    warn "Checksum file missing; skipping verification."
    return
  fi

  local expected
  expected="$(grep "  ${artifact_name}$" "${checksums_path}" | awk '{print $1}' || true)"
  if [[ -z "${expected}" ]]; then
    warn "Checksum entry for ${artifact_name} not found; skipping verification."
    return
  fi

  local actual
  actual="$(compute_file_sha256 "${archive_path}")"

  if [[ "${expected}" != "${actual}" ]]; then
    fatal "Checksum mismatch for ${artifact_name}: expected ${expected}, got ${actual}"
  fi
  log "Checksum verified for ${artifact_name}"
}

target_triple_linux() {
  case "${ARCH}" in
    x86_64|amd64)
      printf 'x86_64-unknown-linux-gnu'
      ;;
    aarch64|arm64)
      printf 'aarch64-unknown-linux-gnu'
      ;;
    *)
      fatal "Unsupported Linux architecture: ${ARCH}"
      ;;
  esac
}

target_triple_macos() {
  case "${ARCH}" in
    arm64)
      printf 'aarch64-apple-darwin'
      ;;
    x86_64|amd64)
      fatal "macOS Intel installs are not supported; use an Apple Silicon host."
      ;;
    *)
      fatal "Unsupported macOS architecture: ${ARCH}"
      ;;
  esac
}

bundle_label_for_target() {
  local target="$1"
  case "${target}" in
    x86_64-unknown-linux-gnu)
      printf 'linux_x86_64'
      ;;
    aarch64-unknown-linux-gnu)
      printf 'linux_aarch64'
      ;;
    x86_64-apple-darwin)
      printf 'macos_x86_64'
      ;;
    aarch64-apple-darwin)
      printf 'macos_arm64'
      ;;
    *)
      fatal "Unsupported release target: ${target}"
      ;;
  esac
}

bundle_archive_name() {
  local label="$1"
  printf 'substrate-v%s-%s.tar.gz' "${VERSION}" "${label}"
}

fetch_bundle_archive() {
  local archive_name="$1"
  local dest_path="$2"

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    if [[ -d "${ARTIFACT_DIR}" && -f "${ARTIFACT_DIR}/${archive_name}" ]]; then
      cp "${ARTIFACT_DIR}/${archive_name}" "${dest_path}"
      return
    fi
    if [[ -f "${ARTIFACT_DIR}" && "$(basename "${ARTIFACT_DIR}")" == "${archive_name}" ]]; then
      cp "${ARTIFACT_DIR}" "${dest_path}"
      return
    fi
    fatal "Expected bundle '${archive_name}' not found in ${ARTIFACT_DIR}."
  fi

  download_artifact "${archive_name}" "${dest_path}"
}

prepare_bundle_payload() {
  local target_triple="$1"
  local release_root="$2"
  local checksums_path="$3"

  local label
  label="$(bundle_label_for_target "${target_triple}")"
  local archive_name
  archive_name="$(bundle_archive_name "${label}")"
  local archive_path="${TMPDIR}/${archive_name}"

  fetch_bundle_archive "${archive_name}" "${archive_path}"
  if [[ -n "${checksums_path}" ]]; then
    verify_checksum "${archive_path}" "${checksums_path}" "${archive_name}"
  fi

  local extract_dir="${TMPDIR}/bundle-${label}"
  rm -rf "${extract_dir}"
  extract_archive "${archive_path}" "${extract_dir}"
  local bundle_root
  bundle_root="$(find_extracted_root "${extract_dir}")"

  rm -rf "${release_root}"
  mkdir -p "${release_root}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] stage bundle contents from %s into %s\n' "${INSTALLER_NAME}" "${bundle_root}" "${release_root}" >&2
    return
  fi
  cp -R "${bundle_root}/." "${release_root}/"
}

extract_archive() {
  local archive_path="$1"
  local dest_dir="$2"

  mkdir -p "${dest_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] extract %s -> %s\n' "${INSTALLER_NAME}" "${archive_path}" "${dest_dir}" >&2
    return 0
  fi

  case "${archive_path}" in
    *.tar.gz|*.tgz)
      tar -xzf "${archive_path}" -C "${dest_dir}"
      ;;
    *.tar.xz|*.txz)
      tar -xJf "${archive_path}" -C "${dest_dir}"
      ;;
    *.zip)
      unzip -q "${archive_path}" -d "${dest_dir}"
      ;;
    *)
      fatal "Unsupported archive format: ${archive_path}"
      ;;
  esac
}

find_extracted_root() {
  local dest_dir="$1"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '%s\n' "${dest_dir}/SIMULATED_ROOT"
    return
  fi
  local entries=()
  while IFS= read -r entry; do
    entries+=("${entry}")
  done < <(find "${dest_dir}" -mindepth 1 -maxdepth 1 -print)
  if [[ ${#entries[@]} -eq 0 ]]; then
    fatal "Failed to determine extracted archive root."
  fi
  if [[ ${#entries[@]} -eq 1 && -d "${entries[0]}" ]]; then
    printf '%s\n' "${entries[0]}"
  else
    printf '%s\n' "${dest_dir}"
  fi
}

link_binaries() {
  local version_dir="$1"
  local bin_dir="$2"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Linking binaries from %s into %s\n' "${INSTALLER_NAME}" "${version_dir}/bin" "${bin_dir}" >&2
    return
  fi

  mkdir -p "${bin_dir}"
  find "${bin_dir}" -maxdepth 1 -type l -exec rm -f {} +
  if [[ -d "${version_dir}/bin" ]]; then
    for binary in "${version_dir}/bin/"*; do
      local name
      name="$(basename "${binary}")"
      ln -sfn "${binary}" "${bin_dir}/${name}"
    done
  else
    warn "No bin directory found in ${version_dir}"
  fi
}

deploy_shims() {
  local substrate_bin="$1"
  if [[ "${NO_SHIMS}" -eq 1 ]]; then
    log "Skipping shim deployment (--no-shims)."
    return
  fi

  log "Deploying shims..."
  local restore_xtrace=0
  local deploy_status=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  if run_cmd_with_redacted_install_bootstrap_carrier "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    --shim-deploy; then
    deploy_status=0
  else
    deploy_status=$?
  fi
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  return "${deploy_status}"
}

harden_shim_symlinks() {
  local shims_dir="$1"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Normalize shims in %s to real binaries\n' "${INSTALLER_NAME}" "${shims_dir}" >&2
    return
  fi

  if [[ ! -d "${shims_dir}" ]]; then
    return
  fi

  local converted=0
  while IFS= read -r -d '' shim_path; do
    local link_target
    link_target="$(readlink "${shim_path}")" || continue

    local resolved_target
    if [[ "${link_target}" == /* ]]; then
      resolved_target="${link_target}"
    else
      local shim_dirname
      shim_dirname="$(cd "$(dirname "${shim_path}")" && pwd -P)"
      resolved_target="${shim_dirname}/${link_target}"
    fi

    if [[ ! -e "${resolved_target}" ]]; then
      continue
    fi

    rm -f "${shim_path}"
    if ! ln "${resolved_target}" "${shim_path}" 2>/dev/null; then
      cp "${resolved_target}" "${shim_path}"
      chmod +x "${shim_path}" 2>/dev/null || true
    fi
    converted=1
  done < <(find "${shims_dir}" -maxdepth 1 -type l -print0 2>/dev/null)

  if [[ ${converted} -eq 1 ]]; then
    log "Normalized shim binaries in ${shims_dir}"
  fi
}

provision_macos_world() {
  local release_root="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  log "Provisioning macOS Lima world backend..."

  local lima_script="${release_root}/scripts/mac/lima-warm.sh"
  if [[ "${DRY_RUN}" -eq 0 && ! -x "${lima_script}" ]]; then
    fatal "Expected Lima warm script not found at ${lima_script}"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] (cd %s && %s %s)\n' "${INSTALLER_NAME}" "${release_root}" "${lima_script}" "${release_root}" >&2
    return
  fi

  (
    cd "${release_root}" &&
    if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
      SUBSTRATE_WORLD_NETFILTER_ENABLE=1 "${lima_script}" "${release_root}"
    else
      "${lima_script}" "${release_root}"
    fi
  )

  if ! limactl shell substrate test -x /usr/local/bin/substrate-world-service >/dev/null 2>&1; then
    fatal "Lima provisioning completed but /usr/local/bin/substrate-world-service is missing. Provide bin/linux/world-service in the release bundle or rerun from a source checkout so the installer can build one."
  fi
  if ! limactl shell substrate test -x /usr/local/bin/substrate-gateway >/dev/null 2>&1; then
    fatal "Lima provisioning completed but /usr/local/bin/substrate-gateway is missing. Provide bin/linux/substrate-gateway in the release bundle or rerun from a source checkout so the installer can build one."
  fi
  log "Verified Linux world-service + substrate-gateway installation inside Lima (copy/build path logged above)."
}

systemd_escape_unit_value() {
  python3 - "$1" <<'PY'
import sys

raw = sys.argv[1].encode("utf-8")
safe = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789/._:-"
out = []
for byte in raw:
    if byte == 0x25:
        out.append("%%")
    elif byte in safe:
        out.append(chr(byte))
    else:
        out.append(f"\\x{byte:02x}")
print("".join(out))
PY
}

provision_linux_world() {
  local version_dir="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  local world_service=""
  local gateway_binary=""
  local acl_helper="${version_dir}/scripts/linux/substrate-apply-socket-acl.sh"
  if [[ -x "${version_dir}/bin/world-service" ]]; then
    world_service="${version_dir}/bin/world-service"
  elif [[ -x "${version_dir}/bin/linux/world-service" ]]; then
    world_service="${version_dir}/bin/linux/world-service"
  fi

  if [[ -x "${version_dir}/bin/substrate-gateway" ]]; then
    gateway_binary="${version_dir}/bin/substrate-gateway"
  elif [[ -x "${version_dir}/bin/linux/substrate-gateway" ]]; then
    gateway_binary="${version_dir}/bin/linux/substrate-gateway"
  fi

  if [[ -z "${world_service}" ]]; then
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      world_service="${version_dir}/bin/world-service"
      warn "Linux world-service binary not found in release bundle; using placeholder path for dry run."
    else
      fatal "Linux world-service binary not found in release bundle under ${version_dir}/bin."
    fi
  fi

  if [[ -z "${gateway_binary}" ]]; then
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      gateway_binary="${version_dir}/bin/substrate-gateway"
      warn "Linux substrate-gateway binary not found in release bundle; using placeholder path for dry run."
    else
      fatal "Linux substrate-gateway binary not found in release bundle under ${version_dir}/bin."
    fi
  fi

  if [[ "${DRY_RUN}" -eq 0 && ! -f "${acl_helper}" ]]; then
    fatal "Linux ACL helper missing from release bundle at ${acl_helper}."
  fi

  log "Installing Linux world agent systemd service and substrate-gateway binary..."

  local service_path="/etc/systemd/system/substrate-world-service.service"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] sudo install -Dm0755 %s /usr/local/bin/substrate-world-service\n' "${INSTALLER_NAME}" "${world_service}" >&2
    printf '[%s][dry-run] sudo install -Dm0755 %s /usr/local/bin/substrate-gateway\n' "${INSTALLER_NAME}" "${gateway_binary}" >&2
    printf '[%s][dry-run] sudo install -Dm0755 %s /usr/libexec/substrate/substrate-apply-socket-acl\n' "${INSTALLER_NAME}" "${acl_helper}" >&2
    printf '[%s][dry-run] sudo install -d -m0750 -o root -g substrate /run/substrate && sudo install -d -m0750 /var/lib/substrate\n' "${INSTALLER_NAME}" >&2
    printf '[%s][dry-run] Write systemd unit to %s\n' "${INSTALLER_NAME}" "${service_path}" >&2
    printf '[%s][dry-run] sudo systemctl daemon-reload && sudo systemctl enable --now substrate-world-service\n' "${INSTALLER_NAME}" >&2
    return
  fi

  run_with_sudo install -Dm0755 "${world_service}" /usr/local/bin/substrate-world-service
  run_with_sudo install -Dm0755 "${gateway_binary}" /usr/local/bin/substrate-gateway
  run_with_sudo install -Dm0755 "${acl_helper}" /usr/libexec/substrate/substrate-apply-socket-acl
  run_with_sudo install -d -m0750 -o root -g substrate /run/substrate
  run_with_sudo install -d -m0750 /var/lib/substrate
  run_with_sudo install -d -m0750 -o root -g substrate /var/lib/substrate/world-deps
  run_with_sudo install -d -m0750 -o root -g substrate /var/lib/substrate/world-deps/bin

  local unit_file
  unit_file="${TMPDIR}/substrate-world-service.service"
  local netfilter_env_line=""
  if [[ "${ENABLE_WORLD_NETFILTER}" -eq 1 ]]; then
    netfilter_env_line="Environment=WORLD_NETFILTER_ENABLE=1"
  fi
  local systemd_home systemd_commitment systemd_account systemd_uid systemd_carrier
  systemd_home="$(systemd_escape_unit_value "${PREFIX}")"
  systemd_commitment="$(systemd_escape_unit_value "${INSTALL_BOOTSTRAP_COMMITMENT}")"
  systemd_account="$(systemd_escape_unit_value "${INSTALL_BOOTSTRAP_ACCOUNT}")"
  systemd_uid="$(systemd_escape_unit_value "${INSTALL_BOOTSTRAP_UID}")"
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  systemd_carrier="$(systemd_escape_unit_value "${INSTALL_BOOTSTRAP_CONTEXT_V1}")"
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  cat > "${unit_file}" <<UNIT
[Unit]
Description=Substrate World Service
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-service
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=SUBSTRATE_AGENT_TCP_PORT=61337
Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock
Environment="SUBSTRATE_HOME=${systemd_home}"
Environment="SUBSTRATE_ROOT=${systemd_home}"
Environment="SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${systemd_commitment}"
Environment="SUBSTRATE_INSTALL_PRIMARY_USER=${systemd_account}"
Environment="SUBSTRATE_INSTALL_PRIMARY_UID=${systemd_uid}"
Environment="SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${systemd_carrier}"
${netfilter_env_line}
Group=substrate
UMask=0027
RuntimeDirectory=substrate
RuntimeDirectoryMode=0750
StateDirectory=substrate
StateDirectoryMode=0750
WorkingDirectory=/var/lib/substrate
StandardOutput=journal
StandardError=journal
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths="${systemd_home}" /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE CAP_CHOWN CAP_SYS_PTRACE

[Install]
WantedBy=multi-user.target
UNIT

  local socket_unit
  socket_unit="${TMPDIR}/substrate-world-service.socket"
  cat > "${socket_unit}" <<UNIT
[Unit]
Description=Substrate World Service Socket
PartOf=substrate-world-service.service

[Socket]
ListenStream=/run/substrate.sock
SocketMode=0660
SocketUser=root
SocketGroup=substrate
DirectoryMode=0750
RemoveOnStop=yes
Service=substrate-world-service.service

[Install]
WantedBy=sockets.target
UNIT

  local socket_dropin
  socket_dropin="${TMPDIR}/20-substrate-group-acl.conf"
  cat > "${socket_dropin}" <<'UNIT'
[Socket]
ExecStartPost=-/usr/libexec/substrate/substrate-apply-socket-acl --socket /run/substrate.sock substrate
UNIT

  run_with_sudo install -Dm0644 "${unit_file}" "${service_path}"
  run_with_sudo install -Dm0644 "${socket_unit}" /etc/systemd/system/substrate-world-service.socket
  run_with_sudo install -Dm0644 "${socket_dropin}" /etc/systemd/system/substrate-world-service.socket.d/20-substrate-group-acl.conf
  local legacy_world_unit_prefix="substrate-world"
  local legacy_service="${legacy_world_unit_prefix}-agent.service"
  local legacy_socket="${legacy_world_unit_prefix}-agent.socket"
  run_with_sudo systemctl stop "${legacy_service}" "${legacy_socket}" || true
  run_with_sudo systemctl disable "${legacy_service}" "${legacy_socket}" || true
  run_with_sudo rm -f "/etc/systemd/system/${legacy_service}" "/etc/systemd/system/${legacy_socket}"
  run_with_sudo systemctl daemon-reload
  run_with_sudo systemctl enable substrate-world-service.service
  run_with_sudo systemctl enable --now substrate-world-service.socket
  run_with_sudo systemctl stop substrate-world-service.service substrate-world-service.socket || true
  run_with_sudo install -d -m0750 -o root -g substrate /run/substrate
  run_with_sudo install -d -m0750 -o root -g substrate /var/lib/substrate/world-deps
  run_with_sudo install -d -m0750 -o root -g substrate /var/lib/substrate/world-deps/bin
  run_with_sudo rm -f /run/substrate.sock
  run_with_sudo systemctl start substrate-world-service.socket
  run_with_sudo /usr/libexec/substrate/substrate-apply-socket-acl --socket /run/substrate.sock substrate || true
  run_with_sudo /usr/libexec/substrate/substrate-apply-socket-acl --directory-traverse /var/lib/substrate substrate || true
  run_with_sudo /usr/libexec/substrate/substrate-apply-socket-acl --tree-readonly /var/lib/substrate/world-deps substrate || true
  run_with_sudo systemctl start substrate-world-service.service
  run_with_sudo systemctl status substrate-world-service.socket --no-pager --lines=10 || true
  run_with_sudo systemctl status substrate-world-service.service --no-pager --lines=10 || true
}

run_world_checks() {
  local substrate_bin="$1"
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world doctor (--no-world)."
    return
  fi
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s --install-bootstrap-context-v1 <carrier> world doctor --json\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    return
  fi

  log "Running substrate world doctor..."
  local check_rc=0
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  if ! "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world doctor --json | jq '.'; then
    check_rc=1
  fi
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${check_rc}" -ne 0 ]]; then
    warn "World doctor reported issues. Review output above."
  fi
}

print_world_deps_summary() {
  local substrate_bin="$1"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s --install-bootstrap-context-v1 <carrier> world deps current list applied --json\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    return
  fi

  log "World dependency status (in world):"
  local check_rc=0
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  if ! "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps current list applied --json | jq -r '
    if (.items | length) == 0 then
      "  (no enabled deps items)"
    else
      .items[]
      | "- \(.name): kind=\(.kind) enabled=\(.enabled // false) world=\(.world // "unknown")\(if .remediation then " remediation=\(.remediation)" else "" end)"
    end
  '; then
    check_rc=1
  fi
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${check_rc}" -ne 0 ]]; then
    warn "world deps check failed; run 'substrate world deps current list applied --json' for details."
  fi
}

rollback_agent_runtime_world_deps_enable() {
  local substrate_bin="$1"
  local deps_item="$2"

  local rollback_rc=0
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  if "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps global remove "${deps_item}"; then
    rollback_rc=0
  else
    rollback_rc=$?
  fi
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${rollback_rc}" -eq 0 ]]; then
    warn "Rolled back world deps global enable for '${deps_item}' after sync failure."
    return 0
  fi

  warn "Unable to roll back world deps global enable for '${deps_item}' after sync failure."
  return 1
}

sync_world_deps() {
  local substrate_bin="$1"
  if [[ "${SYNC_DEPS}" -ne 1 && -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return
  fi
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world dependency sync because --no-world was used."
    return
  fi
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s --install-bootstrap-context-v1 <carrier> world deps current sync\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    print_world_deps_summary "${substrate_bin}"
    return
  fi

  if [[ -n "${PROVISION_AGENT_RUNTIME}" ]]; then
    log "Syncing world dependencies via 'substrate world deps current sync' for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}..."
  else
    log "Syncing world dependencies via 'substrate world deps current sync'..."
  fi
  local rc=0
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  if "${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps current sync; then
    rc=0
  else
    rc=$?
  fi
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${rc}" -ne 0 ]]; then
    if [[ -n "${PROVISION_AGENT_RUNTIME}" ]]; then
      local deps_item
      deps_item="$(world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}")"
      if [[ "${PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER}" -ne 1 ]]; then
        print_world_deps_summary "${substrate_bin}"
        if [[ "${rc}" -eq 4 ]]; then
          fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; '${deps_item}' was already globally enabled before this install, so the installer left that enable in place. Run 'substrate world enable --provision-deps', then rerun 'substrate world deps current sync'."
        fi
        fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; '${deps_item}' was already globally enabled before this install, so the installer left that enable in place. Fix the sync failure and rerun 'substrate world deps current sync'."
      fi
      local rollback_succeeded=0
      if rollback_agent_runtime_world_deps_enable "${substrate_bin}" "${deps_item}"; then
        rollback_succeeded=1
      fi
      print_world_deps_summary "${substrate_bin}"
      if [[ "${rollback_succeeded}" -eq 1 ]]; then
        if [[ "${rc}" -eq 4 ]]; then
          fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; the installer removed the global enable because provisioning-time system packages are still required. Clearing the global enable does not roll back any guest-side state that may already have been applied in the world. Run 'substrate world enable --provision-deps', then $(agent_runtime_retry_after_sync_failure)."
        fi
        fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; the installer removed the global enable so the runtime is not left persistently enabled for future syncs. $(world_deps_global_remove_scope_note) Then $(agent_runtime_retry_after_sync_failure)."
      fi
      if [[ "${rc}" -eq 4 ]]; then
        fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; provisioning-time system packages are still required, and rollback also failed so '${deps_item}' remains globally enabled. Run 'substrate world deps global remove ${deps_item}' to clear the global enable only. $(world_deps_global_remove_scope_note) Then run 'substrate world enable --provision-deps', then $(agent_runtime_retry_after_sync_failure)."
      fi
      fatal_with_code "${rc}" "world deps sync failed for --provision-agent-runtime ${PROVISION_AGENT_RUNTIME}; rollback also failed so '${deps_item}' remains globally enabled. Run 'substrate world deps global remove ${deps_item}' after fixing the sync failure to clear the global enable only. $(world_deps_global_remove_scope_note) Then $(agent_runtime_retry_after_sync_failure)."
    fi
    if [[ "${rc}" -eq 4 ]]; then
      warn "world deps sync requires provisioning-time system packages; run 'substrate world enable --provision-deps'."
    fi
    warn "world deps sync failed; run 'substrate world deps current sync' later to finish provisioning."
  fi
  print_world_deps_summary "${substrate_bin}"
}

provision_agent_runtime_world_deps() {
  local substrate_bin="$1"
  if [[ -z "${PROVISION_AGENT_RUNTIME}" ]]; then
    return
  fi

  local deps_item
  deps_item="$(world_deps_item_for_agent_runtime "${PROVISION_AGENT_RUNTIME}")"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s --install-bootstrap-context-v1 <carrier> world deps global add %s\n' "${INSTALLER_NAME}" "${substrate_bin}" "${deps_item}" >&2
    return
  fi

  log "Enabling agent runtime '${PROVISION_AGENT_RUNTIME}' globally via world deps item '${deps_item}'. The installer will run 'substrate world deps current sync' immediately after this step."
  local add_output
  local restore_xtrace=0
  if [[ $- == *x* ]]; then
    set +x
    restore_xtrace=1
  fi
  add_output="$("${substrate_bin}" \
    --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    world deps global add --json "${deps_item}")"
  if [[ "${restore_xtrace}" -eq 1 ]]; then
    set -x
  fi
  if grep -Fq "\"${deps_item}\"" <<<"${add_output}"; then
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=1
  else
    PROVISION_AGENT_RUNTIME_ADDED_BY_INSTALLER=0
  fi
  printf '%s\n' "${add_output}"
}

PATH_SNIPPET_START="# >>> substrate >>>"
PATH_SNIPPET_END="# <<< substrate <<<"

render_path_snippet_sh() {
  local bin_dir="$1"

  cat <<EOF
${PATH_SNIPPET_START}
if [ -d "${bin_dir}" ]; then
  case ":\${PATH}:" in
    *":${bin_dir}:"*) ;;
    *) export PATH="${bin_dir}:\${PATH}" ;;
  esac
fi
${PATH_SNIPPET_END}
EOF
}

render_path_snippet_fish() {
  local bin_dir="$1"

  cat <<EOF
${PATH_SNIPPET_START}
if test -d "${bin_dir}"
  if type -q fish_add_path
    fish_add_path -m "${bin_dir}"
  else
    set -gx PATH "${bin_dir}" \$PATH
  end
end
${PATH_SNIPPET_END}
EOF
}

upsert_path_snippet() {
  local target="$1"
  local snippet="$2"

  local dir
  dir="$(dirname "${target}")"
  mkdir -p "${dir}"

  local tmp
  tmp="$(mktemp)"

  if [[ -f "${target}" ]] && grep -Fq "${PATH_SNIPPET_START}" "${target}" && grep -Fq "${PATH_SNIPPET_END}" "${target}"; then
    # Remove existing block (if present) so we can append a fresh one.
    sed "\#^${PATH_SNIPPET_START}\$#,\#^${PATH_SNIPPET_END}\$#d" "${target}" > "${tmp}"
  else
    [[ -f "${target}" ]] && cat "${target}" > "${tmp}"
  fi

  {
    if [[ -s "${tmp}" ]]; then
      printf '\n'
    fi
    printf '%s\n' "${snippet}"
  } >> "${tmp}"

  mv "${tmp}" "${target}"
}

update_shell_path() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return 0
  fi

  if [[ "${SUBSTRATE_INSTALL_NO_PATH:-}" == "1" ]]; then
    log "Skipping PATH update because SUBSTRATE_INSTALL_NO_PATH=1."
    return 0
  fi

  local bin_dir="$1"
  local account_home="$2"
  local shell_basename
  shell_basename="$(basename "${SHELL:-}")"

  case "${shell_basename}" in
    zsh)
      local snippet
      snippet="$(render_path_snippet_sh "${bin_dir}")"
      upsert_path_snippet "${account_home}/.zprofile" "${snippet}"
      upsert_path_snippet "${account_home}/.zshrc" "${snippet}"
      ;;
    bash)
      local snippet
      snippet="$(render_path_snippet_sh "${bin_dir}")"
      upsert_path_snippet "${account_home}/.bashrc" "${snippet}"
      upsert_path_snippet "${account_home}/.bash_profile" "${snippet}"
      ;;
    fish)
      local snippet
      snippet="$(render_path_snippet_fish "${bin_dir}")"
      upsert_path_snippet "${account_home}/.config/fish/config.fish" "${snippet}"
      ;;
    *)
      local snippet
      snippet="$(render_path_snippet_sh "${bin_dir}")"
      upsert_path_snippet "${account_home}/.profile" "${snippet}"
      ;;
  esac
}

install_macos() {
  ensure_macos_prereqs
  ensure_version_selected

  local target_triple
  target_triple="$(target_triple_macos)"

  local release_root="${TMPDIR}/payload"
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if ! download_checksums "${checksums_path}"; then
    checksums_path=""
  fi

  prepare_bundle_payload "${target_triple}" "${release_root}" "${checksums_path}"

  local primary_user
  primary_user="$(detect_primary_user)"
  bootstrap_private_substrate_home "${release_root}/bin/substrate" "${primary_user}"

  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  log "Installing to ${version_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] mkdir -p %s\n' "${INSTALLER_NAME}" "${versions_dir}" >&2
    printf '[%s][dry-run] rm -rf %s\n' "${INSTALLER_NAME}" "${version_dir}" >&2
    printf '[%s][dry-run] copy contents of %s into %s\n' "${INSTALLER_NAME}" "${release_root}" "${version_dir}" >&2
  else
    mkdir -p "${versions_dir}"
    rm -rf "${version_dir}"
    mkdir -p "${version_dir}"
    cp -R "${release_root}"/. "${version_dir}"/
  fi

  ensure_version_config_present "${version_dir}"

  link_binaries "${version_dir}" "${bin_dir}"

  local world_enabled=1
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    world_enabled=0
  fi

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_macos_world "${version_dir}"
  local doctor_original_path
  doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" run_world_checks "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" provision_agent_runtime_world_deps "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" sync_world_deps "${substrate_bin}"

  finalize_install_metadata "${world_enabled}"
  update_shell_path "${bin_dir}" "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "Installation complete (dry run). After a real install add ${bin_dir} to your PATH or run ${bin_dir}/substrate directly."
  else
    log "Installation complete. ${bin_dir} was added to your shell PATH."
  fi
  log "manager_init placeholder: ${MANAGER_INIT_PATH}"
  log "manager_env script: ${MANAGER_ENV_PATH}"
  log "config manifests: ${version_dir}/config"
  if [[ -f "${INSTALL_CONFIG_PATH}" ]]; then
    log "install metadata: ${INSTALL_CONFIG_PATH}"
  else
    warn "install metadata missing at ${INSTALL_CONFIG_PATH}; run 'substrate config init' after installing to create defaults."
  fi
  log "If the global config is missing or needs regeneration, run 'substrate config init' after installing."

  if [[ "${world_enabled}" -eq 1 ]]; then
    log "World backend enabled; run '${bin_dir}/substrate world doctor --json' or '${bin_dir}/substrate world deps current sync' as needed."
  else
    log "World backend disabled (--no-world). Run '${bin_dir}/substrate world enable --home \"${PREFIX}\"' when you are ready to provision the backend."
    log "Runtime-family provisioning after install is helper-only in this slice: use '${version_dir}/scripts/substrate/world-enable.sh --home \"${PREFIX}\" --provision-agent-runtime codex' instead of '${bin_dir}/substrate world enable --provision-agent-runtime ...'."
  fi

  write_host_state_metadata "${world_enabled}"
}

install_linux() {
  ensure_linux_prereqs
  ensure_version_selected

  local target_triple
  target_triple="$(target_triple_linux)"

  local release_root="${TMPDIR}/payload"
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if ! download_checksums "${checksums_path}"; then
    checksums_path=""
  fi

  prepare_bundle_payload "${target_triple}" "${release_root}" "${checksums_path}"

  local primary_user
  primary_user="$(detect_primary_user)"
  bootstrap_private_substrate_home "${release_root}/bin/substrate" "${primary_user}"

  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  log "Installing to ${version_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] mkdir -p %s\n' "${INSTALLER_NAME}" "${versions_dir}" >&2
    printf '[%s][dry-run] rm -rf %s\n' "${INSTALLER_NAME}" "${version_dir}" >&2
    printf '[%s][dry-run] copy contents of %s into %s\n' "${INSTALLER_NAME}" "${release_root}" "${version_dir}" >&2
  else
    mkdir -p "${versions_dir}"
    rm -rf "${version_dir}"
    mkdir -p "${version_dir}"
    cp -R "${release_root}"/. "${version_dir}"/
  fi

  ensure_version_config_present "${version_dir}"

  link_binaries "${version_dir}" "${bin_dir}"

  local world_enabled=1
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    world_enabled=0
  fi

  if [[ "${world_enabled}" -eq 1 ]]; then
    ensure_linux_group_membership "${primary_user}"
  fi

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_linux_world "${version_dir}"
  local doctor_original_path
  doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" run_world_checks "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" provision_agent_runtime_world_deps "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_HOME="${PREFIX}" sync_world_deps "${substrate_bin}"

  finalize_install_metadata "${world_enabled}"
  update_shell_path "${bin_dir}" "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}"

  if [[ "${IS_WSL}" -eq 1 ]]; then
    log "Detected WSL environment. Windows host components (forwarder, uninstall) must be managed via PowerShell scripts."
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "Installation complete (dry run). After a real install add ${bin_dir} to your PATH or run ${bin_dir}/substrate directly."
  else
    log "Installation complete. ${bin_dir} was added to your shell PATH."
  fi
  log "manager_init placeholder: ${MANAGER_INIT_PATH}"
  log "manager_env script: ${MANAGER_ENV_PATH}"
  log "config manifests: ${version_dir}/config"
  if [[ -f "${INSTALL_CONFIG_PATH}" ]]; then
    log "install metadata: ${INSTALL_CONFIG_PATH}"
  else
    warn "install metadata missing at ${INSTALL_CONFIG_PATH}; run 'substrate config init' after installing to create defaults."
  fi
  log "If the global config is missing or needs regeneration, run 'substrate config init' after installing."

  if [[ "${world_enabled}" -eq 1 ]]; then
    log "World backend enabled; run '${bin_dir}/substrate world doctor --json' for diagnostics or '${bin_dir}/substrate world deps current sync' to provision world deps."
    print_linger_guidance_linux "${primary_user}"
  else
    log "World backend disabled (--no-world). Run '${bin_dir}/substrate world enable --home \"${PREFIX}\"' when you are ready to provision the backend."
    log "Runtime-family provisioning after install is helper-only in this slice: use '${version_dir}/scripts/substrate/world-enable.sh --home \"${PREFIX}\" --provision-agent-runtime codex' instead of '${bin_dir}/substrate world enable --provision-agent-runtime ...'."
  fi

  write_host_state_metadata "${world_enabled}"
}

main() {
  sanitize_env_path
  parse_args "$@"
  if [[ "${HELP_REQUESTED}" -eq 1 && "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 0 ]]; then
    if [[ "${RELEASE_INSTALL_INHERITED_XTRACE}" -eq 1 ]]; then
      set -x
    fi
    print_usage
    return 0
  fi
  resolve_install_bootstrap_context \
    "${PREFIX_DECLARED}" \
    "${PREFIX}" \
    "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
    "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}"
  if [[ "${RELEASE_INSTALL_INHERITED_XTRACE}" -eq 1 ]]; then
    set -x
  fi
  if [[ "${HELP_REQUESTED}" -eq 1 ]]; then
    print_usage
    return 0
  fi
  validate_agent_runtime_provision_request
  normalize_prefix
  initialize_metadata_paths
  detect_platform
  ensure_supported_linux_world_posture
  prepare_tmpdir

  case "${PLATFORM}" in
    macos)
      install_macos
      ;;
    linux)
      install_linux
      ;;
    windows)
      warn "Automated Windows (PowerShell) installation flow not yet implemented. Refer to docs/install/windows.md."
      exit 2
      ;;
    *)
      fatal "Unsupported platform: ${PLATFORM}"
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
