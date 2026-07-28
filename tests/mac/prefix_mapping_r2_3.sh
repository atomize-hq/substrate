#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="prefix-mapping-r2-3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
LIMA_WARM="${REPO_ROOT}/scripts/mac/lima-warm.sh"
LIMA_DOCTOR="${REPO_ROOT}/scripts/mac/lima-doctor.sh"
CURRENT_UID="$(id -u)"
CURRENT_ACCOUNT="$(id -un)"
CURRENT_HOME="$(python3 - <<'PY'
import os
import pwd

uid = os.geteuid()
print(pwd.getpwuid(uid).pw_dir)
PY
)"

fail() {
    printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
    exit 1
}

info() {
    printf '[%s] %s\n' "${SCRIPT_NAME}" "$1" >&2
}

if [[ "${CURRENT_UID}" -eq 0 ]]; then
    fail "run this fixture as an unprivileged Unix principal"
fi

WORK_ROOT="$(mktemp -d "/tmp/substrate-${SCRIPT_NAME}.XXXXXX")"
KEEP_ROOT="${KEEP_PREFIX_MAPPING_R2_3_FIXTURE:-0}"
STUB_BIN="${WORK_ROOT}/bin"
STATE_ROOT="${WORK_ROOT}/state"
STDOUT_PATH="${WORK_ROOT}/stdout"
STDERR_PATH="${WORK_ROOT}/stderr"
SELECTED_A="${WORK_ROOT}/selected-a"
CUSTOM_A="${WORK_ROOT}/custom-a"
AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir -p "${STUB_BIN}" "${STATE_ROOT}" "${SELECTED_A}" "${CUSTOM_A}" "${AMBIENT_B}"
chmod 0700 "${WORK_ROOT}" "${STUB_BIN}" "${STATE_ROOT}" "${SELECTED_A}" "${CUSTOM_A}" "${AMBIENT_B}"
printf 'ambient-b-sentinel\n' > "${AMBIENT_B}/sentinel"

cleanup() {
    if [[ "${KEEP_ROOT}" -eq 1 ]]; then
        info "preserving ${WORK_ROOT}"
    else
        rm -rf "${WORK_ROOT}"
    fi
}
trap cleanup EXIT

HOST_PATH="${PATH:-}"
HOST_PYTHON3="$(command -v python3)"
RUN_STATUS=0
RUN_LOG=""
RUN_STATE_DIR=""

make_context_values() {
    local prefix="$1"
    local account="$2"
    local uid="$3"
    python3 - "${prefix}" "${account}" "${uid}" <<'PY'
import base64
import hashlib
import sys


def encode(value):
    return base64.urlsafe_b64encode(value.encode("utf-8")).rstrip(b"=").decode("ascii")


prefix, account, uid = sys.argv[1:]
prefix_encoded = encode(prefix)
frame = (
    "domain=substrate.install_bootstrap_context\n"
    "version=1\n"
    f"selected_host_prefix={prefix_encoded}\n"
    f"host_substrate_home={prefix_encoded}\n"
    f"host_substrate_root={prefix_encoded}\n"
    "principal_kind=unix\n"
    f"principal_account={encode(account)}\n"
    f"principal_uid={uid}\n"
).encode("ascii")
commitment = hashlib.sha256(frame).hexdigest()
record = frame + f"host_context_commitment={commitment}\n".encode("ascii")
print(base64.urlsafe_b64encode(record).rstrip(b"=").decode("ascii"))
print(commitment)
PY
}

tamper_commitment() {
    python3 - "$1" <<'PY'
import base64
import sys


carrier = sys.argv[1].encode("ascii")
record = base64.urlsafe_b64decode(carrier + b"=" * ((-len(carrier)) % 4))
marker = b"host_context_commitment="
before, commitment = record.split(marker, 1)
replacement = b"0" if commitment[:1] != b"0" else b"1"
print(base64.urlsafe_b64encode(before + marker + replacement + commitment[1:]).rstrip(b"=").decode("ascii"))
PY
}

reorder_carrier() {
    python3 - "$1" <<'PY'
import base64
import sys


carrier = sys.argv[1].encode("ascii")
record = base64.urlsafe_b64decode(carrier + b"=" * ((-len(carrier)) % 4))
lines = record.rstrip(b"\n").split(b"\n")
lines[2], lines[3] = lines[3], lines[2]
print(base64.urlsafe_b64encode(b"\n".join(lines) + b"\n").rstrip(b"=").decode("ascii"))
PY
}

A_CONTEXT_OUTPUT="$(make_context_values "${SELECTED_A}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")"
A_CARRIER="$(printf '%s\n' "${A_CONTEXT_OUTPUT}" | sed -n '1p')"
A_COMMITMENT="$(printf '%s\n' "${A_CONTEXT_OUTPUT}" | sed -n '2p')"
CUSTOM_CONTEXT_OUTPUT="$(make_context_values "${CUSTOM_A}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")"
CUSTOM_COMMITMENT="$(printf '%s\n' "${CUSTOM_CONTEXT_OUTPUT}" | sed -n '2p')"
TAMPERED_CARRIER="$(tamper_commitment "${A_CARRIER}")"
REORDERED_CARRIER="$(reorder_carrier "${A_CARRIER}")"

write_stub() {
    local name="$1"
    cat > "${STUB_BIN}/${name}"
    chmod +x "${STUB_BIN}/${name}"
}

write_stub_uname() {
    write_stub uname <<'STUB'
#!/usr/bin/env bash
if [[ $# -eq 0 || "$1" == "-s" ]]; then
    printf 'Darwin\n'
    exit 0
fi
if [[ "$1" == "-m" ]]; then
    printf 'arm64\n'
    exit 0
fi
/usr/bin/uname "$@"
STUB
}

write_stub_sysctl() {
    write_stub sysctl <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "-n" && "${2:-}" == "kern.hv_support" ]]; then
    printf '1\n'
    exit 0
fi
printf '0\n'
STUB
}

write_stub_python3() {
    cat > "${STUB_BIN}/python3" <<STUB
#!/usr/bin/env bash
if [[ "\${SUBSTRATE_TEST_PYTHON_MODE:-present}" == "missing" ]]; then
    printf 'python3: command not found\n' >&2
    exit 127
fi
exec "${HOST_PYTHON3}" "\$@"
STUB
    chmod +x "${STUB_BIN}/python3"
}

write_stub_envsubst() {
    write_stub envsubst <<'STUB'
#!/usr/bin/env python3
import os
import re
import sys

content = sys.stdin.read()
pattern = re.compile(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)")

def replace(match):
    key = match.group(1) or match.group(2)
    return os.environ.get(key, "")

sys.stdout.write(pattern.sub(replace, content))
STUB
}

write_stub_jq() {
    write_stub jq <<'STUB'
#!/usr/bin/env python3
import json
import sys

args = sys.argv[1:]
raw = False
exit_mode = False
expr = None
path = None
jq_args = {}
while args:
    token = args.pop(0)
    if token == "-r":
        raw = True
    elif token == "-e":
        exit_mode = True
    elif token == "--arg":
        key = args.pop(0)
        jq_args[key] = args.pop(0)
    elif expr is None:
        expr = token
    else:
        path = token
if expr is None:
    sys.exit(2)
if path:
    with open(path, "r", encoding="utf-8") as handle:
        payload = handle.read()
else:
    payload = sys.stdin.read()
try:
    data = json.loads(payload)
except Exception:
    data = {}

def world_ok(obj):
    return (
        obj.get("ok") is True
        and isinstance(obj.get("host"), dict)
        and obj["host"].get("ok") is True
        and isinstance(obj.get("world"), dict)
        and obj["world"].get("ok") is True
        and obj["world"].get("status") == "ok"
    )

def host_ok(obj):
    return obj.get("ok") is True and isinstance(obj.get("host"), dict) and obj["host"].get("ok") is True

if expr == '.status // "unknown"':
    print(data.get("status") or "unknown")
    sys.exit(0)
if 'map(select(.name == $name))' in expr:
    name = jq_args.get("name")
    if isinstance(data, list):
        matches = [item for item in data if isinstance(item, dict) and item.get("name") == name]
        print((matches[0].get("status") or "unknown") if matches else "__missing__")
    elif isinstance(data, dict) and data.get("name") == name:
        print(data.get("status") or "unknown")
    else:
        print("__missing__")
    sys.exit(0)
if expr == '.ok == true and .host.ok == true':
    ok = host_ok(data)
    if not ok and exit_mode:
        sys.exit(1)
    print("true" if ok else "false")
    sys.exit(0 if ok or not exit_mode else 1)
if expr == '.ok == true and .host.ok == true and .world.ok == true and .world.status == "ok"':
    ok = world_ok(data)
    if not ok and exit_mode:
        sys.exit(1)
    print("true" if ok else "false")
    sys.exit(0 if ok or not exit_mode else 1)
print("unknown")
sys.exit(0)
STUB
}

write_stub_file() {
    write_stub file <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${1:-}" == "-b" ]]; then
    shift
fi
printf 'ELF 64-bit LSB executable\n'
STUB
}

write_stub_vsock_proxy() {
    write_stub vsock-proxy <<'STUB'
#!/usr/bin/env bash
exit 0
STUB
}

write_stub_substrate() {
    write_stub substrate <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
scenario="${SUBSTRATE_TEST_SUBSTRATE_SCENARIO:-healthy}"
if [[ "${SUBSTRATE_TEST_REQUIRE_PREFIX_BIN:-0}" == "1" ]]; then
    printf 'unexpected PATH substrate resolution\n' >&2
    exit 97
fi
if [[ -n "${SUBSTRATE_FORWARDER_PORT:-}" || -n "${SUBSTRATE_WORLD_SOCKET:-}" ]]; then
    printf 'unexpected ambient forwarding override: port=%s socket=%s\n' "${SUBSTRATE_FORWARDER_PORT:-}" "${SUBSTRATE_WORLD_SOCKET:-}" >&2
    exit 98
fi
if [[ "${1:-}" == "host" && "${2:-}" == "doctor" && "${3:-}" == "--json" ]]; then
    printf '{"ok":true,"host":{"ok":true}}\n'
    exit 0
fi
if [[ "${1:-}" == "world" && "${2:-}" == "doctor" && "${3:-}" == "--json" ]]; then
    if [[ "${scenario}" == "world_fail" ]]; then
        printf '{"ok":false,"host":{"ok":true},"world":{"ok":false,"status":"degraded"}}\n'
    else
        printf '{"ok":true,"host":{"ok":true},"world":{"ok":true,"status":"ok"}}\n'
    fi
    exit 0
fi
printf 'unexpected substrate stub invocation: %s\n' "$*" >&2
exit 99
STUB
}

write_prefix_bound_substrate_bins() {
    local target=""
    for target in "${SELECTED_A}/bin/substrate" "${CUSTOM_A}/bin/substrate"; do
        mkdir -p "$(dirname "${target}")"
        cat > "${target}" <<'STUB'
#!/usr/bin/env bash
set -euo pipefail
scenario="${SUBSTRATE_TEST_SUBSTRATE_SCENARIO:-healthy}"
if [[ -n "${SUBSTRATE_FORWARDER_PORT:-}" || -n "${SUBSTRATE_WORLD_SOCKET:-}" ]]; then
    printf 'unexpected ambient forwarding override: port=%s socket=%s\n' "${SUBSTRATE_FORWARDER_PORT:-}" "${SUBSTRATE_WORLD_SOCKET:-}" >&2
    exit 98
fi
if [[ "${1:-}" == "host" && "${2:-}" == "doctor" && "${3:-}" == "--json" ]]; then
    printf '{"ok":true,"host":{"ok":true}}\n'
    exit 0
fi
if [[ "${1:-}" == "world" && "${2:-}" == "doctor" && "${3:-}" == "--json" ]]; then
    if [[ "${scenario}" == "world_fail" ]]; then
        printf '{"ok":false,"host":{"ok":true},"world":{"ok":false,"status":"degraded"}}\n'
    else
        printf '{"ok":true,"host":{"ok":true},"world":{"ok":true,"status":"ok"}}\n'
    fi
    exit 0
fi
printf 'unexpected prefix-bound substrate invocation: %s\n' "$*" >&2
exit 99
STUB
        chmod +x "${target}"
    done
}

write_stub_limactl() {
    write_stub limactl <<'STUB'
#!/usr/bin/env bash
set -euo pipefail

log_path="${SUBSTRATE_TEST_LOG:?}"
state_dir="${SUBSTRATE_TEST_STATE_DIR:?}"
scenario="${SUBSTRATE_TEST_SCENARIO:?}"
expected_vm="${SUBSTRATE_TEST_VM_NAME:-substrate}"
guest_account="${SUBSTRATE_TEST_GUEST_ACCOUNT:-guest}"
guest_uid="${SUBSTRATE_TEST_GUEST_UID:-1000}"
guest_home="${SUBSTRATE_TEST_GUEST_HOME:-/Users/guest}"
guest_substrate_home="${guest_home%/}/.substrate"
machine_id="${SUBSTRATE_TEST_MACHINE_ID:-11111111111111111111111111111111}"
alternate_machine_id="${SUBSTRATE_TEST_MACHINE_ID_2:-22222222222222222222222222222222}"
layout_value="${SUBSTRATE_TEST_LAYOUT:-socket-parity-v2-staged-workspace-v1}"
unit_dir="${SUBSTRATE_TEST_UNIT_DIR:?}"

record() {
    printf 'HOME=%s\tLIMA_HOME=%s\t%s\n' "${HOME:-}" "${LIMA_HOME:-}" "$*" >> "${log_path}"
}

count_file() {
    printf '%s/%s.count' "${state_dir}" "$1"
}

next_count() {
    local key="$1"
    local file
    local value=0
    file="$(count_file "${key}")"
    if [[ -f "${file}" ]]; then
        value="$(<"${file}")"
    fi
    value=$((value + 1))
    printf '%s\n' "${value}" > "${file}"
    printf '%s\n' "${value}"
}

mark() {
    : > "${state_dir}/$1"
}

marked() {
    [[ -f "${state_dir}/$1" ]]
}

render_service() {
    local expected_commitment="${SUBSTRATE_TEST_EXPECTED_COMMITMENT:-}"
    local expected_control_root="${SUBSTRATE_TEST_EXPECTED_CONTROL_ROOT:-}"
    local expected_prefix="${SUBSTRATE_TEST_EXPECTED_PREFIX:-}"
    local expected_host_socket="${SUBSTRATE_TEST_EXPECTED_HOST_SOCKET:-${expected_prefix%/}/sock/agent.sock}"
    local expected_guest_socket="${SUBSTRATE_TEST_EXPECTED_GUEST_SOCKET:-/run/substrate.sock}"
    python3 - "${unit_dir}" "${guest_substrate_home}" "${expected_vm}" "${expected_commitment}" "${expected_control_root}" "${expected_host_socket}" "${expected_guest_socket}" <<'PY'
import os
import pathlib
import sys

unit_dir = pathlib.Path(sys.argv[1])
guest_home = sys.argv[2]
vm_name = sys.argv[3]
commitment = sys.argv[4]
control_root = sys.argv[5]
host_socket = sys.argv[6]
guest_socket = sys.argv[7]
template = (unit_dir / "substrate-world-service.service.tmpl").read_text(encoding="utf-8")
socket_unit = (unit_dir / "substrate-world-service.socket").read_text(encoding="utf-8")
substitutions = {
    "${SUBSTRATE_GUEST_HOME}": guest_home,
    "$SUBSTRATE_GUEST_HOME": guest_home,
    "${WORLD_NETFILTER_ENV}": "",
    "$WORLD_NETFILTER_ENV": "",
    "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT}": commitment,
    "$SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT": commitment,
    "${SUBSTRATE_LIMA_INSTANCE_NAME}": vm_name,
    "$SUBSTRATE_LIMA_INSTANCE_NAME": vm_name,
    "${SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT}": control_root,
    "$SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT": control_root,
    "${SUBSTRATE_LIMA_HOST_SOCKET}": host_socket,
    "$SUBSTRATE_LIMA_HOST_SOCKET": host_socket,
    "${SUBSTRATE_LIMA_GUEST_SOCKET}": guest_socket,
    "$SUBSTRATE_LIMA_GUEST_SOCKET": guest_socket,
}
for needle, value in substitutions.items():
    template = template.replace(needle, value)
if sys.argv[0] == "service":
    sys.stdout.write(template)
else:
    sys.stdout.write(socket_unit)
PY
}

render_unit() {
    local kind="$1"
    local expected_commitment="${SUBSTRATE_TEST_EXPECTED_COMMITMENT:-}"
    local expected_control_root="${SUBSTRATE_TEST_EXPECTED_CONTROL_ROOT:-}"
    local expected_prefix="${SUBSTRATE_TEST_EXPECTED_PREFIX:-}"
    local expected_host_socket="${SUBSTRATE_TEST_EXPECTED_HOST_SOCKET:-${expected_prefix%/}/sock/agent.sock}"
    local expected_guest_socket="${SUBSTRATE_TEST_EXPECTED_GUEST_SOCKET:-/run/substrate.sock}"
    python3 - "${unit_dir}" "${guest_substrate_home}" "${kind}" "${expected_vm}" "${expected_commitment}" "${expected_control_root}" "${expected_host_socket}" "${expected_guest_socket}" <<'PY'
import os
import pathlib
import sys

unit_dir = pathlib.Path(sys.argv[1])
guest_home = sys.argv[2]
kind = sys.argv[3]
vm_name = sys.argv[4]
commitment = sys.argv[5]
control_root = sys.argv[6]
host_socket = sys.argv[7]
guest_socket = sys.argv[8]
if kind == "service":
    template = (unit_dir / "substrate-world-service.service.tmpl").read_text(encoding="utf-8")
    substitutions = {
        "${SUBSTRATE_GUEST_HOME}": guest_home,
        "$SUBSTRATE_GUEST_HOME": guest_home,
        "${WORLD_NETFILTER_ENV}": "",
        "$WORLD_NETFILTER_ENV": "",
        "${SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT}": commitment,
        "$SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT": commitment,
        "${SUBSTRATE_LIMA_INSTANCE_NAME}": vm_name,
        "$SUBSTRATE_LIMA_INSTANCE_NAME": vm_name,
        "${SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT}": control_root,
        "$SUBSTRATE_LIMA_HOST_PLATFORM_CONTROL_ROOT": control_root,
        "${SUBSTRATE_LIMA_HOST_SOCKET}": host_socket,
        "$SUBSTRATE_LIMA_HOST_SOCKET": host_socket,
        "${SUBSTRATE_LIMA_GUEST_SOCKET}": guest_socket,
        "$SUBSTRATE_LIMA_GUEST_SOCKET": guest_socket,
    }
    for needle, value in substitutions.items():
        template = template.replace(needle, value)
    sys.stdout.write(template)
else:
    sys.stdout.write((unit_dir / "substrate-world-service.socket").read_text(encoding="utf-8"))
PY
}

if [[ $# -lt 1 ]]; then
    exit 1
fi

cmd="$1"
shift
record "${cmd} $*"

case "${cmd}" in
    list)
        json=0
        vm="${1:-}"
        if [[ "${vm}" == "--json" ]]; then
            json=1
            vm=""
        else
            shift || true
            if [[ "${1:-}" == "--json" ]]; then
                json=1
            fi
        fi
        if [[ -n "${vm}" && "${vm}" != "${expected_vm}" ]]; then
            exit 1
        fi
        emit_list_status() {
            local status="$1"
            if [[ "${json}" -eq 1 ]]; then
                printf '[{"name":"%s","status":"%s"}]\n' "${expected_vm}" "${status}"
            else
                printf '%s\t%s\n' "${expected_vm}" "${status}"
            fi
        }
        emit_missing() {
            if [[ "${json}" -eq 1 ]]; then
                printf '[]\n'
            else
                exit 1
            fi
        }
        case "${scenario}" in
            warm_create_layout_mismatch)
                if ! marked created; then
                    emit_missing
                    exit 0
                fi
                emit_list_status "Running"
                ;;
            warm_stopped_layout_mismatch)
                if marked started; then
                    emit_list_status "Running"
                else
                    emit_list_status "Stopped"
                fi
                ;;
            warm_broken_status)
                if marked started; then
                    emit_list_status "Broken"
                else
                    emit_list_status "Stopped"
                fi
                ;;
            vm_missing)
                emit_missing
                ;;
            vm_stopped)
                emit_list_status "Stopped"
                ;;
            *)
                emit_list_status "Running"
                ;;
        esac
        exit 0
        ;;
    start)
        if [[ "${1:-}" == "--tty=false" ]]; then
            mark created
            exit 0
        fi
        mark started
        exit 0
        ;;
    stop|delete|copy)
        printf 'unexpected lifecycle action under scenario %s: %s %s\n' "${scenario}" "${cmd}" "$*" >&2
        exit 97
        ;;
    shell)
        vm="${1:-}"
        shift || true
        if [[ "${vm}" != "${expected_vm}" ]]; then
            exit 1
        fi
        case "${1:-}" in
            cat)
                if [[ "${2:-}" == "/etc/machine-id" ]]; then
                    call="$(next_count machine)"
                    if [[ "${scenario}" == "machine_id_changes" && "${call}" -gt 1 ]]; then
                        printf '%s\n' "${alternate_machine_id}"
                    else
                        printf '%s\n' "${machine_id}"
                    fi
                    exit 0
                fi
                ;;
            id)
                if [[ "${2:-}" == "-un" ]]; then
                    printf '%s\n' "${guest_account}"
                    exit 0
                fi
                if [[ "${2:-}" == "-u" ]]; then
                    printf '%s\n' "${guest_uid}"
                    exit 0
                fi
                if [[ "${2:-}" == "-nG" ]]; then
                    printf 'wheel substrate\n'
                    exit 0
                fi
                ;;
            getent)
                if [[ "${2:-}" == "passwd" ]]; then
                    query="${3:-}"
                    case "${scenario}" in
                        missing_getent_home)
                            printf '%s:x:%s:20:::/bin/bash\n' "${guest_account}" "${guest_uid}"
                            exit 0
                            ;;
                        mismatched_principal_home)
                            if [[ "${query}" == "${guest_account}" ]]; then
                                printf '%s:x:%s:20::%s:/bin/bash\n' "${guest_account}" "${guest_uid}" "${guest_home}"
                            else
                                printf 'other:x:%s:20::/srv/other:/bin/bash\n' "${guest_uid}"
                            fi
                            exit 0
                            ;;
                        *)
                            printf '%s:x:%s:20::%s:/bin/bash\n' "${guest_account}" "${guest_uid}" "${guest_home}"
                            exit 0
                            ;;
                    esac
                fi
                ;;
            sudo)
                if [[ "${2:-}" == "-n" ]]; then
                    shift 2
                else
                    shift
                fi
                case "${1:-}" in
                    test)
                        if [[ "${2:-}" == "-S" && "${3:-}" == "/run/substrate.sock" ]]; then
                            if [[ "${scenario}" == "socket_missing" ]]; then
                                exit 1
                            fi
                            exit 0
                        fi
                        if [[ "${2:-}" == "-f" && "${3:-}" == "/etc/systemd/system/substrate-world-service.service" ]]; then
                            exit 0
                        fi
                        if [[ "${2:-}" == "-x" && "${3:-}" == "/usr/local/bin/substrate-gateway" ]]; then
                            exit 1
                        fi
                        ;;
                    ls)
                        printf 'srw-rw---- 1 root substrate 0 Jul 27 00:00 /run/substrate.sock\n'
                        exit 0
                        ;;
                    grep)
                        exit 1
                        ;;
                    systemctl)
                        if [[ "${2:-}" == "cat" && "${3:-}" == "substrate-world-service.service" ]]; then
                            render_unit service
                            exit 0
                        fi
                        if [[ "${2:-}" == "cat" && "${3:-}" == "substrate-world-service.socket" ]]; then
                            render_unit socket
                            exit 0
                        fi
                        ;;
                    timeout)
                        if [[ "${scenario}" == "caps_fail" ]]; then
                            exit 124
                        fi
                        printf '{}\n'
                        exit 0
                        ;;
                    stat)
                        printf 'root:substrate 660\n'
                        exit 0
                        ;;
                    cat)
                        if [[ "${2:-}" == "/etc/substrate-lima-layout" ]]; then
                            printf '%s\n' "${layout_value}"
                            exit 0
                        fi
                        ;;
                esac
                ;;
            systemctl)
                if [[ "${2:-}" == "is-active" && "${3:-}" == "substrate-world-service" ]]; then
                    if [[ "${scenario}" == "service_down" ]]; then
                        exit 3
                    fi
                    printf 'active\n'
                    exit 0
                fi
                ;;
            which)
                if [[ "${2:-}" == "nft" || "${1:-}" == "which" && "${2:-}" == "nft" ]]; then
                    printf '/usr/sbin/nft\n'
                    exit 0
                fi
                ;;
            uname)
                printf 'Linux substrate\n'
                exit 0
                ;;
            bash)
                if [[ "${scenario}" == warm_* ]]; then
                    printf 'unexpected guest mutation shell under scenario %s: %s %s\n' "${scenario}" "${vm}" "$*" >&2
                    exit 97
                fi
                if [[ "${2:-}" == "df -h / | tail -1" || "${1:-}" == "bash" ]]; then
                    printf '/dev/root 10G 5G 5G 50%% /\n'
                    exit 0
                fi
                ;;
        esac
        printf 'unhandled limactl shell invocation: %s %s\n' "${vm}" "$*" >&2
        exit 98
        ;;
esac

printf 'unhandled limactl invocation: %s %s\n' "${cmd}" "$*" >&2
exit 98
STUB
}

setup_stubs() {
    PATH="${STUB_BIN}:${HOST_PATH}"
    export PATH
    write_stub_uname
    write_stub_sysctl
    write_stub_python3
    write_stub_envsubst
    write_stub_jq
    write_stub_file
    write_stub_vsock_proxy
    write_stub_substrate
    write_prefix_bound_substrate_bins
    write_stub_limactl
}

setup_stubs

assert_contains() {
    local needle="$1"
    local file="$2"
    local label="$3"
    grep -Fq -- "${needle}" "${file}" || fail "${label}: missing '${needle}'"
}

assert_not_contains() {
    local needle="$1"
    local file="$2"
    local label="$3"
    if grep -Fq -- "${needle}" "${file}"; then
        fail "${label}: unexpectedly found '${needle}'"
    fi
}

assert_status() {
    local expected="$1"
    local actual="$2"
    local label="$3"
    [[ "${actual}" -eq "${expected}" ]] || fail "${label}: exit ${actual}, expected ${expected}"
}

assert_log_scrubbed() {
    local log_path="$1"
    local label="$2"
    local expected_lima="${CURRENT_HOME%/}/.lima"
    if grep -Fq -- "HOME=${AMBIENT_B}" "${log_path}" || grep -Fq -- "LIMA_HOME=${AMBIENT_B}" "${log_path}"; then
        fail "${label}: ambient B reached a Lima child"
    fi
    awk -F'\t' -v home="HOME=${CURRENT_HOME}" -v lima="LIMA_HOME=${expected_lima}" '
        NF {
            if ($1 != home || $2 != lima) {
                exit 1
            }
        }
    ' "${log_path}" || fail "${label}: child HOME/LIMA_HOME did not match the account-database projection"
}

assert_output_scrubbed() {
    local label="$1"
    local file=""
    local default_prefix="${CURRENT_HOME%/}/.substrate"
    local expected_lima="${CURRENT_HOME%/}/.lima"
    for file in "${STDOUT_PATH}" "${STDERR_PATH}"; do
        assert_not_contains "${A_CARRIER}" "${file}" "${label}"
        assert_not_contains "${A_COMMITMENT}" "${file}" "${label}"
        assert_not_contains "${TAMPERED_CARRIER}" "${file}" "${label}"
        assert_not_contains "${REORDERED_CARRIER}" "${file}" "${label}"
        assert_not_contains "${SELECTED_A}" "${file}" "${label}"
        assert_not_contains "${CUSTOM_A}" "${file}" "${label}"
        assert_not_contains "${default_prefix}" "${file}" "${label}"
        assert_not_contains "${expected_lima}" "${file}" "${label}"
    done
}

assert_no_lifecycle() {
    local log_path="$1"
    local label="$2"
    if grep -Eq -- $'\t(start|stop|delete|copy) ' "${log_path}"; then
        fail "${label}: lifecycle mutation reached limactl"
    fi
}

assert_no_limactl_calls() {
    local log_path="$1"
    local label="$2"
    [[ ! -s "${log_path}" ]] || fail "${label}: limactl ran before validation should have stopped"
}

line_no() {
    local pattern="$1"
    local file="$2"
    grep -n -E -- "${pattern}" "${file}" | head -n1 | cut -d: -f1
}

assert_order() {
    local file="$1"
    local label="$2"
    shift 2
    local previous=0
    local pattern=""
    for pattern in "$@"; do
        local current
        current="$(line_no "${pattern}" "${file}")"
        [[ -n "${current}" ]] || fail "${label}: missing '${pattern}'"
        if (( current <= previous )); then
            fail "${label}: '${pattern}' appeared out of order"
        fi
        previous="${current}"
    done
}

run_case() {
    local label="$1"
    shift
    RUN_LOG="${WORK_ROOT}/${label}.log"
    RUN_STATE_DIR="${STATE_ROOT}/${label}"
    mkdir -p "${RUN_STATE_DIR}"
    : > "${RUN_LOG}"
    : > "${STDOUT_PATH}"
    : > "${STDERR_PATH}"
    set +e
    "$@" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
    RUN_STATUS=$?
    set -e
    assert_output_scrubbed "${label}"
}

run_with_env() {
    local label="$1"
    shift
    RUN_LOG="${WORK_ROOT}/${label}.log"
    RUN_STATE_DIR="${STATE_ROOT}/${label}"
    mkdir -p "${RUN_STATE_DIR}"
    : > "${RUN_LOG}"
    : > "${STDOUT_PATH}"
    : > "${STDERR_PATH}"
    set +e
    env \
        PATH="${PATH}" \
        SUBSTRATE_TEST_LOG="${RUN_LOG}" \
        SUBSTRATE_TEST_STATE_DIR="${RUN_STATE_DIR}" \
        SUBSTRATE_TEST_UNIT_DIR="${REPO_ROOT}/scripts/mac/lima/units" \
        SUBSTRATE_TEST_EXPECTED_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_TEST_EXPECTED_CONTROL_ROOT="${CURRENT_HOME%/}/.lima" \
        SUBSTRATE_TEST_EXPECTED_PREFIX="${SELECTED_A}" \
        SUBSTRATE_TEST_EXPECTED_GUEST_SOCKET="/run/substrate.sock" \
        "$@" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
    RUN_STATUS=$?
    set -e
    assert_output_scrubbed "${label}"
}

make_sourceable_script_copy() {
    local source="$1"
    local target="$2"
    awk '/^resolve_install_bootstrap_context_v1$/{exit} {print}' "${source}" > "${target}"
}

run_public_check_only_custom_prefix() {
    run_with_env public-check-only-custom-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/guest-home" \
        SUBSTRATE_TEST_MACHINE_ID="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa" \
        SUBSTRATE_FORWARDER_PORT="54321" \
        SUBSTRATE_WORLD_SOCKET="/tmp/ambient.sock" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    assert_status 0 "${RUN_STATUS}" "public custom-prefix check-only"
    assert_contains "[check-only] Declared VM name: substrate" "${STDOUT_PATH}" "public custom-prefix check-only"
    assert_contains "[check-only] Observed guest machine-id prefix: aaaaaaaaaaaa..." "${STDOUT_PATH}" "public custom-prefix check-only"
    assert_contains "[check-only] Observed transport target: selected-prefix/sock/agent.sock -> /run/substrate.sock" "${STDOUT_PATH}" "public custom-prefix check-only"
    assert_contains "[check-only] Forwarding activation unavailable: R3 prerequisite unmet." "${STDOUT_PATH}" "public custom-prefix check-only"
    assert_log_scrubbed "${RUN_LOG}" "public custom-prefix check-only"
    assert_no_lifecycle "${RUN_LOG}" "public custom-prefix check-only"
}

run_public_check_only_default_prefix() {
    run_with_env public-check-only-default-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/default-home" \
        SUBSTRATE_TEST_MACHINE_ID="bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb" \
        "${LIMA_WARM}" --check-only
    assert_status 0 "${RUN_STATUS}" "public default-prefix check-only"
    assert_contains "[check-only] Observed transport target: selected-prefix/sock/agent.sock -> /run/substrate.sock" "${STDOUT_PATH}" "public default-prefix check-only"
    assert_log_scrubbed "${RUN_LOG}" "public default-prefix check-only"
    assert_no_lifecycle "${RUN_LOG}" "public default-prefix check-only"
}

run_named_vm_check_only() {
    run_with_env named-vm-check-only \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_LIMA_VM_NAME="named-vm" \
        LIMA_VM_NAME="ambient-vm" \
        SUBSTRATE_TEST_VM_NAME="named-vm" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/named-vm-home" \
        SUBSTRATE_TEST_MACHINE_ID="abababababababababababababababab" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    assert_status 0 "${RUN_STATUS}" "named-vm check-only"
    assert_contains "[check-only] Declared VM name: named-vm" "${STDOUT_PATH}" "named-vm check-only"
    assert_contains "list named-vm --json" "${RUN_LOG}" "named-vm check-only"
    assert_contains "shell named-vm cat /etc/machine-id" "${RUN_LOG}" "named-vm check-only"
    assert_log_scrubbed "${RUN_LOG}" "named-vm check-only"
    assert_no_lifecycle "${RUN_LOG}" "named-vm check-only"
}

run_help_without_python3() {
    run_with_env warm-help-without-python3 \
        SUBSTRATE_TEST_PYTHON_MODE="missing" \
        "${LIMA_WARM}" --help
    assert_status 0 "${RUN_STATUS}" "warm help without python3"
    assert_contains "Usage: scripts/mac/lima-warm.sh" "${STDOUT_PATH}" "warm help without python3"
}

run_runtime_without_python3() {
    run_with_env warm-runtime-without-python3 \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_PYTHON_MODE="missing" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    assert_status 1 "${RUN_STATUS}" "warm runtime without python3"
    assert_contains "python3 is required for authenticated install bootstrap context resolution." "${STDERR_PATH}" "warm runtime without python3"
    assert_no_limactl_calls "${RUN_LOG}" "warm runtime without python3"
}

run_multiline_prefix_rejections() {
    local lf_prefix
    local cr_prefix

    lf_prefix="${CUSTOM_A}"$'\n'"suffix"
    cr_prefix="${CUSTOM_A}"$'\r'"suffix"

    run_with_env warm-multiline-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${lf_prefix}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm multiline prefix unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "warm multiline prefix rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm multiline prefix rejection"

    run_with_env doctor-multiline-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${lf_prefix}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor multiline prefix unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "doctor multiline prefix rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor multiline prefix rejection"

    run_with_env warm-carriage-return-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${cr_prefix}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm carriage-return prefix unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "warm carriage-return prefix rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm carriage-return prefix rejection"

    run_with_env doctor-carriage-return-prefix \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${cr_prefix}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor carriage-return prefix unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "doctor carriage-return prefix rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor carriage-return prefix rejection"
}

run_mapping_multiline_rejections() {
    local warm_copy
    local doctor_copy
    local lf_home
    local cr_home
    local control_root
    local host_socket

    warm_copy="$(mktemp "${WORK_ROOT}/warm-source.XXXXXX.sh")"
    doctor_copy="$(mktemp "${WORK_ROOT}/doctor-source.XXXXXX.sh")"
    make_sourceable_script_copy "${LIMA_WARM}" "${warm_copy}"
    make_sourceable_script_copy "${LIMA_DOCTOR}" "${doctor_copy}"

    lf_home=$'/srv/bad\nhome/.substrate'
    cr_home=$'/srv/cr\rhome/.substrate'
    control_root="${CURRENT_HOME%/}/.lima"
    host_socket="${SELECTED_A}/sock/agent.sock"

    # shellcheck disable=SC2016
    run_case warm-mapping-multiline \
        env \
            REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${warm_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            BAD_HOME="${lf_home}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            bash -lc '
                set -euo pipefail
                cd "${REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                OBSERVED_GUEST_MACHINE_ID="16161616161616161616161616161616"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                OBSERVED_GUEST_ACCOUNT="guest"
                OBSERVED_GUEST_UID="2000"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"
                verify_lima_mapping_v1
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm mapping multiline unexpectedly succeeded"
    assert_contains "invalid platform bootstrap mapping" "${STDERR_PATH}" "warm mapping multiline rejection"
    assert_not_contains "/srv/bad" "${STDERR_PATH}" "warm mapping multiline rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm mapping multiline rejection"

    # shellcheck disable=SC2016
    run_case doctor-mapping-multiline \
        env \
            REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${doctor_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            BAD_HOME="${lf_home}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            bash -lc '
                set -euo pipefail
                cd "${REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                OBSERVED_GUEST_MACHINE_ID="17171717171717171717171717171717"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                OBSERVED_GUEST_ACCOUNT="guest"
                OBSERVED_GUEST_UID="2000"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"
                verify_lima_mapping_v1
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor mapping multiline unexpectedly succeeded"
    assert_contains "invalid platform bootstrap mapping" "${STDERR_PATH}" "doctor mapping multiline rejection"
    assert_not_contains "/srv/bad" "${STDERR_PATH}" "doctor mapping multiline rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor mapping multiline rejection"

    # shellcheck disable=SC2016
    run_case warm-mapping-carriage-return \
        env \
            REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${warm_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            BAD_HOME="${cr_home}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            bash -lc '
                set -euo pipefail
                cd "${REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                OBSERVED_GUEST_MACHINE_ID="18181818181818181818181818181818"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                OBSERVED_GUEST_ACCOUNT="guest"
                OBSERVED_GUEST_UID="2000"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"
                verify_lima_mapping_v1
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm mapping carriage-return unexpectedly succeeded"
    assert_contains "invalid platform bootstrap mapping" "${STDERR_PATH}" "warm mapping carriage-return rejection"
    assert_not_contains "/srv/cr" "${STDERR_PATH}" "warm mapping carriage-return rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm mapping carriage-return rejection"

    # shellcheck disable=SC2016
    run_case doctor-mapping-carriage-return \
        env \
            REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${doctor_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            BAD_HOME="${cr_home}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            bash -lc '
                set -euo pipefail
                cd "${REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                OBSERVED_GUEST_MACHINE_ID="19191919191919191919191919191919"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                OBSERVED_GUEST_ACCOUNT="guest"
                OBSERVED_GUEST_UID="2000"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="/run/substrate.sock"
                verify_lima_mapping_v1
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor mapping carriage-return unexpectedly succeeded"
    assert_contains "invalid platform bootstrap mapping" "${STDERR_PATH}" "doctor mapping carriage-return rejection"
    assert_not_contains "/srv/cr" "${STDERR_PATH}" "doctor mapping carriage-return rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor mapping carriage-return rejection"
}

run_projected_unit_unsafe_character_rejections() {
    local warm_copy
    local doctor_copy
    local control_root
    local host_socket
    local guest_socket
    local backslash_home
    local quote_home
    local percent_home

    warm_copy="$(mktemp "${WORK_ROOT}/warm-unit-source.XXXXXX.sh")"
    doctor_copy="$(mktemp "${WORK_ROOT}/doctor-unit-source.XXXXXX.sh")"
    make_sourceable_script_copy "${LIMA_WARM}" "${warm_copy}"
    make_sourceable_script_copy "${LIMA_DOCTOR}" "${doctor_copy}"

    control_root="${CURRENT_HOME%/}/.lima"
    host_socket="${SELECTED_A}/sock/agent.sock"
    guest_socket="/run/substrate.sock"
    backslash_home='/srv/bad\home/.substrate'
    quote_home='/srv/bad"home/.substrate'
    percent_home='/srv/bad%home/.substrate'

    # shellcheck disable=SC2016
    run_case warm-unit-backslash \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${warm_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${backslash_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                write_systemd_units "${BAD_HOME}"
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm backslash projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDERR_PATH}" "warm backslash projected-unit rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm backslash projected-unit rejection"

    # shellcheck disable=SC2016
    run_case warm-unit-quote \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${warm_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${quote_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                write_systemd_units "${BAD_HOME}"
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm quote projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDERR_PATH}" "warm quote projected-unit rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm quote projected-unit rejection"

    # shellcheck disable=SC2016
    run_case warm-unit-percent \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${warm_copy}" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${percent_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                HOST_PLATFORM_CONTROL_ROOT="${CONTROL_ROOT}"
                OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                write_systemd_units "${BAD_HOME}"
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "warm percent projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDERR_PATH}" "warm percent projected-unit rejection"
    assert_no_limactl_calls "${RUN_LOG}" "warm percent projected-unit rejection"

    # shellcheck disable=SC2016
    run_case doctor-unit-backslash \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${doctor_copy}" \
            CASE_LOG="${WORK_ROOT}/doctor-unit-backslash.log" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${backslash_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                run_limactl_with_mapping_env_v1() {
                    printf "%s\n" "$*" >> "${CASE_LOG}"
                    case "$1" in
                        list)
                            if [[ "${3:-}" == "--json" ]]; then
                                printf "[{\"name\":\"%s\",\"status\":\"Running\"}]\n" "$2"
                            fi
                            return 0
                            ;;
                        shell)
                            if [[ "${3:-}" == "sudo" && "${4:-}" == "-n" && "${5:-}" == "cat" && "${6:-}" == "/etc/substrate-lima-layout" ]]; then
                                printf "%s\n" "${LAYOUT_EXPECTED}"
                                return 0
                            fi
                            ;;
                    esac
                    printf "unexpected doctor limactl invocation: %s\n" "$*" >&2
                    return 97
                }
                observe_lima_mapping_v1() {
                    OBSERVED_PLATFORM_MAPPING_V1="mapping"
                    OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                    OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                    OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                    OBSERVED_GUEST_MACHINE_ID="20202020202020202020202020202020"
                    OBSERVED_GUEST_ACCOUNT="guest"
                    OBSERVED_GUEST_UID="2000"
                }
                verify_lima_mapping_v1() { :; }
                check_rendered_unit_parity
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor backslash projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDOUT_PATH}" "doctor backslash projected-unit rejection"
    assert_not_contains "systemctl cat substrate-world-service.service" "${RUN_LOG}" "doctor backslash projected-unit rejection"
    assert_no_lifecycle "${RUN_LOG}" "doctor backslash projected-unit rejection"

    # shellcheck disable=SC2016
    run_case doctor-unit-quote \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${doctor_copy}" \
            CASE_LOG="${WORK_ROOT}/doctor-unit-quote.log" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${quote_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                run_limactl_with_mapping_env_v1() {
                    printf "%s\n" "$*" >> "${CASE_LOG}"
                    case "$1" in
                        list)
                            if [[ "${3:-}" == "--json" ]]; then
                                printf "[{\"name\":\"%s\",\"status\":\"Running\"}]\n" "$2"
                            fi
                            return 0
                            ;;
                        shell)
                            if [[ "${3:-}" == "sudo" && "${4:-}" == "-n" && "${5:-}" == "cat" && "${6:-}" == "/etc/substrate-lima-layout" ]]; then
                                printf "%s\n" "${LAYOUT_EXPECTED}"
                                return 0
                            fi
                            ;;
                    esac
                    printf "unexpected doctor limactl invocation: %s\n" "$*" >&2
                    return 97
                }
                observe_lima_mapping_v1() {
                    OBSERVED_PLATFORM_MAPPING_V1="mapping"
                    OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                    OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                    OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                    OBSERVED_GUEST_MACHINE_ID="21212121212121212121212121212121"
                    OBSERVED_GUEST_ACCOUNT="guest"
                    OBSERVED_GUEST_UID="2000"
                }
                verify_lima_mapping_v1() { :; }
                check_rendered_unit_parity
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor quote projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDOUT_PATH}" "doctor quote projected-unit rejection"
    assert_not_contains "systemctl cat substrate-world-service.service" "${RUN_LOG}" "doctor quote projected-unit rejection"
    assert_no_lifecycle "${RUN_LOG}" "doctor quote projected-unit rejection"

    # shellcheck disable=SC2016
    run_case doctor-unit-percent \
        env \
            SOURCE_REPO_ROOT="${REPO_ROOT}" \
            SOURCE_COPY="${doctor_copy}" \
            CASE_LOG="${WORK_ROOT}/doctor-unit-percent.log" \
            COMMITMENT="${A_COMMITMENT}" \
            CONTROL_ROOT="${control_root}" \
            HOST_SOCKET="${host_socket}" \
            GUEST_SOCKET="${guest_socket}" \
            BAD_HOME="${percent_home}" \
            bash -lc '
                set -euo pipefail
                cd "${SOURCE_REPO_ROOT}"
                set --
                source "${SOURCE_COPY}"
                CANONICAL_UNIT_SOURCE_DIR="${SOURCE_REPO_ROOT}/scripts/mac/lima/units"
                INSTALL_BOOTSTRAP_COMMITMENT="${COMMITMENT}"
                VM_NAME="substrate"
                run_limactl_with_mapping_env_v1() {
                    printf "%s\n" "$*" >> "${CASE_LOG}"
                    case "$1" in
                        list)
                            if [[ "${3:-}" == "--json" ]]; then
                                printf "[{\"name\":\"%s\",\"status\":\"Running\"}]\n" "$2"
                            fi
                            return 0
                            ;;
                        shell)
                            if [[ "${3:-}" == "sudo" && "${4:-}" == "-n" && "${5:-}" == "cat" && "${6:-}" == "/etc/substrate-lima-layout" ]]; then
                                printf "%s\n" "${LAYOUT_EXPECTED}"
                                return 0
                            fi
                            ;;
                    esac
                    printf "unexpected doctor limactl invocation: %s\n" "$*" >&2
                    return 97
                }
                observe_lima_mapping_v1() {
                    OBSERVED_PLATFORM_MAPPING_V1="mapping"
                    OBSERVED_GUEST_SUBSTRATE_HOME="${BAD_HOME}"
                    OBSERVED_TRANSPORT_HOST="${HOST_SOCKET}"
                    OBSERVED_TRANSPORT_GUEST_SOCKET="${GUEST_SOCKET}"
                    OBSERVED_GUEST_MACHINE_ID="22222222222222222222222222222222"
                    OBSERVED_GUEST_ACCOUNT="guest"
                    OBSERVED_GUEST_UID="2000"
                }
                verify_lima_mapping_v1() { :; }
                check_rendered_unit_parity
            '
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor percent projected-unit rejection unexpectedly succeeded"
    assert_contains "Verified guest unit projection contains a systemd-unsafe character." "${STDOUT_PATH}" "doctor percent projected-unit rejection"
    assert_not_contains "systemctl cat substrate-world-service.service" "${RUN_LOG}" "doctor percent projected-unit rejection"
    assert_no_lifecycle "${RUN_LOG}" "doctor percent projected-unit rejection"
}

run_check_only_layout_mismatch() {
    run_with_env check-only-layout-mismatch \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/layout-mismatch-home" \
        SUBSTRATE_TEST_MACHINE_ID="cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd" \
        SUBSTRATE_TEST_LAYOUT="socket-parity-v1" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    assert_status 0 "${RUN_STATUS}" "check-only layout mismatch"
    assert_contains "[check-only] Layout sentinel: socket-parity-v1" "${STDOUT_PATH}" "check-only layout mismatch"
    assert_contains "[check-only] Layout mismatch: R3 lifecycle reconciliation is required before guest projection." "${STDOUT_PATH}" "check-only layout mismatch"
    assert_not_contains "Agent socket metadata:" "${STDOUT_PATH}" "check-only layout mismatch"
    assert_not_contains "Guest systemd env" "${STDOUT_PATH}" "check-only layout mismatch"
    assert_not_contains "Guest gateway" "${STDOUT_PATH}" "check-only layout mismatch"
    assert_contains "sudo -n cat /etc/substrate-lima-layout" "${RUN_LOG}" "check-only layout mismatch"
    assert_log_scrubbed "${RUN_LOG}" "check-only layout mismatch"
    assert_no_lifecycle "${RUN_LOG}" "check-only layout mismatch"
}

run_internal_matching_projection() {
    local expected_lima="${CURRENT_HOME%/}/.lima"
    run_with_env internal-matching-projection \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${expected_lima}" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/internal-home" \
        SUBSTRATE_TEST_MACHINE_ID="cccccccccccccccccccccccccccccccc" \
        "${LIMA_WARM}" --check-only --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    assert_status 0 "${RUN_STATUS}" "internal matching projection"
    assert_contains "[check-only] Install context commitment: ${A_COMMITMENT:0:12}..." "${STDOUT_PATH}" "internal matching projection"
    assert_log_scrubbed "${RUN_LOG}" "internal matching projection"
    assert_no_lifecycle "${RUN_LOG}" "internal matching projection"
}

run_internal_conflicting_env_rejected() {
    local expected_lima="${CURRENT_HOME%/}/.lima"
    run_with_env internal-conflicting-home \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${expected_lima}" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "internal conflicting HOME unexpectedly succeeded"
    assert_contains "Internal Lima child HOME conflicts" "${STDERR_PATH}" "internal conflicting HOME rejection"
    assert_no_limactl_calls "${RUN_LOG}" "internal conflicting HOME rejection"

    run_with_env internal-conflicting-lima-home \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "internal conflicting LIMA_HOME unexpectedly succeeded"
    assert_contains "Internal Lima child LIMA_HOME conflicts" "${STDERR_PATH}" "internal conflicting LIMA_HOME rejection"
    assert_no_limactl_calls "${RUN_LOG}" "internal conflicting LIMA_HOME rejection"
}

run_tampered_carrier_rejections() {
    run_with_env tampered-carrier \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${CURRENT_HOME%/}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${TAMPERED_CARRIER}" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${TAMPERED_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "tampered carrier unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "tampered carrier rejection"
    assert_no_limactl_calls "${RUN_LOG}" "tampered carrier rejection"

    run_with_env reordered-carrier \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${CURRENT_HOME%/}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${REORDERED_CARRIER}" \
        SUBSTRATE_TEST_SCENARIO="check_only_running" \
        "${LIMA_WARM}" --check-only --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${REORDERED_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "reordered carrier unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "reordered carrier rejection"
    assert_no_limactl_calls "${RUN_LOG}" "reordered carrier rejection"

    run_with_env doctor-tampered-carrier \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${CURRENT_HOME%/}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${TAMPERED_CARRIER}" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${TAMPERED_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor tampered carrier unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "doctor tampered carrier rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor tampered carrier rejection"

    run_with_env doctor-reordered-carrier \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${CURRENT_HOME%/}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${REORDERED_CARRIER}" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${REORDERED_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor reordered carrier unexpectedly succeeded"
    assert_contains "invalid install bootstrap context" "${STDERR_PATH}" "doctor reordered carrier rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor reordered carrier rejection"
}

run_machine_id_and_home_rejections() {
    run_with_env machine-id-changes \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="machine_id_changes" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/guest-home" \
        SUBSTRATE_TEST_MACHINE_ID="dddddddddddddddddddddddddddddddd" \
        SUBSTRATE_TEST_MACHINE_ID_2="eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "changing machine-id unexpectedly succeeded"
    assert_contains "stable Lima guest machine identity" "${STDERR_PATH}" "changing machine-id rejection"
    assert_no_lifecycle "${RUN_LOG}" "changing machine-id rejection"

    run_with_env missing-getent-home \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="missing_getent_home" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/home/guest-from-env" \
        SUBSTRATE_TEST_MACHINE_ID="ffffffffffffffffffffffffffffffff" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "missing guest home unexpectedly succeeded"
    if ! grep -Fq "round-trip" "${STDERR_PATH}" && ! grep -Fq "platform bootstrap mapping" "${STDERR_PATH}"; then
        fail "missing guest home rejection did not fail closed"
    fi
    assert_no_lifecycle "${RUN_LOG}" "missing guest home rejection"

    run_with_env mismatched-principal-home \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="mismatched_principal_home" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/mismatched-home" \
        SUBSTRATE_TEST_MACHINE_ID="12121212121212121212121212121212" \
        "${LIMA_WARM}" --check-only --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "mismatched principal/home unexpectedly succeeded"
    assert_contains "round-trip" "${STDERR_PATH}" "mismatched principal/home rejection"
    assert_no_lifecycle "${RUN_LOG}" "mismatched principal/home rejection"
}

run_stage1_create_then_stage2_layout_stop() {
    run_with_env stage1-create-layout-stop \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="warm_create_layout_mismatch" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/create-home" \
        SUBSTRATE_TEST_MACHINE_ID="13131313131313131313131313131313" \
        SUBSTRATE_TEST_LAYOUT="socket-parity-v1" \
        "${LIMA_WARM}" --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "stage1 create layout-mismatch unexpectedly succeeded"
    assert_contains "R3 lifecycle reconciliation is required" "${STDERR_PATH}" "stage1 create layout stop"
    assert_log_scrubbed "${RUN_LOG}" "stage1 create layout stop"
    assert_order "${RUN_LOG}" "stage1 create order" \
        "list substrate --json$" \
        "start --tty=false --name substrate " \
        "shell substrate cat /etc/machine-id$" \
        "shell substrate id -un$" \
        "shell substrate id -u$" \
        "shell substrate getent passwd guest$" \
        "shell substrate getent passwd 2000$" \
        "shell substrate sudo -n cat /etc/substrate-lima-layout$"
    local create_line
    local machine_line
    local second_status_line
    create_line="$(grep -n -E -- 'start --tty=false --name substrate ' "${RUN_LOG}" | sed -n '1p' | cut -d: -f1)"
    second_status_line="$(grep -n -E -- 'list substrate --json$' "${RUN_LOG}" | sed -n '2p' | cut -d: -f1)"
    machine_line="$(grep -n -E -- 'shell substrate cat /etc/machine-id$' "${RUN_LOG}" | sed -n '1p' | cut -d: -f1)"
    [[ -n "${second_status_line}" && -n "${create_line}" && "${second_status_line}" -gt "${create_line}" ]] \
        || fail "stage1 create layout stop: missing post-create Running poll"
    [[ -n "${machine_line}" && "${second_status_line}" -lt "${machine_line}" ]] \
        || fail "stage1 create layout stop: Stage 2 began before the post-create Running poll"
    assert_not_contains $'\tstop ' "${RUN_LOG}" "stage1 create layout stop"
    assert_not_contains $'\tdelete ' "${RUN_LOG}" "stage1 create layout stop"
    assert_not_contains $'\tcopy ' "${RUN_LOG}" "stage1 create layout stop"
    assert_not_contains "shell substrate bash" "${RUN_LOG}" "stage1 create layout stop"
}

run_stage1_start_then_stage2_layout_stop() {
    run_with_env stage1-start-layout-stop \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="warm_stopped_layout_mismatch" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/start-home" \
        SUBSTRATE_TEST_MACHINE_ID="14141414141414141414141414141414" \
        SUBSTRATE_TEST_LAYOUT="socket-parity-v1" \
        "${LIMA_WARM}" --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "stage1 start layout-mismatch unexpectedly succeeded"
    assert_contains "R3 lifecycle reconciliation is required" "${STDERR_PATH}" "stage1 start layout stop"
    assert_log_scrubbed "${RUN_LOG}" "stage1 start layout stop"
    assert_order "${RUN_LOG}" "stage1 start order" \
        "list substrate --json$" \
        "start substrate$" \
        "shell substrate cat /etc/machine-id$" \
        "shell substrate id -un$" \
        "shell substrate id -u$" \
        "shell substrate getent passwd guest$" \
        "shell substrate getent passwd 2000$" \
        "shell substrate sudo -n cat /etc/substrate-lima-layout$"
    local start_line
    local machine_line
    local third_status_line
    start_line="$(grep -n -E -- 'start substrate$' "${RUN_LOG}" | sed -n '1p' | cut -d: -f1)"
    third_status_line="$(grep -n -E -- 'list substrate --json$' "${RUN_LOG}" | sed -n '3p' | cut -d: -f1)"
    machine_line="$(grep -n -E -- 'shell substrate cat /etc/machine-id$' "${RUN_LOG}" | sed -n '1p' | cut -d: -f1)"
    [[ -n "${third_status_line}" && -n "${start_line}" && "${third_status_line}" -gt "${start_line}" ]] \
        || fail "stage1 start layout stop: missing post-start Running poll"
    [[ -n "${machine_line}" && "${third_status_line}" -lt "${machine_line}" ]] \
        || fail "stage1 start layout stop: Stage 2 began before the post-start Running poll"
    assert_not_contains $'\tstop ' "${RUN_LOG}" "stage1 start layout stop"
    assert_not_contains $'\tdelete ' "${RUN_LOG}" "stage1 start layout stop"
    assert_not_contains $'\tcopy ' "${RUN_LOG}" "stage1 start layout stop"
    assert_not_contains "shell substrate bash" "${RUN_LOG}" "stage1 start layout stop"
}

run_stage1_broken_status_rejected() {
    run_with_env stage1-broken-status \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_SCENARIO="warm_broken_status" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/broken-home" \
        SUBSTRATE_TEST_MACHINE_ID="15151515151515151515151515151515" \
        "${LIMA_WARM}" --install-prefix "${CUSTOM_A}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "stage1 broken status unexpectedly succeeded"
    assert_contains "unsupported wait status 'Broken'" "${STDERR_PATH}" "stage1 broken status rejection"
    assert_order "${RUN_LOG}" "stage1 broken status order" \
        "list substrate --json$" \
        "start substrate$"
    assert_not_contains "shell substrate cat /etc/machine-id" "${RUN_LOG}" "stage1 broken status rejection"
    assert_not_contains $'\tstop ' "${RUN_LOG}" "stage1 broken status rejection"
    assert_not_contains $'\tdelete ' "${RUN_LOG}" "stage1 broken status rejection"
    assert_not_contains $'\tcopy ' "${RUN_LOG}" "stage1 broken status rejection"
}

run_doctor_no_lifecycle() {
    run_with_env doctor-no-lifecycle \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_FORWARDER_PORT="4545" \
        SUBSTRATE_WORLD_SOCKET="/tmp/ambient.sock" \
        SUBSTRATE_TEST_EXPECTED_COMMITMENT="${CUSTOM_COMMITMENT}" \
        SUBSTRATE_TEST_EXPECTED_PREFIX="${CUSTOM_A}" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/doctor-home" \
        SUBSTRATE_TEST_MACHINE_ID="15151515151515151515151515151515" \
        SUBSTRATE_TEST_LAYOUT="socket-parity-v2-staged-workspace-v1" \
        "${LIMA_DOCTOR}" --install-prefix "${CUSTOM_A}"
    assert_status 0 "${RUN_STATUS}" "doctor no-lifecycle"
    assert_contains "selected commitment:" "${STDOUT_PATH}" "doctor no-lifecycle"
    assert_contains "Lima control root: resolved from account database." "${STDOUT_PATH}" "doctor no-lifecycle"
    assert_contains "Observed guest machine-id prefix 151515151515..." "${STDOUT_PATH}" "doctor no-lifecycle"
    assert_contains "Forwarding activation unavailable here: R3 prerequisite unmet." "${STDOUT_PATH}" "doctor no-lifecycle"
    assert_contains "Bounded routed readiness prerequisites passed. Forwarding activation remains an R3 prerequisite." "${STDOUT_PATH}" "doctor no-lifecycle"
    assert_log_scrubbed "${RUN_LOG}" "doctor no-lifecycle"
    assert_no_lifecycle "${RUN_LOG}" "doctor no-lifecycle"
}

run_doctor_internal_matching_projection() {
    local expected_lima="${CURRENT_HOME%/}/.lima"
    run_with_env doctor-internal-matching-projection \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${expected_lima}" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_FORWARDER_PORT="4545" \
        SUBSTRATE_WORLD_SOCKET="/tmp/ambient.sock" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/doctor-internal-home" \
        SUBSTRATE_TEST_MACHINE_ID="dededededededededededededededede" \
        "${LIMA_DOCTOR}" --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    assert_status 0 "${RUN_STATUS}" "doctor internal matching projection"
    assert_contains "selected commitment: ${A_COMMITMENT:0:12}..." "${STDOUT_PATH}" "doctor internal matching projection"
    assert_log_scrubbed "${RUN_LOG}" "doctor internal matching projection"
    assert_no_lifecycle "${RUN_LOG}" "doctor internal matching projection"
}

run_doctor_internal_conflicting_env_rejected() {
    local expected_lima="${CURRENT_HOME%/}/.lima"
    run_with_env doctor-internal-conflicting-home \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${expected_lima}" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor internal conflicting HOME unexpectedly succeeded"
    assert_contains "Internal Lima child HOME conflicts" "${STDERR_PATH}" "doctor internal conflicting HOME rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor internal conflicting HOME rejection"

    run_with_env doctor-internal-conflicting-lima-home \
        HOME="${CURRENT_HOME}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_HOME="${SELECTED_A}" \
        SUBSTRATE_ROOT="${SELECTED_A}" \
        SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${A_COMMITMENT}" \
        SUBSTRATE_INSTALL_PRIMARY_USER="${CURRENT_ACCOUNT}" \
        SUBSTRATE_INSTALL_PRIMARY_UID="${CURRENT_UID}" \
        SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${A_CARRIER}" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_SCENARIO="doctor_running" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="healthy" \
        "${LIMA_DOCTOR}" --install-prefix "${SELECTED_A}" --install-bootstrap-context-v1 "${A_CARRIER}"
    [[ "${RUN_STATUS}" -ne 0 ]] || fail "doctor internal conflicting LIMA_HOME unexpectedly succeeded"
    assert_contains "Internal Lima child LIMA_HOME conflicts" "${STDERR_PATH}" "doctor internal conflicting LIMA_HOME rejection"
    assert_no_limactl_calls "${RUN_LOG}" "doctor internal conflicting LIMA_HOME rejection"
}

run_public_check_only_custom_prefix
run_public_check_only_default_prefix
run_named_vm_check_only
run_help_without_python3
run_runtime_without_python3
run_multiline_prefix_rejections
run_mapping_multiline_rejections
run_projected_unit_unsafe_character_rejections
run_check_only_layout_mismatch
run_internal_matching_projection
run_internal_conflicting_env_rejected
run_tampered_carrier_rejections
run_machine_id_and_home_rejections
run_stage1_create_then_stage2_layout_stop
run_stage1_start_then_stage2_layout_stop
run_stage1_broken_status_rejected
run_doctor_no_lifecycle
run_doctor_internal_matching_projection
run_doctor_internal_conflicting_env_rejected

[[ "$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -exec basename {} \;)" == "sentinel" ]] \
    || fail "ambient B was mutated"
[[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"

printf '[%s] PASS\n' "${SCRIPT_NAME}"
