#!/usr/bin/env bash
set -euo pipefail

SOURCE_PATH="${BASH_SOURCE[0]}"
while [[ -L "${SOURCE_PATH}" ]]; do
    SOURCE_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
    SOURCE_PATH="$(readlink "${SOURCE_PATH}")"
    [[ "${SOURCE_PATH}" != /* ]] && SOURCE_PATH="${SOURCE_DIR}/${SOURCE_PATH}"
done
SCRIPT_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
CANONICAL_UNIT_SOURCE_DIR=""
# Check-only status preserves its frozen named-instance observation surface. Mutation is separately
# constrained in ensure_vm_ready to the one selected R2 Stage-1 identity.
VM_NAME="${SUBSTRATE_LIMA_VM_NAME:-substrate}"
PROFILE="${LIMA_PROFILE_PATH:-${SCRIPT_DIR}/lima/substrate.yaml}"
PROJECT_PATH=""
CHECK_ONLY=0
BUILD_PROFILE="${LIMA_BUILD_PROFILE:-release}"
LAYOUT_SENTINEL="/etc/substrate-lima-layout"
LAYOUT_VERSION="socket-parity-v2-staged-workspace-v1"
STAGED_WORKSPACE_ROOT="/var/lib/substrate/staged-workspace"
STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_ROOT}/current"
STAGED_WORKSPACE_MANIFEST_NAME=".substrate-lima-stage-manifest"
WAIT_TIMEOUT=120
SKIP_GUEST_BUILD="${SUBSTRATE_LIMA_SKIP_GUEST_BUILD:-0}"
HELP_REQUESTED=0
INSTALL_PREFIX_RAW=""
INSTALL_PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
INSTALL_BOOTSTRAP_CONTEXT_DECLARED=0
PLATFORM_BOOTSTRAP_MAPPING_V1=""
EXECUTOR_BUILD_EVIDENCE_V1=""
PUBLISHER_REQUEST_V1=""
LIMA_STAGE_ONE_AUTHORIZATION_V1=""
INSTALL_BOOTSTRAP_COMMITMENT=""
INSTALL_BOOTSTRAP_ACCOUNT=""
INSTALL_BOOTSTRAP_UID=""
INSTALL_BOOTSTRAP_ACCOUNT_HOME=""
INSTALL_CONTEXT_MODE=""
HOST_ACCOUNT_HOME=""
HOST_PLATFORM_CONTROL_ROOT=""
OBSERVED_GUEST_MACHINE_ID=""
OBSERVED_GUEST_ACCOUNT=""
OBSERVED_GUEST_UID=""
OBSERVED_GUEST_HOME=""
OBSERVED_GUEST_SUBSTRATE_HOME=""
OBSERVED_PLATFORM_MAPPING_V1=""
OBSERVED_TRANSPORT_HOST=""
OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"

log() {
    printf '==> %s\n' "$1"
}

warn() {
    printf 'WARN | %s\n' "$1" >&2
}

fatal() {
    printf 'ERROR | %s\n' "$1" >&2
    exit 1
}

usage() {
    cat <<'USAGE'
Usage: scripts/mac/lima-warm.sh [options] [<project-path>]

This helper is the degraded-but-supported macOS declared-instance Stage-1
create/start plus matching-layout guest-projection wrapper.
Preferred day-to-day operator path after provisioning: `substrate host doctor
[--json]`, `substrate world doctor [--json]`, `substrate world gateway
sync|status|restart`, `substrate world enable`, and `substrate world deps
current sync` for dependency reconciliation.
Breakglass only: raw `limactl shell`, plain SSH, direct guest `systemctl` or
`journalctl`, guest socket `curl`, and host-side `SUBSTRATE_WORLD_SOCKET`
override use.

Options:
  --check-only      Report the current Lima VM status without creating or provisioning it
  --install-prefix  Bind the helper to one host install prefix
  --platform-bootstrap-mapping-v1  Bind the exact selected Lima mapping
  --executor-build-evidence-v1  Require the exact non-executing executor artifact evidence
  --publisher-request-v1  Bind the exact managed role/action request
  --lima-stage-one-authorization-v1  Bind the signed Stage-1 create/start authority
  -h, --help        Show this help text

Arguments:
  <project-path>    Repository or release path to stage for guest provisioning (default: current directory)
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --check-only)
            CHECK_ONLY=1
            shift
            ;;
        --install-prefix)
            [[ $# -ge 2 ]] || fatal "Missing value for --install-prefix"
            [[ "${INSTALL_PREFIX_DECLARED}" -eq 0 ]] || fatal "Duplicate --install-prefix"
            [[ -n "$2" ]] || fatal "Empty value for --install-prefix"
            INSTALL_PREFIX_RAW="$2"
            INSTALL_PREFIX_DECLARED=1
            shift 2
            ;;
        --install-bootstrap-context-v1)
            [[ $# -ge 2 ]] || fatal "Missing value for --install-bootstrap-context-v1"
            [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 0 ]] || fatal "Duplicate --install-bootstrap-context-v1"
            [[ -n "$2" ]] || fatal "Empty value for --install-bootstrap-context-v1"
            INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
            INSTALL_BOOTSTRAP_CONTEXT_DECLARED=1
            shift 2
            ;;
        --platform-bootstrap-mapping-v1)
            [[ $# -ge 2 && -z "${PLATFORM_BOOTSTRAP_MAPPING_V1}" && -n "$2" ]] || fatal "Invalid --platform-bootstrap-mapping-v1"
            PLATFORM_BOOTSTRAP_MAPPING_V1="$2"
            shift 2
            ;;
        --executor-build-evidence-v1)
            [[ $# -ge 2 && -z "${EXECUTOR_BUILD_EVIDENCE_V1}" && -n "$2" ]] || fatal "Invalid --executor-build-evidence-v1"
            EXECUTOR_BUILD_EVIDENCE_V1="$2"
            shift 2
            ;;
        --publisher-request-v1)
            [[ $# -ge 2 && -z "${PUBLISHER_REQUEST_V1}" && -n "$2" ]] || fatal "Invalid --publisher-request-v1"
            PUBLISHER_REQUEST_V1="$2"
            shift 2
            ;;
        --lima-stage-one-authorization-v1)
            [[ $# -ge 2 && -z "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" && -n "$2" ]] || fatal "Invalid --lima-stage-one-authorization-v1"
            LIMA_STAGE_ONE_AUTHORIZATION_V1="$2"
            shift 2
            ;;
        -h|--help)
            HELP_REQUESTED=1
            shift
            ;;
        *)
            if [[ -z "${PROJECT_PATH}" ]]; then
                PROJECT_PATH="$1"
            else
                fatal "Unexpected argument: $1"
            fi
            shift
            ;;
    esac
done

if [[ -z "${PROJECT_PATH}" ]]; then
    PROJECT_PATH="$(pwd)"
fi
PROJECT_PATH="$(cd "${PROJECT_PATH}" && pwd)"

project_unit_source_dir="${PROJECT_PATH}/scripts/mac/lima/units"
script_unit_source_dir="${SCRIPT_DIR}/lima/units"
if [[ -d "${project_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${project_unit_source_dir}"
elif [[ -d "${script_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${script_unit_source_dir}"
else
    fatal "Canonical guest unit directory not found. Expected ${project_unit_source_dir} or ${script_unit_source_dir}."
fi

require_cmd() {
    local name="$1"
    if ! command -v "${name}" >/dev/null 2>&1; then
        fatal "Required command '${name}' not found. Install it and rerun."
    fi
}

require_mapped_lifecycle_authority_v1() {
    [[ -n "${PUBLISHER_REQUEST_V1}" ]] || fatal "R3 lifecycle mutation requires an exact managed publisher request."
    [[ -n "${LIMA_STAGE_ONE_AUTHORIZATION_V1}" ]] || fatal "R3 warm mutation requires LimaStageOneAuthorizationV1."
}

resolve_install_bootstrap_context_v1() {
    local context_file
    local context_err
    local status

    context_file="$(mktemp)"
    context_err="$(mktemp)"
    command -v python3 >/dev/null 2>&1 || fatal "python3 is required for authenticated install bootstrap context resolution."
    if python3 - \
        "${INSTALL_PREFIX_DECLARED}" \
        "${INSTALL_PREFIX_RAW}" \
        "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" >"${context_file}" 2>"${context_err}" <<'PY'
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
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw or "\r" in raw or "\n" in raw:
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
    entry, current_uid = current_principal()
    mode = "internal" if internal else "public"
    if internal:
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
        carrier = b64_encode(
            commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")
        ).decode("ascii")
    account_home = normalize_path(entry.pw_dir)
    for value in (mode, prefix, carrier, commitment, account, str(uid), account_home):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid install bootstrap context", file=sys.stderr)
    raise SystemExit(2)
PY
    then
        :
    else
        status=$?
        if [[ "${status}" -eq 127 ]]; then
            rm -f "${context_file}" "${context_err}"
            fatal "python3 is required for authenticated install bootstrap context resolution."
        fi
        [[ ! -s "${context_err}" ]] || cat "${context_err}" >&2
        rm -f "${context_file}" "${context_err}"
        exit "${status}"
    fi
    rm -f "${context_err}"
    exec 3<"${context_file}"
    IFS= read -r -d '' INSTALL_CONTEXT_MODE <&3 || fatal "Failed to read install bootstrap mode"
    IFS= read -r -d '' INSTALL_PREFIX_RAW <&3 || fatal "Failed to read install bootstrap prefix"
    IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1 <&3 || fatal "Failed to read install bootstrap carrier"
    IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT <&3 || fatal "Failed to read install bootstrap commitment"
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT <&3 || fatal "Failed to read install bootstrap account"
    IFS= read -r -d '' INSTALL_BOOTSTRAP_UID <&3 || fatal "Failed to read install bootstrap uid"
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT_HOME <&3 || fatal "Failed to read install bootstrap account home"
    exec 3<&-
    rm -f "${context_file}"

    export SUBSTRATE_HOME="${INSTALL_PREFIX_RAW}"
    export SUBSTRATE_ROOT="${INSTALL_PREFIX_RAW}"
    export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
    export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
    export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
    export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

resolve_lima_control_root_v1() {
    local control_file

    control_file="$(mktemp)"
    python3 - \
        "${INSTALL_CONTEXT_MODE}" \
        "${INSTALL_BOOTSTRAP_ACCOUNT}" \
        "${INSTALL_BOOTSTRAP_UID}" <<'PY' >"${control_file}"
import os
import pwd
import sys


def fail():
    raise ValueError("invalid lima control root")


def normalize_path(raw):
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw or "\r" in raw or "\n" in raw:
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


try:
    mode = sys.argv[1]
    account = sys.argv[2]
    uid = int(sys.argv[3])
    entry = pwd.getpwuid(uid)
    if entry.pw_name != account:
        fail()
    if pwd.getpwnam(account).pw_uid != uid:
        fail()
    home = normalize_path(entry.pw_dir)
    control_root = normalize_path(home.rstrip("/") + "/.lima")
    for value in (home, control_root):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid lima control root", file=sys.stderr)
    raise SystemExit(2)
PY
    exec 3<"${control_file}"
    IFS= read -r -d '' HOST_ACCOUNT_HOME <&3 || fatal "Failed to read host account home"
    IFS= read -r -d '' HOST_PLATFORM_CONTROL_ROOT <&3 || fatal "Failed to read Lima control root"
    exec 3<&-
    rm -f "${control_file}"

    [[ "${HOST_ACCOUNT_HOME}" == "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}" ]] \
        || fatal "Account-database home changed while resolving the Lima control root."

    if [[ "${INSTALL_CONTEXT_MODE}" == "internal" ]]; then
        if [[ -n "${HOME:-}" && "${HOME}" != "${HOST_ACCOUNT_HOME}" ]]; then
            fatal "Internal Lima child HOME conflicts with the validated account-database home."
        fi
        if [[ -n "${LIMA_HOME:-}" && "${LIMA_HOME}" != "${HOST_PLATFORM_CONTROL_ROOT}" ]]; then
            fatal "Internal Lima child LIMA_HOME conflicts with the validated Lima control root."
        fi
    fi

    export HOME="${HOST_ACCOUNT_HOME}"
    export LIMA_HOME="${HOST_PLATFORM_CONTROL_ROOT}"
}

run_limactl_with_mapping_env_v1() {
    env HOME="${HOST_ACCOUNT_HOME}" LIMA_HOME="${HOST_PLATFORM_CONTROL_ROOT}" limactl "$@"
}

observe_lima_mapping_v1() {
    local machine_id=""
    local machine_id_again=""
    local account=""
    local uid=""
    local name_entry=""
    local uid_entry=""
    local entry_account=""
    local entry_uid=""
    local entry_home=""

    machine_id="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" cat /etc/machine-id 2>/dev/null | tr -d '\r\n' || true)"
    machine_id_again="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" cat /etc/machine-id 2>/dev/null | tr -d '\r\n' || true)"
    [[ -n "${machine_id}" && "${machine_id}" == "${machine_id_again}" ]] \
        || fatal "Unable to observe a stable Lima guest machine identity."

    account="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" id -un 2>/dev/null | tr -d '\r\n' || true)"
    uid="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" id -u 2>/dev/null | tr -d '\r\n' || true)"
    name_entry="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" getent passwd "${account}" 2>/dev/null | tr -d '\r' || true)"
    uid_entry="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" getent passwd "${uid}" 2>/dev/null | tr -d '\r' || true)"
    [[ -n "${account}" && -n "${uid}" && -n "${name_entry}" && "${name_entry}" == "${uid_entry}" ]] \
        || fatal "Unable to round-trip the Lima guest account database identity."

    IFS=':' read -r entry_account _ entry_uid _ _ entry_home _ <<<"${name_entry}"
    [[ "${entry_account}" == "${account}" && "${entry_uid}" == "${uid}" && -n "${entry_home}" ]] \
        || fatal "Lima guest identity does not round-trip through the account database."

    OBSERVED_GUEST_MACHINE_ID="${machine_id}"
    OBSERVED_GUEST_ACCOUNT="${account}"
    OBSERVED_GUEST_UID="${uid}"
    OBSERVED_GUEST_HOME="${entry_home}"
    OBSERVED_GUEST_SUBSTRATE_HOME="${OBSERVED_GUEST_HOME%/}/.substrate"
    OBSERVED_TRANSPORT_HOST="${INSTALL_PREFIX_RAW%/}/sock/agent.sock"
    OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"
}

verify_lima_mapping_v1() {
    local mapping_file

    mapping_file="$(mktemp)"
    python3 - \
        "${INSTALL_BOOTSTRAP_COMMITMENT}" \
        "${VM_NAME}" \
        "${OBSERVED_GUEST_MACHINE_ID}" \
        "${HOST_PLATFORM_CONTROL_ROOT}" \
        "${OBSERVED_GUEST_SUBSTRATE_HOME}" \
        "${OBSERVED_GUEST_ACCOUNT}" \
        "${OBSERVED_GUEST_UID}" \
        "${OBSERVED_TRANSPORT_HOST}" \
        "${OBSERVED_TRANSPORT_GUEST_SOCKET}" <<'PY' >"${mapping_file}"
import base64
import hashlib
import re
import sys

DOMAIN = "substrate.platform_bootstrap_mapping"
KEYS = (
    "domain",
    "version",
    "host_context_commitment",
    "platform_kind",
    "instance_name",
    "guest_machine_id",
    "host_platform_control_root",
    "realized_substrate_home",
    "realized_principal_account",
    "realized_principal_uid",
    "transport_kind",
    "transport_host",
    "transport_guest_socket",
)


def fail():
    raise ValueError("invalid platform bootstrap mapping")


def normalize_path(raw):
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw or "\r" in raw or "\n" in raw:
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


def valid_text(raw):
    return bool(raw) and all(ch not in "\0\r\n" for ch in raw)


def b64_encode(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=").decode("ascii")


def b64_decode(value):
    raw = value.encode("ascii")
    if not raw or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw):
        fail()
    decoded = base64.urlsafe_b64decode(raw + b"=" * ((-len(raw)) % 4))
    if base64.urlsafe_b64encode(decoded).rstrip(b"=") != raw:
        fail()
    return decoded.decode("utf-8")


def record(commitment, vm_name, machine_id, control_root, realized_home, account, uid, host_socket, guest_socket):
    if not re.fullmatch(r"[0-9a-f]{64}", commitment):
        fail()
    if not re.fullmatch(r"[0-9a-f]{32}", machine_id):
        fail()
    if not re.fullmatch(r"0|[1-9][0-9]*", uid):
        fail()
    if not valid_text(vm_name) or not valid_text(account):
        fail()
    lines = [
        f"domain={DOMAIN}",
        "version=1",
        f"host_context_commitment={commitment}",
        "platform_kind=lima",
        f"instance_name={b64_encode(vm_name.encode('utf-8'))}",
        f"guest_machine_id={machine_id}",
        f"host_platform_control_root={b64_encode(normalize_path(control_root).encode('utf-8'))}",
        f"realized_substrate_home={b64_encode(normalize_path(realized_home).encode('utf-8'))}",
        f"realized_principal_account={b64_encode(account.encode('utf-8'))}",
        f"realized_principal_uid={uid}",
        "transport_kind=lima",
        f"transport_host={b64_encode(normalize_path(host_socket).encode('utf-8'))}",
        f"transport_guest_socket={b64_encode(normalize_path(guest_socket).encode('utf-8'))}",
    ]
    return ("\n".join(lines) + "\n").encode("ascii")


def decode(encoded, expected_commitment):
    raw = encoded.encode("ascii")
    if not raw or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw):
        fail()
    record_bytes = base64.urlsafe_b64decode(raw + b"=" * ((-len(raw)) % 4))
    if base64.urlsafe_b64encode(record_bytes).rstrip(b"=") != raw or not record_bytes.endswith(b"\n"):
        fail()
    lines = record_bytes[:-1].split(b"\n")
    if len(lines) != len(KEYS):
        fail()
    values = {}
    for expected, line in zip(KEYS, lines):
        if line.count(b"=") != 1:
            fail()
        key, value = line.split(b"=", 1)
        if key.decode("ascii") != expected:
            fail()
        values[expected] = value.decode("ascii")
    if values["domain"] != DOMAIN or values["version"] != "1":
        fail()
    if values["host_context_commitment"] != expected_commitment:
        fail()
    if values["platform_kind"] != "lima" or values["transport_kind"] != "lima":
        fail()
    rebuilt = record(
        values["host_context_commitment"],
        b64_decode(values["instance_name"]),
        values["guest_machine_id"],
        b64_decode(values["host_platform_control_root"]),
        b64_decode(values["realized_substrate_home"]),
        b64_decode(values["realized_principal_account"]),
        values["realized_principal_uid"],
        b64_decode(values["transport_host"]),
        b64_decode(values["transport_guest_socket"]),
    )
    if rebuilt != record_bytes:
        fail()
    return base64.urlsafe_b64encode(rebuilt).rstrip(b"=").decode("ascii")


try:
    commitment, vm_name, machine_id, control_root, realized_home, account, uid, host_socket, guest_socket = sys.argv[1:]
    raw_record = record(commitment, vm_name, machine_id, control_root, realized_home, account, uid, host_socket, guest_socket)
    encoded = base64.urlsafe_b64encode(raw_record).rstrip(b"=").decode("ascii")
    if decode(encoded, commitment) != encoded:
        fail()
    for value in (
        encoded,
        normalize_path(control_root),
        normalize_path(realized_home),
        normalize_path(host_socket),
        normalize_path(guest_socket),
    ):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid platform bootstrap mapping", file=sys.stderr)
    raise SystemExit(2)
PY
    exec 3<"${mapping_file}"
    IFS= read -r -d '' OBSERVED_PLATFORM_MAPPING_V1 <&3 || fatal "Failed to read platform mapping"
    IFS= read -r -d '' HOST_PLATFORM_CONTROL_ROOT <&3 || fatal "Failed to normalize Lima control root"
    IFS= read -r -d '' OBSERVED_GUEST_SUBSTRATE_HOME <&3 || fatal "Failed to normalize guest substrate home"
    IFS= read -r -d '' OBSERVED_TRANSPORT_HOST <&3 || fatal "Failed to normalize host transport path"
    IFS= read -r -d '' OBSERVED_TRANSPORT_GUEST_SOCKET <&3 || fatal "Failed to normalize guest transport path"
    exec 3<&-
    rm -f "${mapping_file}"
}

check_only_status() {
    local host_os
    host_os="$(uname -s 2>/dev/null || echo "unknown")"
    if [[ "${host_os}" != "Darwin" ]]; then
        echo "[check-only] Host ${host_os} does not support Lima provisioning; skipping warm check."
        exit 0
    fi

    local virtualization=1
    if command -v sysctl >/dev/null 2>&1; then
        virtualization="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
    fi
    if [[ "${virtualization}" != "1" ]]; then
        echo "[check-only] Virtualization.framework unavailable (sysctl kern.hv_support != 1)."
    else
        echo "[check-only] Virtualization.framework detected."
    fi

    for binary in limactl jq envsubst file; do
        if command -v "${binary}" >/dev/null 2>&1; then
            echo "[check-only] ${binary} found."
        else
            echo "[check-only] ${binary} missing."
        fi
    done
    local commitment_preview
    commitment_preview="${INSTALL_BOOTSTRAP_COMMITMENT:0:12}..."
    echo "[check-only] Install context commitment: ${commitment_preview}"
    echo "[check-only] Declared VM name: ${VM_NAME}"
    echo "[check-only] Lima control root: resolved from account database."

    if vm_exists; then
        local status
        status="$(vm_status)"
        echo "[check-only] Lima VM '${VM_NAME}' status: ${status}"
        if [[ "${status}" == "Running" ]]; then
            local layout
            local machine_id_preview
            observe_lima_mapping_v1
            verify_lima_mapping_v1
            [[ -n "${OBSERVED_PLATFORM_MAPPING_V1}" ]] || fatal "Verified Lima mapping record is empty."
            layout="$(current_layout_version)"
            machine_id_preview="${OBSERVED_GUEST_MACHINE_ID:0:12}..."
            echo "[check-only] Observed guest machine-id prefix: ${machine_id_preview}"
            echo "[check-only] Observed guest substrate home resolved from account database."
            echo "[check-only] Observed transport target: selected-prefix/sock/agent.sock -> ${OBSERVED_TRANSPORT_GUEST_SOCKET}"
            echo "[check-only] Layout sentinel: ${layout:-missing}"
            if [[ "${layout}" != "${LAYOUT_VERSION}" ]]; then
                echo "[check-only] Layout mismatch: R3 lifecycle reconciliation is required before guest projection."
                echo "[check-only] Forwarding activation unavailable: R3 prerequisite unmet."
                exit 0
            fi
            echo "[check-only] Forwarding activation unavailable: R3 prerequisite unmet."
            if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n test -S /run/substrate.sock >/dev/null 2>&1; then
                local socket_meta
                socket_meta="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)"
                if [[ "${socket_meta}" == "root:substrate 660" ]]; then
                    echo "[check-only] Agent socket metadata: expected ownership/mode."
                else
                    echo "[check-only] Agent socket metadata: unexpected ownership/mode."
                fi
            else
                echo "[check-only] Agent socket missing inside guest."
            fi
            if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n test -f /etc/systemd/system/substrate-world-service.service >/dev/null 2>&1; then
                if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n grep -q '^Environment=WORLD_NETFILTER_ENABLE=1$' /etc/systemd/system/substrate-world-service.service >/dev/null 2>&1; then
                    echo "[check-only] Guest systemd env includes WORLD_NETFILTER_ENABLE=1."
                else
                    echo "[check-only] Guest systemd env does not include WORLD_NETFILTER_ENABLE=1."
                fi
            else
                echo "[check-only] Guest systemd service file for substrate-world-service is missing."
            fi
            if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n test -x /usr/local/bin/substrate-gateway >/dev/null 2>&1; then
                echo "[check-only] Guest gateway binary present at /usr/local/bin/substrate-gateway."
            else
                echo "[check-only] Guest gateway binary missing at /usr/local/bin/substrate-gateway."
            fi
        fi
    else
        echo "[check-only] Lima VM '${VM_NAME}' not found."
    fi

    exit 0
}

check_host_prereqs() {
    local host_os
    host_os="$(uname -s 2>/dev/null || echo "unknown")"
    if [[ "${host_os}" != "Darwin" ]]; then
        fatal "This helper only supports macOS hosts (detected ${host_os})."
    fi
    require_cmd limactl
    require_cmd jq
    require_cmd envsubst
    require_cmd file
    require_cmd sysctl

    local hv
    hv="$(sysctl -n kern.hv_support 2>/dev/null || echo "0")"
    if [[ "${hv}" != "1" ]]; then
        fatal "Virtualization.framework unavailable (sysctl kern.hv_support != 1). Enable it in System Settings > Privacy & Security."
    fi
}

render_profile() {
    TMP_PROFILE="$(mktemp)"
    trap 'rm -f "$TMP_PROFILE"' EXIT
    PROJECT="${PROJECT_PATH}" envsubst < "${PROFILE}" > "${TMP_PROFILE}"
}

vm_exists() {
    [[ "$(vm_status)" != "__missing__" ]]
}

vm_status() {
    local list_json
    list_json="$(run_limactl_with_mapping_env_v1 list "${VM_NAME}" --json 2>/dev/null)" \
        || fatal "Unable to inspect Lima instances for '${VM_NAME}'."
    printf '%s' "${list_json}" | jq -r --arg name "${VM_NAME}" '
        if type == "array" then
            (map(select(.name == $name)) | if length == 0 then "__missing__" else .[0].status // "unknown" end)
        elif (.name // "") == $name then
            .status // "unknown"
        else
            "__missing__"
        end
    ' || fatal "Unable to decode Lima status for '${VM_NAME}'."
}

create_vm() {
    local start_err
    log "Creating Lima VM '${VM_NAME}' from ${PROFILE} ..."
    start_err="$(mktemp)"
    if ! run_limactl_with_mapping_env_v1 start --tty=false --name "${VM_NAME}" "${TMP_PROFILE}" > /dev/null 2>"${start_err}"; then
        rm -f "${start_err}"
        fatal "Unable to create declared Lima VM '${VM_NAME}'."
    fi
    rm -f "${start_err}"
}

start_vm() {
    local start_err
    log "Starting existing Lima VM '${VM_NAME}' ..."
    start_err="$(mktemp)"
    if ! run_limactl_with_mapping_env_v1 start "${VM_NAME}" > /dev/null 2>"${start_err}"; then
        rm -f "${start_err}"
        fatal "Unable to start declared Lima VM '${VM_NAME}'."
    fi
    rm -f "${start_err}"
}

wait_for_running() {
    local remaining=${WAIT_TIMEOUT}
    while (( remaining > 0 )); do
        local status
        status="$(vm_status)"
        case "${status}" in
            Running)
                log "Lima VM '${VM_NAME}' is running."
                return
                ;;
            Starting)
                ;;
            __missing__)
                fatal "Lima VM '${VM_NAME}' disappeared while waiting for Running state."
                ;;
            *)
                fatal "Lima VM '${VM_NAME}' entered unsupported wait status '${status}' before reaching Running."
                ;;
        esac
        sleep 2
        remaining=$((remaining - 2))
    done
    fatal "Lima VM '${VM_NAME}' did not reach Running state within ${WAIT_TIMEOUT} seconds."
}

destroy_vm() {
    # The typed publisher owns destruction and exact before-state restoration.
    "${SCRIPT_DIR}/lima-lifecycle.sh" destroy-vm \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

current_layout_version() {
    limactl shell "${VM_NAME}" sudo -n cat "${LAYOUT_SENTINEL}" 2>/dev/null || true
}

ensure_vm_ready() {
    # No shell-selected VM is started or created here. The mapped executor receives the exact
    # signed Stage-1 authorization and validates it against the carrier/mapping before any action.
    "${SCRIPT_DIR}/lima-lifecycle.sh" ensure-vm-ready \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

host_git_head() {
    if ! command -v git >/dev/null 2>&1; then
        return
    fi
    git -C "${PROJECT_PATH}" rev-parse HEAD 2>/dev/null || true
}

create_stage_manifest() {
    local manifest_path git_head
    manifest_path="$(mktemp)"
    git_head="$(host_git_head)"
    {
        printf 'project_path=%s\n' "${PROJECT_PATH}"
        printf 'project_basename=%s\n' "$(basename "${PROJECT_PATH}")"
        if [[ -n "${git_head}" ]]; then
            printf 'git_head=%s\n' "${git_head}"
        fi
    } > "${manifest_path}"
    printf '%s\n' "${manifest_path}"
}

stage_workspace() {
    # The selected manifest and exact guest artifact transaction are executor-only.
    "${SCRIPT_DIR}/lima-lifecycle.sh" stage-workspace \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

verify_staged_workspace() {
    local expected_git_head expect_cargo_sources
    expected_git_head="$(host_git_head)"
    expect_cargo_sources=0
    if [[ -f "${PROJECT_PATH}/Cargo.toml" ]]; then
        expect_cargo_sources=1
    fi
    limactl shell "${VM_NAME}" sudo -n env \
        STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_CURRENT}" \
        STAGED_WORKSPACE_MANIFEST_NAME="${STAGED_WORKSPACE_MANIFEST_NAME}" \
        EXPECTED_PROJECT_PATH="${PROJECT_PATH}" \
        EXPECTED_GIT_HEAD="${expected_git_head}" \
        EXPECT_CARGO_SOURCES="${expect_cargo_sources}" \
        bash <<'EOF'
set -euo pipefail
manifest="${STAGED_WORKSPACE_CURRENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
test -d "${STAGED_WORKSPACE_CURRENT}"
test -f "${manifest}"
grep -Fqx "project_path=${EXPECTED_PROJECT_PATH}" "${manifest}"
if [[ -n "${EXPECTED_GIT_HEAD}" ]]; then
    grep -Fqx "git_head=${EXPECTED_GIT_HEAD}" "${manifest}"
fi
if [[ "${EXPECT_CARGO_SOURCES}" == "1" ]]; then
    test -f "${STAGED_WORKSPACE_CURRENT}/Cargo.toml"
fi
EOF
}

ensure_substrate_group() {
    # Membership mutation is part of the one mapped guest lifecycle transaction.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

host_agent_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/world-service"
        "${base}/bin/world-service-linux"
        "${base}/bin/world-service"
        "${base}/target/release/world-service"
        "${base}/target/debug/world-service"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_agent_from_host() {
    # Host-to-guest artifact ingress is accepted only through ExecutorBuildEvidenceV1.
    "${SCRIPT_DIR}/lima-lifecycle.sh" install-guest-artifacts \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

host_cli_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/substrate"
        "${base}/bin/substrate-linux"
        "${base}/bin/substrate"
        "${base}/target/aarch64-unknown-linux-gnu/${BUILD_PROFILE}/substrate"
        "${base}/target/x86_64-unknown-linux-gnu/${BUILD_PROFILE}/substrate"
        "${base}/target/${BUILD_PROFILE}/substrate"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_cli_from_host() {
    # Host-to-guest artifact ingress is accepted only through ExecutorBuildEvidenceV1.
    "${SCRIPT_DIR}/lima-lifecycle.sh" install-guest-artifacts \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

host_gateway_candidate() {
    local base="${PROJECT_PATH}"
    local candidates=(
        "${base}/bin/linux/substrate-gateway"
        "${base}/bin/substrate-gateway-linux"
        "${base}/bin/substrate-gateway"
        "${base}/target/release/substrate-gateway"
        "${base}/target/debug/substrate-gateway"
    )
    local path
    for path in "${candidates[@]}"; do
        if [[ -f "${path}" ]]; then
            local file_type
            file_type="$(file -b "${path}" 2>/dev/null || true)"
            if echo "${file_type}" | grep -qi "ELF"; then
                printf '%s\n' "${path}"
                return 0
            fi
        fi
    done
    return 1
}

install_gateway_from_host() {
    # Host-to-guest artifact ingress is accepted only through ExecutorBuildEvidenceV1.
    "${SCRIPT_DIR}/lima-lifecycle.sh" install-guest-artifacts \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

build_missing_components_inside_vm() {
    fatal "In-guest DNS, package, Rustup, Cargo, and build remediation is tombstoned; provide exact ExecutorBuildEvidenceV1 artifacts."
}

fix_dns() {
    fatal "Guest DNS mutation is tombstoned by the R3 mapped lifecycle executor."
}

ensure_cargo() {
    fatal "Guest package, Rustup, and Cargo remediation is tombstoned by the R3 mapped lifecycle executor."
}

run_guest_cargo_build() {
    fatal "Guest Cargo build is tombstoned; exact executor-built artifacts are required."
}

install_guest_binaries() {
    # No guest toolchain fallback is permitted; artifact validation and installation are one transaction.
    "${SCRIPT_DIR}/lima-lifecycle.sh" install-guest-artifacts \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

verify_guest_binaries() {
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
test -x /usr/local/bin/substrate-world-service
test -x /usr/local/bin/substrate-gateway
EOF
}

bootstrap_guest_private_home() {
    # Private-home creation/restoration belongs to the mapped executor receipt.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

write_systemd_units() {
    local guest_substrate_home="$1"
    # Preserve the pre-request projection guard: the executor never receives an unsafe
    # systemd-rendered field, even in a sourceable/static fixture invocation.
    for projected_unit_value in \
        "${guest_substrate_home}" \
        "${INSTALL_BOOTSTRAP_COMMITMENT}" \
        "${VM_NAME}" \
        "${HOST_PLATFORM_CONTROL_ROOT}" \
        "${OBSERVED_TRANSPORT_HOST}" \
        "${OBSERVED_TRANSPORT_GUEST_SOCKET}"; do
        case "${projected_unit_value}" in
            *\"*|*%*|*\\*)
                fatal "Verified guest unit projection contains a systemd-unsafe character."
                ;;
        esac
    done
    # Keep the exact rendered-unit projection bindings visible to the mapped-executor request.
    # This no-op is intentionally after validation: it preserves the R2 projection contract
    # without creating a unit or mutating guest service/socket state in the shell.
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}" \
    SUBSTRATE_LIMA_INSTANCE_NAME="${VM_NAME}" \
    SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT="${HOST_PLATFORM_CONTROL_ROOT}" \
    SUBSTRATE_LIMA_HOST_SOCKET="${OBSERVED_TRANSPORT_HOST}" \
    SUBSTRATE_LIMA_GUEST_SOCKET="${OBSERVED_TRANSPORT_GUEST_SOCKET}" \
        true

    # Exact unit installation is coupled to the mapped receipt and socket identity.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

enable_socket_activation() {
    # Service/socket state is prepared and restored only by the mapped executor.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

socket_summary() {
    local meta
    meta="$(limactl shell "${VM_NAME}" sudo stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)"
    if [[ -n "${meta}" ]]; then
        log "Agent socket perms: ${meta} (expected root:substrate 660)"
        if [[ "${meta}" != "root:substrate 660" ]]; then
            warn "Socket permissions differ from expected root:substrate 660."
        fi
    else
        warn "Unable to read /run/substrate.sock metadata."
    fi
}

write_layout_sentinel() {
    # Layout-sentinel mutation is part of the mapped executor transaction.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"
}

linger_guidance() {
    local vm_user="$1"
    local linger
    linger="$(limactl shell "${VM_NAME}" sudo -n loginctl show-user "${vm_user}" -p Linger 2>/dev/null | cut -d= -f2 || true)"
    if [[ "${linger}" != "yes" ]]; then
        warn "loginctl lingering for ${vm_user} is ${linger:-unknown}. Rerun \`substrate world enable\` (or this degraded-but-supported helper) after correcting it. Breakglass guest-admin repair: run 'limactl shell ${VM_NAME} sudo loginctl enable-linger ${vm_user}' so socket activation survives logout."
    else
        log "loginctl lingering already enabled for ${vm_user}."
    fi
}

configure_guest() {
    local layout

    observe_lima_mapping_v1
    verify_lima_mapping_v1
    [[ -n "${OBSERVED_PLATFORM_MAPPING_V1}" ]] || fatal "Verified Lima mapping record is empty."

    layout="$(current_layout_version)"
    if [[ "${layout}" != "${LAYOUT_VERSION}" ]]; then
        fatal "Lima VM layout (${layout:-missing}) does not match ${LAYOUT_VERSION}; R3 lifecycle reconciliation is required before guest projection."
    fi
    [[ "${PLATFORM_BOOTSTRAP_MAPPING_V1}" == "${OBSERVED_PLATFORM_MAPPING_V1}" ]] \
        || fatal "Supplied PlatformBootstrapMappingV1 does not equal the observed selected mapping."

    # This is the sole mutable guest-projection request. The executor receives the selected
    # mapping, PM carrier, and exact artifact evidence and owns prepare/receipt/restore.
    "${SCRIPT_DIR}/lima-lifecycle.sh" configure-guest \
        --install-prefix "${INSTALL_PREFIX_RAW}" \
        --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" \
        --platform-bootstrap-mapping-v1 "${PLATFORM_BOOTSTRAP_MAPPING_V1}" \
        --executor-build-evidence-v1 "${EXECUTOR_BUILD_EVIDENCE_V1}" \
        --publisher-request-v1 "${PUBLISHER_REQUEST_V1}" \
        --lima-stage-one-authorization-v1 "${LIMA_STAGE_ONE_AUTHORIZATION_V1}"

    socket_summary
    linger_guidance "${OBSERVED_GUEST_ACCOUNT}"
}

if [[ ${HELP_REQUESTED} -eq 1 ]]; then
    usage
    exit 0
fi

resolve_install_bootstrap_context_v1
resolve_lima_control_root_v1

if [[ ${CHECK_ONLY} -eq 1 ]]; then
    check_only_status
fi

require_mapped_lifecycle_authority_v1
check_host_prereqs
render_profile
ensure_vm_ready
configure_guest

cat <<EOF
Supported operator path:
  supported: substrate host doctor [--json]; substrate world doctor [--json]; substrate world gateway sync|status|restart; substrate world enable; substrate world deps current sync for dependency reconciliation
  degraded-but-supported: scripts/mac/lima-doctor.sh remains the routed-first wrapper for doctor proof; scripts/mac/lima-warm.sh remains the current macOS declared-instance Stage-1 create/start plus matching-layout guest-projection wrapper
  note: substrate workspace sync is not yet the frozen normal macOS same-user Lima sync/copy contract in this packet
  breakglass: raw limactl shell, plain SSH, direct guest systemctl, guest socket curl, guest journalctl, and host-side SUBSTRATE_WORLD_SOCKET override use
EOF

log "Lima world backend '${VM_NAME}' reached the current bounded mapping prerequisites. Preferred supported operations: substrate host doctor [--json]; substrate world doctor [--json]; substrate world gateway sync|status|restart; substrate world enable when provisioning is needed; substrate world deps current sync when guest dependency reconciliation is needed."
log "This helper remains degraded-but-supported for macOS declared-instance Stage-1 create/start plus matching-layout guest projection."
log "Escalate to raw limactl shell, plain SSH, direct guest systemctl/journalctl, guest socket curl, or host-side SUBSTRATE_WORLD_SOCKET override use only as breakglass."
log "Optional orchestration parity proof: scripts/mac/orchestration-smoke.sh"
