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
    warn "Destroying Lima VM '${VM_NAME}' to apply socket parity layout..."
    limactl stop "${VM_NAME}" >/dev/null 2>&1 || true
    limactl delete "${VM_NAME}" >/dev/null 2>&1 || true
}

current_layout_version() {
    limactl shell "${VM_NAME}" sudo -n cat "${LAYOUT_SENTINEL}" 2>/dev/null || true
}

ensure_vm_ready() {
    if vm_exists; then
        local status
        status="$(vm_status)"
        case "${status}" in
            Running)
                log "Lima VM '${VM_NAME}' already running."
                ;;
            Stopped)
                warn "Lima VM '${VM_NAME}' status: ${status}; attempting to start."
                start_vm
                ;;
            *)
                fatal "Lima VM '${VM_NAME}' has unsupported Stage-1 status '${status}'. Expected confirmed absence, Stopped, or Running."
                ;;
        esac
    else
        create_vm
    fi

    wait_for_running
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
    local vm_user="$1"
    local workspace_name stage_parent manifest_path entry entry_name
    workspace_name="$(basename "${PROJECT_PATH}")"
    stage_parent="/tmp/substrate-stage-workspace"
    manifest_path="$(create_stage_manifest)"

    log "Staging workspace input into guest-local path ${STAGED_WORKSPACE_CURRENT}"
    # Keep the ingress path aligned with Slice 09 / Slice 10 authority and validation:
    # transfer the requested workspace directly via `limactl copy` rather than
    # via a separate host-side staged tree, while preserving the explicit
    # exclusions that bound guest staging to the supported validation surface.
    limactl shell "${VM_NAME}" env STAGE_PARENT="${stage_parent}" WORKSPACE_NAME="${workspace_name}" bash <<'EOF'
set -euo pipefail
rm -rf "${STAGE_PARENT}"
mkdir -p "${STAGE_PARENT}/${WORKSPACE_NAME}"
EOF
    shopt -s dotglob nullglob
    for entry in "${PROJECT_PATH}"/*; do
        entry_name="$(basename "${entry}")"
        case "${entry_name}" in
            .git|target|.codex|.DS_Store)
                continue
                ;;
        esac
        if [[ -d "${entry}" ]]; then
            limactl copy --recursive "${entry}" "${VM_NAME}:${stage_parent}/${workspace_name}/"
        else
            limactl copy "${entry}" "${VM_NAME}:${stage_parent}/${workspace_name}/"
        fi
    done
    shopt -u dotglob nullglob
    limactl copy "${manifest_path}" "${VM_NAME}:${stage_parent}/${STAGED_WORKSPACE_MANIFEST_NAME}"
    limactl shell "${VM_NAME}" \
        env STAGE_PARENT="${stage_parent}" \
            WORKSPACE_NAME="${workspace_name}" \
            STAGED_WORKSPACE_ROOT="${STAGED_WORKSPACE_ROOT}" \
            STAGED_WORKSPACE_CURRENT="${STAGED_WORKSPACE_CURRENT}" \
            STAGED_WORKSPACE_MANIFEST_NAME="${STAGED_WORKSPACE_MANIFEST_NAME}" \
            VM_USER="${vm_user}" \
        bash <<'EOF'
set -euo pipefail
test -d "${STAGE_PARENT}/${WORKSPACE_NAME}"
test -f "${STAGE_PARENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
# Finder metadata can disappear between traversal and deletion while the staged
# tree is being copied into place; use rm -f so transient ENOENTs do not abort
# the supported staging path.
find "${STAGE_PARENT}/${WORKSPACE_NAME}" -name '.DS_Store' -exec rm -f {} +
sudo install -d -o root -g substrate -m0750 "${STAGED_WORKSPACE_ROOT}"
sudo rm -rf "${STAGED_WORKSPACE_CURRENT}"
sudo mv "${STAGE_PARENT}/${WORKSPACE_NAME}" "${STAGED_WORKSPACE_CURRENT}"
sudo chown -R "${VM_USER}:substrate" "${STAGED_WORKSPACE_CURRENT}"
sudo chmod 0750 "${STAGED_WORKSPACE_CURRENT}"
sudo install -o "${VM_USER}" -g substrate -m0640 \
    "${STAGE_PARENT}/${STAGED_WORKSPACE_MANIFEST_NAME}" \
    "${STAGED_WORKSPACE_CURRENT}/${STAGED_WORKSPACE_MANIFEST_NAME}"
rm -rf "${STAGE_PARENT}"
EOF
    rm -f "${manifest_path}"
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
    local vm_user="$1"
    limactl shell "${VM_NAME}" bash <<EOF
set -euo pipefail
if ! getent group substrate >/dev/null 2>&1; then
    sudo groupadd --system substrate
fi
if id -nG "${vm_user}" | tr ' ' '\n' | grep -qx substrate; then
    exit 0
fi
sudo usermod -aG substrate "${vm_user}"
EOF
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
    local agent_path="$1"
    log "Installing Linux world-service from ${agent_path}"
    limactl copy "${agent_path}" "${VM_NAME}:/tmp/world-service"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/world-service /usr/local/bin/substrate-world-service
sudo rm -f /tmp/world-service
EOF
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
    local cli_path="$1"
    log "Installing Linux substrate CLI from ${cli_path}"
    limactl copy "${cli_path}" "${VM_NAME}:/tmp/substrate-cli"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/substrate-cli /usr/local/bin/substrate
sudo tee /usr/local/bin/world >/dev/null <<'WORLD'
#!/usr/bin/env bash
exec substrate world "$@"
WORLD
sudo chmod 0755 /usr/local/bin/world
sudo rm -f /tmp/substrate-cli
EOF
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
    local gateway_path="$1"
    log "Installing Linux substrate-gateway from ${gateway_path}"
    limactl copy "${gateway_path}" "${VM_NAME}:/tmp/substrate-gateway"
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0755 /tmp/substrate-gateway /usr/local/bin/substrate-gateway
sudo rm -f /tmp/substrate-gateway
EOF
}

build_missing_components_inside_vm() {
    local build_cli="${1:-0}"
    local build_agent="${2:-0}"
    local build_gateway="${3:-0}"

    if [[ "${build_cli}" -ne 1 && "${build_agent}" -ne 1 && "${build_gateway}" -ne 1 ]]; then
        return 0
    fi

    if [[ "${build_agent}" -eq 1 ]]; then
        log "Building Linux world-service inside Lima (profile: ${BUILD_PROFILE})"
    fi
    if [[ "${build_cli}" -eq 1 ]]; then
        log "Building Linux substrate CLI inside Lima for diagnostics (profile: ${BUILD_PROFILE})"
    fi
    if [[ "${build_gateway}" -eq 1 ]]; then
        log "Building Linux substrate-gateway inside Lima (profile: ${BUILD_PROFILE})"
    fi

    if [[ ! -f "${PROJECT_PATH}/Cargo.toml" ]]; then
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Linux world-service missing and ${PROJECT_PATH} does not contain Cargo sources. Provide bin/linux/world-service or rerun from a source checkout."
        fi
        if [[ "${build_gateway}" -eq 1 ]]; then
            fatal "Linux substrate-gateway missing and ${PROJECT_PATH} does not contain Cargo sources. Provide bin/linux/substrate-gateway or rerun from a source checkout."
        fi
        warn "Skipping guest CLI build; ${PROJECT_PATH} lacks Cargo sources."
        # The guest CLI is optional (diagnostics only) and release bundles don't ship Cargo sources.
        # Do not abort provisioning when only the guest CLI is missing.
        return 0
    fi

    local status=0
    if limactl shell "${VM_NAME}" env BUILD_PROFILE="${BUILD_PROFILE}" BUILD_GUEST_CLI="${build_cli}" BUILD_GUEST_AGENT="${build_agent}" BUILD_GUEST_GATEWAY="${build_gateway}" STAGED_WORKSPACE_PATH="${STAGED_WORKSPACE_CURRENT}" bash <<'EOF'
set -euo pipefail
build_cli="${BUILD_GUEST_CLI:-0}"
build_agent="${BUILD_GUEST_AGENT:-0}"
build_gateway="${BUILD_GUEST_GATEWAY:-0}"

fix_dns() {
    local probe_host="${1:-ports.ubuntu.com}"
    if getent hosts "${probe_host}" >/dev/null 2>&1; then
        return 0
    fi
    echo "[lima-warm] DNS resolution failed inside Lima for ${probe_host}; applying fallback resolv.conf (1.1.1.1 / 8.8.8.8)..." >&2
    local SUDO_CMD="sudo"
    if sudo -n true 2>/dev/null; then
        SUDO_CMD="sudo -n"
    fi
    $SUDO_CMD sh -c "printf 'nameserver 1.1.1.1\nnameserver 8.8.8.8\n' > /etc/resolv.conf" || true
    $SUDO_CMD systemctl restart dnsmasq 2>/dev/null || true
    $SUDO_CMD systemctl restart systemd-resolved 2>/dev/null || true
    getent hosts "${probe_host}" >/dev/null 2>&1
}

ensure_cargo() {
    # Cargo.lock v4 requires a newer cargo than Ubuntu 24.04's apt cargo on some images.
    # Prefer rustup when we detect a v4 lockfile so we don't fail during `cargo build --locked`.
    local needs_lockfile_v4=0
    if sudo -u "$(id -un)" -g substrate test -f "${STAGED_WORKSPACE_PATH}/Cargo.lock" \
        && sudo -u "$(id -un)" -g substrate grep -qx 'version = 4' "${STAGED_WORKSPACE_PATH}/Cargo.lock" 2>/dev/null; then
        needs_lockfile_v4=1
    fi

    if command -v rustup >/dev/null 2>&1; then
        if [ -f "$HOME/.cargo/env" ]; then
            # shellcheck disable=SC1090
            source "$HOME/.cargo/env"
        fi
        export PATH="$HOME/.cargo/bin:$PATH"
        rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
        rustup default stable >/dev/null 2>&1 || true
        if command -v cargo >/dev/null 2>&1; then
            return 0
        fi
    fi

    if [[ "${needs_lockfile_v4}" -eq 1 ]]; then
        echo "[lima-warm] Cargo.lock v4 detected; installing rustup toolchain (stable)..." >&2
        fix_dns ports.ubuntu.com || true
        if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
            # shellcheck disable=SC1090
            source "$HOME/.cargo/env"
            export PATH="$HOME/.cargo/bin:$PATH"
            rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
            rustup default stable >/dev/null 2>&1 || true
            command -v cargo >/dev/null 2>&1
            return $?
        fi
        return 1
    fi

    if command -v cargo >/dev/null 2>&1; then
        return 0
    fi
    echo "[lima-warm] cargo not found inside Lima VM; attempting apt install (rustc cargo)..." >&2
    local SUDO="sudo"
    if sudo -n true 2>/dev/null; then
        SUDO="sudo -n"
    fi
    fix_dns ports.ubuntu.com || true
    if $SUDO apt-get update && $SUDO apt-get install -y rustc cargo; then
        return 0
    fi
    echo "[lima-warm] apt install failed; trying rustup via curl (IPv4, retries)..." >&2
    fix_dns ports.ubuntu.com || true
    if curl -4 --connect-timeout 10 --retry 3 --retry-delay 1 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal; then
        # shellcheck disable=SC1090
        source "$HOME/.cargo/env"
        return 0
    fi
    return 1
}

if [[ "${build_cli}" != "1" && "${build_agent}" != "1" && "${build_gateway}" != "1" ]]; then
    exit 0
fi

if ! ensure_cargo; then
    echo "[lima-warm][ERROR] unable to install cargo inside Lima VM; install Rust manually or provide Linux binaries." >&2
    exit 1
fi
if command -v rustup >/dev/null 2>&1; then
    rustup toolchain install stable --profile minimal >/dev/null 2>&1 || true
    rustup default stable >/dev/null 2>&1 || true
fi
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.cargo/env"
fi
cargo_bin="$(command -v cargo || true)"
if [[ -z "${cargo_bin}" ]]; then
    echo "[lima-warm][ERROR] cargo still missing after toolchain installation." >&2
    exit 1
fi
BUILD_DIR="/tmp/substrate-lima-build"
BUILD_OUTPUT_DIR="${BUILD_PROFILE}"
BUILD_PROFILE_FLAG=()
case "${BUILD_PROFILE}" in
debug)
    BUILD_OUTPUT_DIR="debug"
    ;;
release)
    BUILD_OUTPUT_DIR="release"
    BUILD_PROFILE_FLAG=(--profile release)
    ;;
*)
    BUILD_PROFILE_FLAG=(--profile "${BUILD_PROFILE}")
    ;;
esac
mkdir -p "${BUILD_DIR}"
workspace_dir="${STAGED_WORKSPACE_PATH}"
if ! sudo -u "$(id -un)" -g substrate test -r "${workspace_dir}/Cargo.toml"; then
    echo "[lima-warm][ERROR] staged workspace is not readable under the substrate group: ${workspace_dir}" >&2
    exit 1
fi
run_guest_cargo_build() {
    fix_dns static.crates.io || true
    if sudo -u "$(id -un)" -g substrate env \
        HOME="$HOME" \
        PATH="$PATH" \
        CARGO_TARGET_DIR="${BUILD_DIR}" \
        bash -lc 'cd "$1" && shift && exec "$@"' bash \
        "${workspace_dir}" "${cargo_bin}" build "$@"; then
        return 0
    fi
    fix_dns static.crates.io || true
    sudo -u "$(id -un)" -g substrate env \
        HOME="$HOME" \
        PATH="$PATH" \
        CARGO_TARGET_DIR="${BUILD_DIR}" \
        bash -lc 'cd "$1" && shift && exec "$@"' bash \
        "${workspace_dir}" "${cargo_bin}" build "$@"
}
mandatory_build_failed=0
cli_build_failed=0
if [[ "${build_agent}" == "1" ]]; then
    if ! run_guest_cargo_build -p world-service "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][ERROR] failed to build Linux world-service inside Lima." >&2
        mandatory_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/world-service" /usr/local/bin/substrate-world-service
    fi
fi
if [[ "${build_gateway}" == "1" ]]; then
    if ! run_guest_cargo_build -p substrate-gateway "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][ERROR] failed to build Linux substrate-gateway inside Lima." >&2
        mandatory_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/substrate-gateway" /usr/local/bin/substrate-gateway
    fi
fi
if [[ "${build_cli}" == "1" ]]; then
    if ! run_guest_cargo_build --bin substrate "${BUILD_PROFILE_FLAG[@]}" --locked; then
        echo "[lima-warm][WARN] failed to build the optional Linux substrate CLI inside Lima; continuing because diagnostics can fall back to the host CLI." >&2
        cli_build_failed=1
    else
        sudo install -Dm0755 "${BUILD_DIR}/${BUILD_OUTPUT_DIR}/substrate" /usr/local/bin/substrate
        sudo tee /usr/local/bin/world >/dev/null <<'WORLD'
#!/usr/bin/env bash
exec substrate world "$@"
WORLD
        sudo chmod 0755 /usr/local/bin/world
    fi
fi
rm -rf "${BUILD_DIR}" || true
if [[ "${mandatory_build_failed}" -ne 0 ]]; then
    exit 1
fi
if [[ "${cli_build_failed}" -ne 0 ]]; then
    echo "[lima-warm][WARN] Guest provisioning completed without a Linux substrate CLI binary." >&2
fi
EOF
    then
        status=0
    else
        status=$?
    fi
    if [[ "${status}" -ne 0 ]]; then
        if [[ "${build_agent}" -eq 1 && "${build_gateway}" -eq 1 ]]; then
            fatal "Failed to build mandatory Linux guest binaries inside Lima (world-service and/or substrate-gateway) (exit ${status}). Provide prebuilt binaries under bin/linux/ or rerun from a source checkout."
        fi
        if [[ "${build_agent}" -eq 1 ]]; then
            fatal "Failed to build Linux world-service inside Lima (exit ${status}). Provide a prebuilt agent under bin/linux/world-service or rerun from a source checkout."
        fi
        if [[ "${build_gateway}" -eq 1 ]]; then
            fatal "Failed to build Linux substrate-gateway inside Lima (exit ${status}). Provide a prebuilt gateway under bin/linux/substrate-gateway or rerun from a source checkout."
        fi
        warn "Failed to build Linux CLI inside Lima; diagnostics requiring a guest CLI will need to run on the host."
        return 0
    fi
}

install_guest_binaries() {
    local cli_candidate agent_candidate gateway_candidate
    local need_cli_build=0
    local need_agent_build=0
    local need_gateway_build=0

    cli_candidate="$(host_cli_candidate)" || true
    if [[ -n "${cli_candidate:-}" ]]; then
        install_cli_from_host "${cli_candidate}"
    else
        if [[ -f "${PROJECT_PATH}/Cargo.toml" ]]; then
            log "Linux substrate CLI not found in ${PROJECT_PATH}; attempting in-guest build for diagnostics."
            need_cli_build=1
        else
            warn "Linux substrate CLI not found in ${PROJECT_PATH}; skipping guest CLI install (no Cargo sources in bundle). Diagnostics will fall back to host CLI."
        fi
    fi

    agent_candidate="$(host_agent_candidate)" || true
    if [[ -n "${agent_candidate:-}" ]]; then
        install_agent_from_host "${agent_candidate}"
    else
        log "Linux world-service binary not found or invalid in ${PROJECT_PATH}; falling back to an in-guest build."
        need_agent_build=1
    fi

    gateway_candidate="$(host_gateway_candidate)" || true
    if [[ -n "${gateway_candidate:-}" ]]; then
        install_gateway_from_host "${gateway_candidate}"
    else
        log "Linux substrate-gateway binary not found or invalid in ${PROJECT_PATH}; falling back to an in-guest build."
        need_gateway_build=1
    fi

    if [[ "${need_cli_build}" -eq 1 || "${need_agent_build}" -eq 1 || "${need_gateway_build}" -eq 1 ]]; then
        if [[ "${SKIP_GUEST_BUILD}" -eq 1 ]]; then
            if [[ "${need_agent_build}" -eq 1 ]]; then
                warn "Linux world-service missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Ensure another step installs /usr/local/bin/substrate-world-service."
            fi
            if [[ "${need_gateway_build}" -eq 1 ]]; then
                warn "Linux substrate-gateway missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Ensure another step installs /usr/local/bin/substrate-gateway."
            fi
            if [[ "${need_cli_build}" -eq 1 ]]; then
                warn "Linux CLI missing but SUBSTRATE_LIMA_SKIP_GUEST_BUILD=1; skipping guest build. Diagnostics will fall back to host CLI."
            fi
            return 0
        fi
        build_missing_components_inside_vm "${need_cli_build}" "${need_agent_build}" "${need_gateway_build}"
    fi
}

verify_guest_binaries() {
    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
test -x /usr/local/bin/substrate-world-service
test -x /usr/local/bin/substrate-gateway
EOF
}

bootstrap_guest_private_home() {
    local vm_user="$1"
    local guest_substrate_home="$2"
    log "Bootstrapping private guest SUBSTRATE_HOME for ${vm_user}"
    limactl shell "${VM_NAME}" env \
        SUBSTRATE_GUEST_HOME="${guest_substrate_home}" \
        SUBSTRATE_GUEST_USER="${vm_user}" \
        bash -s <<'EOF'
set -euo pipefail
if [[ -x /usr/local/bin/substrate ]]; then
    SUBSTRATE_HOME="${SUBSTRATE_GUEST_HOME}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${SUBSTRATE_GUEST_USER}" \
        /usr/local/bin/substrate --version >/dev/null
    exit 0
fi

# The guest CLI is an optional diagnostic artifact in the currently landed Lima
# architecture. Preserve that supported path with a process-isolated, no-follow
# provisioner that implements the same greenfield private-root acceptance rules.
python3 - "${SUBSTRATE_GUEST_HOME}" <<'PY'
import fcntl
import errno
import os
import stat
import struct
import sys

raw = sys.argv[1]
parts = raw.split("/")
if not raw.startswith("/") or any(part in (".", "..") for part in parts):
    raise SystemExit("unsupported guest SUBSTRATE_HOME: invalid physical path")
normal = [part for part in parts if part]
if not normal:
    raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-type")

directory_flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
def acl_grants_named_principal(fd, name):
    try:
        value = os.getxattr(fd, name)
    except OSError as error:
        if error.errno in (errno.ENODATA, getattr(errno, "ENOATTR", errno.ENODATA)):
            return False
        raise SystemExit("unsupported guest SUBSTRATE_HOME: ACL validation-unavailable")
    if len(value) < 4 or struct.unpack_from("<I", value)[0] != 2 or (len(value) - 4) % 8:
        return True
    mask = None
    user_object = False
    group_object = False
    other = False
    named = []
    for offset in range(4, len(value), 8):
        tag, permissions, identifier = struct.unpack_from("<HHI", value, offset)
        if permissions & ~0o7:
            return True
        if tag in (0x0002, 0x0008):
            if identifier == 0xffffffff or any(
                    seen_tag == tag and seen_id == identifier
                    for seen_tag, seen_id, _permissions in named):
                return True
            named.append((tag, identifier, permissions))
        elif tag == 0x0010:
            if identifier != 0xffffffff or mask is not None:
                return True
            mask = permissions
        elif tag == 0x0001:
            if identifier != 0xffffffff or user_object:
                return True
            user_object = True
        elif tag == 0x0004:
            if identifier != 0xffffffff or group_object:
                return True
            group_object = True
        elif tag == 0x0020:
            if identifier != 0xffffffff or other:
                return True
            other = True
        else:
            return True
    if not user_object or not group_object or not other:
        return True
    if not named:
        return False
    if mask is None:
        return True
    return any(permissions & mask for _tag, _identifier, permissions in named)

def validate_ancestor(fd):
    value = os.fstat(fd)
    if not stat.S_ISDIR(value.st_mode):
        raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-type ancestor")
    if value.st_uid not in (0, os.geteuid()):
        raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-owner ancestor")
    if stat.S_IMODE(value.st_mode) & 0o022:
        raise SystemExit("unsupported guest SUBSTRATE_HOME: unsafe ancestor mode")
    if acl_grants_named_principal(fd, "system.posix_acl_access") or \
            acl_grants_named_principal(fd, "system.posix_acl_default"):
        raise SystemExit("unsupported guest SUBSTRATE_HOME: foreign-acl ancestor")
    return value

chain = [os.open("/", directory_flags)]
chain_names = []
chain_identities = []
try:
    root_stat = validate_ancestor(chain[0])
    chain_identities.append((root_stat.st_dev, root_stat.st_ino))
    for component in normal[:-1]:
        observed = os.stat(component, dir_fd=chain[-1], follow_symlinks=False)
        if not stat.S_ISDIR(observed.st_mode):
            raise SystemExit("unsupported guest SUBSTRATE_HOME: symlink-or-wrong-type ancestor")
        next_fd = os.open(component, directory_flags, dir_fd=chain[-1])
        opened = validate_ancestor(next_fd)
        if (opened.st_dev, opened.st_ino) != (observed.st_dev, observed.st_ino):
            os.close(next_fd)
            raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced ancestor")
        chain_names.append(component)
        chain_identities.append((opened.st_dev, opened.st_ino))
        chain.append(next_fd)

    current = chain[-1]
    leaf = normal[-1]
    parent = os.fstat(current)
    if not stat.S_ISDIR(parent.st_mode):
        raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-type parent")
    if parent.st_uid not in (0, os.geteuid()):
        raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-owner parent")
    if stat.S_IMODE(parent.st_mode) & 0o022:
        raise SystemExit("unsupported guest SUBSTRATE_HOME: unsafe parent mode")
    # Cooperative Substrate creators serialize candidate initialization. Malicious root or
    # same-UID replacement before first open is outside the A1 V1 threat model.
    fcntl.flock(current, fcntl.LOCK_EX)
    created = False
    previous_umask = os.umask(0)
    try:
        try:
            os.mkdir(leaf, 0o700, dir_fd=current)
            created = True
        except FileExistsError:
            pass
    finally:
        os.umask(previous_umask)

    try:
        accepted = os.open(leaf, directory_flags, dir_fd=current)
    except OSError:
        observed = os.stat(leaf, dir_fd=current, follow_symlinks=False)
        if stat.S_ISLNK(observed.st_mode):
            raise SystemExit("unsupported guest SUBSTRATE_HOME: symlink")
        if not stat.S_ISDIR(observed.st_mode):
            raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-type")
        if observed.st_uid != os.geteuid():
            raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-owner")
        if stat.S_IMODE(observed.st_mode) != 0o700:
            raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-mode")
        raise SystemExit("unsupported guest SUBSTRATE_HOME: validation-unavailable")
    try:
        # This first no-follow open is the only source of accepted child identity. mkdir above
        # established a candidate name only; it returned no inode-bound handle.
        opened = os.fstat(accepted)
        if created:
            os.fchmod(accepted, 0o700)
            opened = os.fstat(accepted)
        if opened.st_uid != os.geteuid():
            raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-owner")
        if stat.S_IMODE(opened.st_mode) != 0o700:
            raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-mode")
        acl_names = set(os.listxattr(accepted))
        if {"system.posix_acl_access", "system.posix_acl_default"} & acl_names:
            raise SystemExit("unsupported guest SUBSTRATE_HOME: foreign-acl")
        reopened = os.open(leaf, directory_flags, dir_fd=current)
        try:
            reopened_stat = os.fstat(reopened)
            if (reopened_stat.st_dev, reopened_stat.st_ino) != (opened.st_dev, opened.st_ino):
                raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced")
            if not stat.S_ISDIR(reopened_stat.st_mode):
                raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-type")
            if reopened_stat.st_uid != os.geteuid():
                raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-owner")
            if stat.S_IMODE(reopened_stat.st_mode) != 0o700:
                raise SystemExit("unsupported guest SUBSTRATE_HOME: wrong-mode")
            reopened_acls = set(os.listxattr(reopened))
            if {"system.posix_acl_access", "system.posix_acl_default"} & reopened_acls:
                raise SystemExit("unsupported guest SUBSTRATE_HOME: foreign-acl")
            named = os.stat(leaf, dir_fd=current, follow_symlinks=False)
            if (named.st_dev, named.st_ino) != (reopened_stat.st_dev, reopened_stat.st_ino):
                raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced")
        finally:
            os.close(reopened)
        current_parent = validate_ancestor(current)
        if (current_parent.st_dev, current_parent.st_ino) != (parent.st_dev, parent.st_ino):
            raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced parent")
        for index, component in enumerate(chain_names):
            current_ancestor = validate_ancestor(chain[index + 1])
            if (current_ancestor.st_dev, current_ancestor.st_ino) != chain_identities[index + 1]:
                raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced ancestor")
            named_ancestor = os.stat(component, dir_fd=chain[index], follow_symlinks=False)
            if (named_ancestor.st_dev, named_ancestor.st_ino) != chain_identities[index + 1]:
                raise SystemExit("unsupported guest SUBSTRATE_HOME: replaced ancestor")
        os.fsync(accepted)
        os.fsync(current)
    finally:
        os.close(accepted)
finally:
    for descriptor in reversed(chain):
        os.close(descriptor)
PY
EOF
}

write_systemd_units() {
    local guest_substrate_home="$1"
    local enable_netfilter="${SUBSTRATE_WORLD_NETFILTER_ENABLE:-0}"
    local service_template="${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl"
    local socket_template="${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket"
    local rendered_units_dir
    local netfilter_env=""

    [[ -f "${service_template}" ]] || fatal "Missing canonical service unit source: ${service_template}"
    [[ -f "${socket_template}" ]] || fatal "Missing canonical socket unit source: ${socket_template}"

    case "${enable_netfilter}" in
        1|true|yes|TRUE|YES)
            log "Installing canonical guest systemd units with WORLD_NETFILTER_ENABLE=1"
            netfilter_env="Environment=WORLD_NETFILTER_ENABLE=1"
            ;;
        *)
            log "Installing canonical guest systemd units without WORLD_NETFILTER_ENABLE=1"
            ;;
    esac
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
    rendered_units_dir="$(mktemp -d)"
    SUBSTRATE_GUEST_HOME="${guest_substrate_home}" \
    SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}" \
    SUBSTRATE_LIMA_INSTANCE_NAME="${VM_NAME}" \
    SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT="${HOST_PLATFORM_CONTROL_ROOT}" \
    SUBSTRATE_LIMA_HOST_SOCKET="${OBSERVED_TRANSPORT_HOST}" \
    SUBSTRATE_LIMA_GUEST_SOCKET="${OBSERVED_TRANSPORT_GUEST_SOCKET}" \
    WORLD_NETFILTER_ENV="${netfilter_env}" \
        envsubst < "${service_template}" > "${rendered_units_dir}/substrate-world-service.service"
    envsubst < "${socket_template}" > "${rendered_units_dir}/substrate-world-service.socket"

    limactl copy "${rendered_units_dir}/substrate-world-service.service" \
        "${VM_NAME}:/tmp/substrate-world-service.service"
    limactl copy "${rendered_units_dir}/substrate-world-service.socket" \
        "${VM_NAME}:/tmp/substrate-world-service.socket"
    rm -rf "${rendered_units_dir}"

    limactl shell "${VM_NAME}" bash <<'EOF'
set -euo pipefail
sudo install -Dm0644 /tmp/substrate-world-service.service /etc/systemd/system/substrate-world-service.service
sudo install -Dm0644 /tmp/substrate-world-service.socket /etc/systemd/system/substrate-world-service.socket
sudo rm -f /tmp/substrate-world-service.service /tmp/substrate-world-service.socket
EOF
}

enable_socket_activation() {
    local guest_substrate_home="$1"
    limactl shell "${VM_NAME}" env SUBSTRATE_GUEST_HOME="${guest_substrate_home}" bash <<'EOF'
set -euo pipefail
legacy_unit_prefix="substrate-world"
legacy_service="${legacy_unit_prefix}-agent.service"
legacy_socket="${legacy_unit_prefix}-agent.socket"
sudo install -d -m0750 -o root -g substrate /var/lib/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate/substrate-gateway-runtime
sudo systemctl stop "${legacy_service}" "${legacy_socket}" >/dev/null 2>&1 || true
sudo systemctl disable "${legacy_service}" "${legacy_socket}" >/dev/null 2>&1 || true
sudo rm -f "/etc/systemd/system/${legacy_service}" "/etc/systemd/system/${legacy_socket}"
sudo systemctl daemon-reload
sudo systemctl enable substrate-world-service.service >/dev/null
sudo systemctl enable substrate-world-service.socket >/dev/null
sudo systemctl stop substrate-world-service.service >/dev/null 2>&1 || true
sudo systemctl stop substrate-world-service.socket >/dev/null 2>&1 || true
sudo install -d -m0750 -o root -g substrate /run/substrate
sudo install -d -m0750 -o root -g substrate /run/substrate/substrate-gateway-runtime
sudo rm -f /run/substrate.sock
sudo systemctl start substrate-world-service.socket
sudo systemctl start substrate-world-service.service
EOF
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
    limactl shell "${VM_NAME}" bash <<EOF
set -euo pipefail
echo "${LAYOUT_VERSION}" | sudo tee "${LAYOUT_SENTINEL}" >/dev/null
EOF
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

    local vm_user
    vm_user="${OBSERVED_GUEST_ACCOUNT}"
    if [[ -z "${vm_user}" ]]; then
        fatal "Unable to determine Lima guest user."
    fi
    ensure_substrate_group "${vm_user}"
    stage_workspace "${vm_user}"
    verify_staged_workspace
    install_guest_binaries
    verify_guest_binaries
    bootstrap_guest_private_home "${vm_user}" "${OBSERVED_GUEST_SUBSTRATE_HOME}"
    write_systemd_units "${OBSERVED_GUEST_SUBSTRATE_HOME}"
    enable_socket_activation "${OBSERVED_GUEST_SUBSTRATE_HOME}"
    socket_summary
    write_layout_sentinel
    linger_guidance "${vm_user}"
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
