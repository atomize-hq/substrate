#!/usr/bin/env bash
set -euo pipefail

failures=0
breakglass_summary_failures=0
LAYOUT_EXPECTED="socket-parity-v2-staged-workspace-v1"
SOURCE_PATH="${BASH_SOURCE[0]}"
while [[ -L "${SOURCE_PATH}" ]]; do
    SOURCE_DIR="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
    SOURCE_PATH="$(readlink "${SOURCE_PATH}")"
    [[ "${SOURCE_PATH}" != /* ]] && SOURCE_PATH="${SOURCE_DIR}/${SOURCE_PATH}"
done
SCRIPTS_ROOT="$(cd "$(dirname "${SOURCE_PATH}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
PROJECT_PATH="${SUBSTRATE_PROJECT_PATH:-$(pwd)}"
if [[ -d "${PROJECT_PATH}" ]]; then
    PROJECT_PATH="$(cd "${PROJECT_PATH}" && pwd)"
else
    PROJECT_PATH="${REPO_ROOT}"
fi
project_unit_source_dir="${PROJECT_PATH}/scripts/mac/lima/units"
script_unit_source_dir="${SCRIPTS_ROOT}/lima/units"
if [[ -d "${project_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${project_unit_source_dir}"
elif [[ -d "${script_unit_source_dir}" ]]; then
    CANONICAL_UNIT_SOURCE_DIR="${script_unit_source_dir}"
else
    echo "ERROR: Canonical guest unit directory not found. Expected ${project_unit_source_dir} or ${script_unit_source_dir}." >&2
    exit 1
fi
VM_NAME="${SUBSTRATE_LIMA_VM_NAME:-substrate}"
RUN_BREAKGLASS_CHECKS="${SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS:-0}"
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

usage() {
    cat <<'USAGE'
Usage: scripts/mac/lima-doctor.sh [options]

Options:
  --install-prefix <path>               Bind the helper to one host install prefix
  --install-bootstrap-context-v1 <arg>  Validate one committed install-bootstrap carrier
  -h, --help                            Show this help text
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --install-prefix)
            [[ $# -ge 2 ]] || { echo "ERROR: Missing value for --install-prefix" >&2; exit 1; }
            [[ "${INSTALL_PREFIX_DECLARED}" -eq 0 ]] || { echo "ERROR: Duplicate --install-prefix" >&2; exit 1; }
            [[ -n "$2" ]] || { echo "ERROR: Empty value for --install-prefix" >&2; exit 1; }
            INSTALL_PREFIX_RAW="$2"
            INSTALL_PREFIX_DECLARED=1
            shift 2
            ;;
        --install-bootstrap-context-v1)
            [[ $# -ge 2 ]] || { echo "ERROR: Missing value for --install-bootstrap-context-v1" >&2; exit 1; }
            [[ "${INSTALL_BOOTSTRAP_CONTEXT_DECLARED}" -eq 0 ]] || { echo "ERROR: Duplicate --install-bootstrap-context-v1" >&2; exit 1; }
            [[ -n "$2" ]] || { echo "ERROR: Empty value for --install-bootstrap-context-v1" >&2; exit 1; }
            INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
            INSTALL_BOOTSTRAP_CONTEXT_DECLARED=1
            shift 2
            ;;
        -h|--help)
            HELP_REQUESTED=1
            shift
            ;;
        *)
            echo "ERROR: Unknown argument: $1" >&2
            exit 1
            ;;
    esac
done

check() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m %s\n' "${name}"
    else
        printf '\033[31m[FAIL]\033[0m %s\n' "${name}"
        failures=$((failures+1))
    fi
}

check_with_failure_detail() {
    local name="$1"
    shift

    local stdout_file=""
    local stderr_file=""
    stdout_file="$(mktemp)"
    stderr_file="$(mktemp)"

    if "$@" >"${stdout_file}" 2>"${stderr_file}"; then
        printf '\033[32m[PASS]\033[0m %s\n' "${name}"
    else
        printf '\033[31m[FAIL]\033[0m %s\n' "${name}"
        failures=$((failures+1))
        if [[ -s "${stdout_file}" ]]; then
            sed 's/^/  /' "${stdout_file}"
        fi
        if [[ -s "${stderr_file}" ]]; then
            sed 's/^/  /' "${stderr_file}" >&2
        fi
    fi

    rm -f "${stdout_file}" "${stderr_file}"
}

warn() {
    printf '\033[33m[WARN]\033[0m %s\n' "$1"
}

resolve_install_bootstrap_context_v1() {
    local context_file
    local context_err
    local status

    context_file="$(mktemp)"
    context_err="$(mktemp)"
    command -v python3 >/dev/null 2>&1 || { echo "ERROR: python3 is required for authenticated install bootstrap context resolution." >&2; exit 1; }
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
            echo "ERROR: python3 is required for authenticated install bootstrap context resolution." >&2
            exit 1
        fi
        [[ ! -s "${context_err}" ]] || cat "${context_err}" >&2
        rm -f "${context_file}" "${context_err}"
        exit "${status}"
    fi
    rm -f "${context_err}"
    exec 3<"${context_file}"
    IFS= read -r -d '' INSTALL_CONTEXT_MODE <&3 || { echo "ERROR: Failed to read install bootstrap mode" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_PREFIX_RAW <&3 || { echo "ERROR: Failed to read install bootstrap prefix" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1 <&3 || { echo "ERROR: Failed to read install bootstrap carrier" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT <&3 || { echo "ERROR: Failed to read install bootstrap commitment" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT <&3 || { echo "ERROR: Failed to read install bootstrap account" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_BOOTSTRAP_UID <&3 || { echo "ERROR: Failed to read install bootstrap uid" >&2; exit 1; }
    IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT_HOME <&3 || { echo "ERROR: Failed to read install bootstrap account home" >&2; exit 1; }
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
    _mode = sys.argv[1]
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
    IFS= read -r -d '' HOST_ACCOUNT_HOME <&3 || { echo "ERROR: Failed to read host account home" >&2; exit 1; }
    IFS= read -r -d '' HOST_PLATFORM_CONTROL_ROOT <&3 || { echo "ERROR: Failed to read Lima control root" >&2; exit 1; }
    exec 3<&-
    rm -f "${control_file}"

    [[ "${HOST_ACCOUNT_HOME}" == "${INSTALL_BOOTSTRAP_ACCOUNT_HOME}" ]] \
        || { echo "ERROR: Account-database home changed while resolving the Lima control root." >&2; exit 1; }

    if [[ "${INSTALL_CONTEXT_MODE}" == "internal" ]]; then
        if [[ -n "${HOME:-}" && "${HOME}" != "${HOST_ACCOUNT_HOME}" ]]; then
            echo "ERROR: Internal Lima child HOME conflicts with the validated account-database home." >&2
            exit 1
        fi
        if [[ -n "${LIMA_HOME:-}" && "${LIMA_HOME}" != "${HOST_PLATFORM_CONTROL_ROOT}" ]]; then
            echo "ERROR: Internal Lima child LIMA_HOME conflicts with the validated Lima control root." >&2
            exit 1
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
        || { echo "ERROR: Unable to observe a stable Lima guest machine identity." >&2; exit 1; }

    account="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" id -un 2>/dev/null | tr -d '\r\n' || true)"
    uid="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" id -u 2>/dev/null | tr -d '\r\n' || true)"
    name_entry="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" getent passwd "${account}" 2>/dev/null | tr -d '\r' || true)"
    uid_entry="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" getent passwd "${uid}" 2>/dev/null | tr -d '\r' || true)"
    [[ -n "${account}" && -n "${uid}" && -n "${name_entry}" && "${name_entry}" == "${uid_entry}" ]] \
        || { echo "ERROR: Unable to round-trip the Lima guest account database identity." >&2; exit 1; }

    IFS=':' read -r entry_account _ entry_uid _ _ entry_home _ <<<"${name_entry}"
    [[ "${entry_account}" == "${account}" && "${entry_uid}" == "${uid}" && -n "${entry_home}" ]] \
        || { echo "ERROR: Lima guest identity does not round-trip through the account database." >&2; exit 1; }

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
    IFS= read -r -d '' OBSERVED_PLATFORM_MAPPING_V1 <&3 || { echo "ERROR: Failed to read platform mapping" >&2; exit 1; }
    IFS= read -r -d '' HOST_PLATFORM_CONTROL_ROOT <&3 || { echo "ERROR: Failed to normalize Lima control root" >&2; exit 1; }
    IFS= read -r -d '' OBSERVED_GUEST_SUBSTRATE_HOME <&3 || { echo "ERROR: Failed to normalize guest substrate home" >&2; exit 1; }
    IFS= read -r -d '' OBSERVED_TRANSPORT_HOST <&3 || { echo "ERROR: Failed to normalize host transport path" >&2; exit 1; }
    IFS= read -r -d '' OBSERVED_TRANSPORT_GUEST_SOCKET <&3 || { echo "ERROR: Failed to normalize guest transport path" >&2; exit 1; }
    exec 3<&-
    rm -f "${mapping_file}"
}

# shellcheck disable=SC2329
diagnose() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m %s\n' "${name}"
    else
        warn "${name}"
    fi
}

note_routed_override_bypass() {
    if [[ -n "${SUBSTRATE_WORLD_SOCKET:-}" ]]; then
        warn "Ignoring SUBSTRATE_WORLD_SOCKET during routed readiness proof; it remains advanced/test/breakglass on macOS."
    fi
}

# shellcheck disable=SC2329
run_routed_readiness_command() {
    env -u SUBSTRATE_WORLD_SOCKET "$@"
}

resolve_substrate_bin() {
    if [[ -n "${SUBSTRATE_BIN:-}" ]]; then
        printf '%s\n' "${SUBSTRATE_BIN}"
        return
    fi

    if [[ -x "${REPO_ROOT}/target/debug/substrate" ]]; then
        printf '%s\n' "${REPO_ROOT}/target/debug/substrate"
        return
    fi

    if command -v substrate >/dev/null 2>&1; then
        command -v substrate
        return
    fi

    printf '%s\n' substrate
}

# shellcheck disable=SC2329
substrate_cli_available() {
    if [[ "${SUBSTRATE_BIN}" == */* ]]; then
        test -x "${SUBSTRATE_BIN}"
    else
        command -v "${SUBSTRATE_BIN}" >/dev/null 2>&1
    fi
}

check_doctor_json() {
    local name="$1"
    local jq_filter="$2"
    shift 2

    local stdout_file=""
    local stderr_file=""
    stdout_file="$(mktemp)"
    stderr_file="$(mktemp)"

    if "$@" >"${stdout_file}" 2>"${stderr_file}"; then
        if jq -e "${jq_filter}" "${stdout_file}" >/dev/null 2>&1; then
            printf '\033[32m[PASS]\033[0m %s\n' "${name}"
        else
            printf '\033[31m[FAIL]\033[0m %s\n' "${name}"
            failures=$((failures+1))
            warn "Doctor output did not satisfy the routed-readiness contract for ${name}."
        fi
    else
        printf '\033[31m[FAIL]\033[0m %s\n' "${name}"
        failures=$((failures+1))
        if [[ -s "${stderr_file}" ]]; then
            sed 's/^/  /' "${stderr_file}" >&2
        fi
    fi

    rm -f "${stdout_file}" "${stderr_file}"
}

# shellcheck disable=SC2329
host_sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

# shellcheck disable=SC2329
check_rendered_unit_parity() {
    local parity_tmp=""
    local expected_dir=""
    local actual_dir=""
    local guest_substrate_home=""
    local layout_version=""
    local recovery_hint="run \`substrate world enable\` first; if you need the current helper-backed declared-instance Stage-1 create/start path directly, \`scripts/mac/lima-warm.sh\` remains degraded-but-supported. Further guest projection still requires a matching layout, and forwarding activation remains an R3 prerequisite."
    local enable_netfilter="${SUBSTRATE_WORLD_NETFILTER_ENABLE:-0}"
    local expected_netfilter_env=""
    local expected_service_sha=""
    local expected_socket_sha=""
    local actual_service_sha=""
    local actual_socket_sha=""
    local mismatch=0
    local status=""

    if [[ ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" || ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" ]]; then
        warn "Missing canonical unit sources under ${CANONICAL_UNIT_SOURCE_DIR}."
        return 1
    fi

    if ! command -v envsubst >/dev/null 2>&1; then
        warn "envsubst is required to render the canonical guest units locally."
        return 1
    fi

    if ! run_limactl_with_mapping_env_v1 list "${VM_NAME}" >/dev/null 2>&1; then
        warn "VM '${VM_NAME}' does not exist; ${recovery_hint}"
        return 1
    fi

    status="$(run_limactl_with_mapping_env_v1 list "${VM_NAME}" --json | jq -r --arg name "${VM_NAME}" 'if type == "array" then (map(select(.name == $name)) | if length == 0 then "__missing__" else .[0].status // "unknown" end) elif (.name // "") == $name then .status // "unknown" else "__missing__" end' 2>/dev/null || true)"
    if [[ "${status}" != "Running" ]]; then
        warn "VM '${VM_NAME}' is not running (status: ${status:-unknown}); ${recovery_hint}"
        return 1
    fi

    observe_lima_mapping_v1
    verify_lima_mapping_v1
    [[ -n "${OBSERVED_PLATFORM_MAPPING_V1}" ]] || { echo "ERROR: Verified Lima mapping record is empty." >&2; exit 1; }
    guest_substrate_home="${OBSERVED_GUEST_SUBSTRATE_HOME}"
    layout_version="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n cat /etc/substrate-lima-layout 2>/dev/null | tr -d '\r' || true)"
    if [[ "${layout_version}" != "${LAYOUT_EXPECTED}" ]]; then
        warn "Layout sentinel ${layout_version:-missing} (expected ${LAYOUT_EXPECTED}); R3 lifecycle reconciliation remains required before this doctor can treat the guest as ready."
        return 1
    fi

    parity_tmp="$(mktemp -d)"
    expected_dir="${parity_tmp}/expected"
    actual_dir="${parity_tmp}/actual"
    mkdir -p "${expected_dir}" "${actual_dir}"

    case "${enable_netfilter}" in
        1|true|yes|TRUE|YES)
            expected_netfilter_env="Environment=WORLD_NETFILTER_ENABLE=1"
            ;;
    esac

    SUBSTRATE_GUEST_HOME="${guest_substrate_home}" WORLD_NETFILTER_ENV="${expected_netfilter_env}" \
        envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" > "${expected_dir}/substrate-world-service.service"
    envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" > "${expected_dir}/substrate-world-service.socket"

    if ! run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.service \
        | sed '/^# \//d' \
        | awk 'BEGIN { seen=0 } { if (!seen && $0 == "") next; seen=1; print }' > "${actual_dir}/substrate-world-service.service"; then
        warn "Unable to capture the loaded guest service unit via systemctl cat."
        rm -rf "${parity_tmp}"
        return 1
    fi

    if ! run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.socket \
        | sed '/^# \//d' \
        | awk 'BEGIN { seen=0 } { if (!seen && $0 == "") next; seen=1; print }' > "${actual_dir}/substrate-world-service.socket"; then
        warn "Unable to capture the loaded guest socket unit via systemctl cat."
        rm -rf "${parity_tmp}"
        return 1
    fi

    expected_service_sha="$(host_sha256 "${expected_dir}/substrate-world-service.service")"
    expected_socket_sha="$(host_sha256 "${expected_dir}/substrate-world-service.socket")"
    actual_service_sha="$(host_sha256 "${actual_dir}/substrate-world-service.service")"
    actual_socket_sha="$(host_sha256 "${actual_dir}/substrate-world-service.socket")"

    if ! cmp -s "${expected_dir}/substrate-world-service.service" "${actual_dir}/substrate-world-service.service"; then
        warn "Guest service unit differs from the canonical rendered contract (expected sha256 ${expected_service_sha}, loaded guest sha256 ${actual_service_sha:-unknown})."
        mismatch=1
    fi

    if ! cmp -s "${expected_dir}/substrate-world-service.socket" "${actual_dir}/substrate-world-service.socket"; then
        warn "Guest socket unit differs from the canonical rendered contract (expected sha256 ${expected_socket_sha}, loaded guest sha256 ${actual_socket_sha:-unknown})."
        mismatch=1
    fi

    rm -rf "${parity_tmp}"
    return "${mismatch}"
}

run_breakglass_guest_checks() {
    echo "Guest-Direct Breakglass Diagnostics:"
    local lifecycle_hint="run \`substrate world enable\` first; if you need the current helper-backed declared-instance Stage-1 create/start path directly, \`scripts/mac/lima-warm.sh\` remains degraded-but-supported. Further guest projection still requires a matching layout."
    local status=""
    local breakglass_failures=0

    if ! run_limactl_with_mapping_env_v1 list "${VM_NAME}" >/dev/null 2>&1; then
        warn "VM '${VM_NAME}' does not exist; ${lifecycle_hint}"
        breakglass_failures=$((breakglass_failures + 1))
        if [[ "${RUN_BREAKGLASS_CHECKS}" == "1" ]]; then
            breakglass_summary_failures=$((breakglass_summary_failures + breakglass_failures))
            failures=$((failures + breakglass_failures))
        fi
        return
    fi

    status="$(run_limactl_with_mapping_env_v1 list "${VM_NAME}" --json | jq -r --arg name "${VM_NAME}" 'if type == "array" then (map(select(.name == $name)) | if length == 0 then "__missing__" else .[0].status // "unknown" end) elif (.name // "") == $name then .status // "unknown" else "__missing__" end' 2>/dev/null || true)"
    if [[ "${status}" != "Running" ]]; then
        warn "VM '${VM_NAME}' is not running (status: ${status:-unknown}); ${lifecycle_hint}"
        breakglass_failures=$((breakglass_failures + 1))
        if [[ "${RUN_BREAKGLASS_CHECKS}" == "1" ]]; then
            breakglass_summary_failures=$((breakglass_summary_failures + breakglass_failures))
            failures=$((failures + breakglass_failures))
        fi
        return
    fi

    observe_lima_mapping_v1
    verify_lima_mapping_v1
    [[ -n "${OBSERVED_PLATFORM_MAPPING_V1}" ]] || { echo "ERROR: Verified Lima mapping record is empty." >&2; exit 1; }

    local commitment_preview
    local machine_id_preview
    commitment_preview="${INSTALL_BOOTSTRAP_COMMITMENT:0:12}..."
    machine_id_preview="${OBSERVED_GUEST_MACHINE_ID:0:12}..."

    printf '\033[32m[PASS]\033[0m VM %q exists and is running\n' "${VM_NAME}"
    printf '\033[32m[PASS]\033[0m Mapping commitment %s\n' "${commitment_preview}"
    printf '\033[32m[PASS]\033[0m Lima control root resolved from account database.\n'
    printf '\033[32m[PASS]\033[0m Observed guest machine-id prefix %s\n' "${machine_id_preview}"
    printf '\033[32m[PASS]\033[0m Observed guest substrate home resolved from account database.\n'
    warn "Forwarding activation unavailable here: R3 prerequisite unmet."
    diagnose "SSH connectivity (breakglass)" run_limactl_with_mapping_env_v1 shell "${VM_NAME}" uname -a

    if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n test -S /run/substrate.sock >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m Agent socket exists (breakglass)\n'

        if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n timeout 5 curl --fail --unix-socket /run/substrate.sock http://localhost/v1/capabilities >/dev/null 2>&1; then
            printf '\033[32m[PASS]\033[0m Agent responds to direct capabilities probe (breakglass)\n'
        else
            warn "Agent direct capabilities probe failed (breakglass)."
            breakglass_failures=$((breakglass_failures + 1))
        fi

        local socket_meta=""
        socket_meta="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)"
        if [[ "${socket_meta}" == "root:substrate 660" ]]; then
            printf '\033[32m[PASS]\033[0m Socket ownership root:substrate (0660) (breakglass)\n'
        else
            warn "Socket ownership/mode does not match expected root:substrate 660. ${lifecycle_hint}"
            breakglass_failures=$((breakglass_failures + 1))
        fi

        if [[ -n "${OBSERVED_GUEST_ACCOUNT}" ]] && run_limactl_with_mapping_env_v1 shell "${VM_NAME}" id -nG "${OBSERVED_GUEST_ACCOUNT}" 2>/dev/null | tr ' ' '\n' | grep -qx substrate; then
            printf '\033[32m[PASS]\033[0m Guest account belongs to substrate group (breakglass)\n'
        else
            warn "Unable to confirm substrate group membership for the observed guest account. ${lifecycle_hint}"
            breakglass_failures=$((breakglass_failures + 1))
        fi

        local layout_version=""
        layout_version="$(run_limactl_with_mapping_env_v1 shell "${VM_NAME}" sudo -n cat /etc/substrate-lima-layout 2>/dev/null | tr -d '\r' || true)"
        if [[ "${layout_version}" == "${LAYOUT_EXPECTED}" ]]; then
            printf '\033[32m[PASS]\033[0m Socket parity layout detected (%s) (breakglass)\n' "${layout_version}"
        else
            warn "Layout sentinel ${layout_version:-missing} (expected ${LAYOUT_EXPECTED}). ${lifecycle_hint}"
            breakglass_failures=$((breakglass_failures + 1))
        fi
    else
        warn "Agent socket not found (breakglass)."
        breakglass_failures=$((breakglass_failures + 1))
    fi

    if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" systemctl is-active substrate-world-service >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m substrate-world-service service is active (breakglass)\n'
    else
        warn "substrate-world-service service is not active (breakglass)."
        breakglass_failures=$((breakglass_failures + 1))
    fi

    if run_limactl_with_mapping_env_v1 shell "${VM_NAME}" which nft >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m nftables available (breakglass)\n'
    else
        warn "nftables unavailable in guest (breakglass)."
    fi

    echo ""
    echo "Disk Usage (breakglass):"
    run_limactl_with_mapping_env_v1 shell "${VM_NAME}" bash -lc 'df -h / | tail -1' 2>/dev/null || warn "Could not get guest disk usage."
    if [[ "${RUN_BREAKGLASS_CHECKS}" == "1" && "${breakglass_failures}" -ne 0 ]]; then
        breakglass_summary_failures=$((breakglass_summary_failures + breakglass_failures))
        failures=$((failures + breakglass_failures))
    fi
}

if [[ "${HELP_REQUESTED}" -eq 1 ]]; then
    usage
    exit 0
fi

resolve_install_bootstrap_context_v1
resolve_lima_control_root_v1

if [[ -z "${SUBSTRATE_BIN:-}" && -x "${INSTALL_PREFIX_RAW}/bin/substrate" ]]; then
    SUBSTRATE_BIN="${INSTALL_PREFIX_RAW}/bin/substrate"
fi
SUBSTRATE_BIN="$(resolve_substrate_bin)"

echo "=== Substrate Lima Doctor ==="
echo ""
echo "Support posture:"
echo "  supported: substrate host doctor [--json]; substrate world doctor [--json]; substrate world gateway sync|status|restart; substrate world enable; substrate world deps current sync for dependency reconciliation"
echo "  degraded-but-supported: scripts/mac/lima-doctor.sh validates routed readiness plus authenticated mapping/loaded-unit parity, and only escalates to additional guest-direct breakglass diagnostics after failure or explicit opt-in; scripts/mac/lima-warm.sh retains create/warm/repair plus staged-workspace copy"
echo "  breakglass: raw limactl shell, plain SSH, direct guest systemctl, guest socket curl, guest journalctl, and host-side SUBSTRATE_WORLD_SOCKET override use"
echo "  note: substrate workspace sync is not the frozen normal macOS sync/copy path in this packet"
echo ""
echo "Mapping posture:"
echo "  selected commitment: ${INSTALL_BOOTSTRAP_COMMITMENT:0:12}..."
echo "  declared VM: ${VM_NAME}"
echo "  Lima control root: resolved from account database."
echo "  forwarding activation: R3 prerequisite unmet"
echo ""

echo "Host Environment:"
check "Lima installed" command -v limactl
check "jq installed" command -v jq
check "envsubst installed" command -v envsubst
check "Substrate CLI available" substrate_cli_available
check "Virtualization available" test "$(sysctl -n kern.hv_support 2>/dev/null)" -eq 1

if command -v vsock-proxy >/dev/null 2>&1; then
    printf '\033[32m[PASS]\033[0m vsock-proxy available\n'
else
    warn "vsock-proxy not found (SSH forwarding fallback remains available)."
fi

echo ""
echo "Routed Readiness Snapshot (bounded M1 prerequisites only):"
restore_forwarder_port=0
saved_forwarder_port=""
if [[ "${SUBSTRATE_FORWARDER_PORT+x}" == x ]]; then
    restore_forwarder_port=1
    saved_forwarder_port="${SUBSTRATE_FORWARDER_PORT}"
    unset SUBSTRATE_FORWARDER_PORT
fi
note_routed_override_bypass
check_doctor_json "substrate host doctor --json" '.ok == true and .host.ok == true' run_routed_readiness_command "${SUBSTRATE_BIN}" host doctor --json
check_doctor_json "substrate world doctor --json" '.ok == true and .host.ok == true and .world.ok == true and .world.status == "ok"' run_routed_readiness_command "${SUBSTRATE_BIN}" world doctor --json
if [[ "${restore_forwarder_port}" -eq 1 ]]; then
    export SUBSTRATE_FORWARDER_PORT="${saved_forwarder_port}"
fi
check_with_failure_detail "Canonical rendered guest units match the loaded service/socket contract" check_rendered_unit_parity

echo ""
if [[ "${RUN_BREAKGLASS_CHECKS}" == "1" ]]; then
    echo "  Running because SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS=1 explicitly requested guest-direct breakglass evidence."
    run_breakglass_guest_checks
elif [[ "${failures}" -ne 0 ]]; then
    echo "  Routed readiness failed above; escalating to guest-direct breakglass diagnostics."
    run_breakglass_guest_checks
else
    echo "Guest-Direct Breakglass Diagnostics:"
    echo "  Skipped because routed readiness is healthy."
    echo "  Set SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS=1 to run guest-direct diagnostics explicitly."
fi

echo ""
if [ "${failures}" -ne 0 ]; then
    if [[ "${breakglass_summary_failures}" -ne 0 ]]; then
        routed_failures=$((failures - breakglass_summary_failures))
        if [[ "${routed_failures}" -eq 0 ]]; then
            echo "Doctor detected ${breakglass_summary_failures} guest-direct breakglass issue(s). See above output for details." >&2
        else
            echo "Doctor detected ${failures} issue(s) across routed readiness and guest-direct breakglass checks. See above output for details." >&2
        fi
    else
        echo "Doctor detected ${failures} routed-readiness issue(s). See above output for details." >&2
    fi
    exit 1
else
    echo "Bounded routed readiness prerequisites passed. Forwarding activation remains an R3 prerequisite."
    exit 0
fi
