#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="mac-doctor-fixture"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DOCTOR_SCRIPT="${REPO_ROOT}/scripts/mac/lima-doctor.sh"
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
KEEP_ROOT="${KEEP_MAC_DOCTOR_FIXTURE:-0}"
STUB_BIN="${WORK_ROOT}/bin"
STATE_ROOT="${WORK_ROOT}/state"
STDOUT_PATH="${WORK_ROOT}/stdout"
STDERR_PATH="${WORK_ROOT}/stderr"
SELECTED_A="${WORK_ROOT}/selected-a"
AMBIENT_B="${WORK_ROOT}/ambient-b"
mkdir -p "${STUB_BIN}" "${STATE_ROOT}" "${SELECTED_A}" "${AMBIENT_B}"
chmod 0700 "${WORK_ROOT}" "${STUB_BIN}" "${STATE_ROOT}" "${SELECTED_A}" "${AMBIENT_B}"
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

A_CONTEXT_OUTPUT="$(make_context_values "${SELECTED_A}" "${CURRENT_ACCOUNT}" "${CURRENT_UID}")"
A_CARRIER="$(printf '%s\n' "${A_CONTEXT_OUTPUT}" | sed -n '1p')"
A_COMMITMENT="$(printf '%s\n' "${A_CONTEXT_OUTPUT}" | sed -n '2p')"

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
exit_mode = False
expr = None
path = None
jq_args = {}
while args:
    token = args.pop(0)
    if token == "-r":
        continue
    if token == "-e":
        exit_mode = True
        continue
    if token == "--arg":
        key = args.pop(0)
        jq_args[key] = args.pop(0)
        continue
    if expr is None:
        expr = token
    else:
        path = token
if expr is None:
    sys.exit(2)
payload = open(path, "r", encoding="utf-8").read() if path else sys.stdin.read()
try:
    data = json.loads(payload)
except Exception:
    data = {}

def host_ok(obj):
    return obj.get("ok") is True and isinstance(obj.get("host"), dict) and obj["host"].get("ok") is True

def world_ok(obj):
    return (
        host_ok(obj)
        and isinstance(obj.get("world"), dict)
        and obj["world"].get("ok") is True
        and obj["world"].get("status") == "ok"
    )

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
    print("true" if ok else "false")
    sys.exit(0 if ok or not exit_mode else 1)
if expr == '.ok == true and .host.ok == true and .world.ok == true and .world.status == "ok"':
    ok = world_ok(data)
    print("true" if ok else "false")
    sys.exit(0 if ok or not exit_mode else 1)
print("unknown")
sys.exit(0)
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

write_prefix_bound_substrate_bin() {
    mkdir -p "${SELECTED_A}/bin"
    cat > "${SELECTED_A}/bin/substrate" <<'STUB'
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
    chmod +x "${SELECTED_A}/bin/substrate"
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
guest_uid="${SUBSTRATE_TEST_GUEST_UID:-2000}"
guest_home="${SUBSTRATE_TEST_GUEST_HOME:-/srv/doctor-home}"
guest_substrate_home="${guest_home%/}/.substrate"
machine_id="${SUBSTRATE_TEST_MACHINE_ID:-16161616161616161616161616161616}"
layout_value="${SUBSTRATE_TEST_LAYOUT:-socket-parity-v2-staged-workspace-v1}"
unit_dir="${SUBSTRATE_TEST_UNIT_DIR:?}"

record() {
    printf 'HOME=%s\tLIMA_HOME=%s\t%s\n' "${HOME:-}" "${LIMA_HOME:-}" "$*" >> "${log_path}"
}

render_unit() {
    local kind="$1"
    python3 - "${unit_dir}" "${guest_substrate_home}" "${kind}" <<'PY'
import pathlib
import sys

unit_dir = pathlib.Path(sys.argv[1])
guest_home = sys.argv[2]
kind = sys.argv[3]
if kind == "service":
    template = (unit_dir / "substrate-world-service.service.tmpl").read_text(encoding="utf-8")
    template = template.replace("${SUBSTRATE_GUEST_HOME}", guest_home).replace("$SUBSTRATE_GUEST_HOME", guest_home)
    template = template.replace("${WORLD_NETFILTER_ENV}", "").replace("$WORLD_NETFILTER_ENV", "")
    sys.stdout.write(template)
else:
    sys.stdout.write((unit_dir / "substrate-world-service.socket").read_text(encoding="utf-8"))
PY
    if [[ "${scenario}" == "unit_mismatch" && "${kind}" == "service" ]]; then
        printf '# fixture-mismatch\n'
    fi
}

if [[ $# -lt 1 ]]; then
    exit 1
fi

cmd="$1"
shift
record "${cmd} $*"

case "${cmd}" in
    list)
        vm="${1:-}"
        [[ "${vm}" == "${expected_vm}" ]] || exit 1
        shift || true
        json=0
        [[ "${1:-}" == "--json" ]] && json=1
        case "${scenario}" in
            vm_missing)
                exit 1
                ;;
            vm_stopped)
                if [[ "${json}" -eq 1 ]]; then
                    printf '[{"name":"%s","status":"Stopped"}]\n' "${vm}"
                else
                    printf '%s\tStopped\n' "${vm}"
                fi
                ;;
            *)
                if [[ "${json}" -eq 1 ]]; then
                    printf '[{"name":"%s","status":"Running"}]\n' "${vm}"
                else
                    printf '%s\tRunning\n' "${vm}"
                fi
                ;;
        esac
        exit 0
        ;;
    start|stop|delete|copy)
        printf 'unexpected lifecycle action under scenario %s: %s %s\n' "${scenario}" "${cmd}" "$*" >&2
        exit 97
        ;;
    shell)
        vm="${1:-}"
        shift || true
        [[ "${vm}" == "${expected_vm}" ]] || exit 1
        case "${1:-}" in
            cat)
                if [[ "${2:-}" == "/etc/machine-id" ]]; then
                    printf '%s\n' "${machine_id}"
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
                    printf '%s:x:%s:20::%s:/bin/bash\n' "${guest_account}" "${guest_uid}" "${guest_home}"
                    exit 0
                fi
                ;;
            uname)
                printf 'Linux substrate\n'
                exit 0
                ;;
            which)
                [[ "${2:-}" == "nft" ]] && printf '/usr/sbin/nft\n' && exit 0
                ;;
            bash)
                printf '/dev/root 10G 5G 5G 50%% /\n'
                exit 0
                ;;
            systemctl)
                if [[ "${2:-}" == "is-active" && "${3:-}" == "substrate-world-service" ]]; then
                    printf 'active\n'
                    exit 0
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
                            [[ "${scenario}" == "socket_missing" ]] && exit 1
                            exit 0
                        fi
                        ;;
                    timeout)
                        printf '{}\n'
                        exit 0
                        ;;
                    stat)
                        printf 'root:substrate 660\n'
                        exit 0
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
                    cat)
                        if [[ "${2:-}" == "/etc/substrate-lima-layout" ]]; then
                            printf '%s\n' "${layout_value}"
                            exit 0
                        fi
                        ;;
                esac
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
    write_stub_vsock_proxy
    write_stub_substrate
    write_prefix_bound_substrate_bin
    write_stub_limactl
}

setup_stubs

assert_status() {
    local expected="$1"
    local actual="$2"
    local label="$3"
    [[ "${actual}" -eq "${expected}" ]] || fail "${label}: exit ${actual}, expected ${expected}"
}

assert_contains() {
    local needle="$1"
    local file="$2"
    local label="$3"
    grep -Fq -- "${needle}" "${file}" || fail "${label}: missing '${needle}'"
}

assert_contains_either() {
    local needle="$1"
    local file_a="$2"
    local file_b="$3"
    local label="$4"
    if grep -Fq -- "${needle}" "${file_a}" || grep -Fq -- "${needle}" "${file_b}"; then
        return 0
    fi
    fail "${label}: missing '${needle}'"
}

assert_not_contains() {
    local needle="$1"
    local file="$2"
    local label="$3"
    if grep -Fq -- "${needle}" "${file}"; then
        fail "${label}: unexpectedly found '${needle}'"
    fi
}

assert_log_scrubbed() {
    local log_path="$1"
    local label="$2"
    local expected_lima="${CURRENT_HOME%/}/.lima"
    if grep -Fq -- "HOME=${AMBIENT_B}" "${log_path}" || grep -Fq -- "LIMA_HOME=${AMBIENT_B}" "${log_path}"; then
        fail "${label}: ambient B reached limactl"
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
    local expected_lima="${CURRENT_HOME%/}/.lima"
    for file in "${STDOUT_PATH}" "${STDERR_PATH}"; do
        assert_not_contains "${A_CARRIER}" "${file}" "${label}"
        assert_not_contains "${A_COMMITMENT}" "${file}" "${label}"
        assert_not_contains "${SELECTED_A}" "${file}" "${label}"
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

run_doctor_case() {
    local label="$1"
    local scenario="$2"
    local substrate_scenario="$3"
    local breakglass="$4"
    local expected_exit="$5"
    local needle="$6"
    local vm_name="${7:-substrate}"
    local layout_value="${8:-socket-parity-v2-staged-workspace-v1}"
    RUN_LOG="${WORK_ROOT}/${label}.log"
    RUN_STATE_DIR="${STATE_ROOT}/${label}"
    mkdir -p "${RUN_STATE_DIR}"
    : > "${RUN_LOG}"
    : > "${STDOUT_PATH}"
    : > "${STDERR_PATH}"
    set +e
    env \
        PATH="${PATH}" \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_FORWARDER_PORT="4545" \
        SUBSTRATE_WORLD_SOCKET="/tmp/ambient.sock" \
        SUBSTRATE_TEST_REQUIRE_PREFIX_BIN="1" \
        SUBSTRATE_TEST_LOG="${RUN_LOG}" \
        SUBSTRATE_TEST_STATE_DIR="${RUN_STATE_DIR}" \
        SUBSTRATE_TEST_UNIT_DIR="${REPO_ROOT}/scripts/mac/lima/units" \
        SUBSTRATE_TEST_SCENARIO="${scenario}" \
        SUBSTRATE_TEST_SUBSTRATE_SCENARIO="${substrate_scenario}" \
        SUBSTRATE_TEST_VM_NAME="${vm_name}" \
        SUBSTRATE_TEST_GUEST_ACCOUNT="guest" \
        SUBSTRATE_TEST_GUEST_UID="2000" \
        SUBSTRATE_TEST_GUEST_HOME="/srv/doctor-home" \
        SUBSTRATE_TEST_MACHINE_ID="17171717171717171717171717171717" \
        SUBSTRATE_TEST_LAYOUT="${layout_value}" \
        SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS="${breakglass}" \
        SUBSTRATE_LIMA_VM_NAME="${vm_name}" \
        LIMA_VM_NAME="ambient-vm" \
        "${DOCTOR_SCRIPT}" --install-prefix "${SELECTED_A}" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
    RUN_STATUS=$?
    set -e

    assert_status "${expected_exit}" "${RUN_STATUS}" "${label}"
    assert_contains "selected commitment:" "${STDOUT_PATH}" "${label}"
    assert_contains "Lima control root: resolved from account database." "${STDOUT_PATH}" "${label}"
    assert_contains_either "${needle}" "${STDOUT_PATH}" "${STDERR_PATH}" "${label}"
    assert_output_scrubbed "${label}"
    assert_log_scrubbed "${RUN_LOG}" "${label}"
    assert_no_lifecycle "${RUN_LOG}" "${label}"
}

run_doctor_help_without_python3() {
    RUN_LOG="${WORK_ROOT}/doctor-help.log"
    : > "${RUN_LOG}"
    : > "${STDOUT_PATH}"
    : > "${STDERR_PATH}"
    set +e
    env \
        PATH="${PATH}" \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_PYTHON_MODE="missing" \
        "${DOCTOR_SCRIPT}" --help >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
    RUN_STATUS=$?
    set -e
    assert_status 0 "${RUN_STATUS}" "doctor help without python3"
    assert_contains "Usage: scripts/mac/lima-doctor.sh" "${STDOUT_PATH}" "doctor help without python3"
}

run_doctor_runtime_without_python3() {
    RUN_LOG="${WORK_ROOT}/doctor-runtime-no-python.log"
    : > "${RUN_LOG}"
    : > "${STDOUT_PATH}"
    : > "${STDERR_PATH}"
    set +e
    env \
        PATH="${PATH}" \
        HOME="${AMBIENT_B}" \
        LIMA_HOME="${AMBIENT_B}/.lima" \
        SUBSTRATE_TEST_PYTHON_MODE="missing" \
        "${DOCTOR_SCRIPT}" --install-prefix "${SELECTED_A}" >"${STDOUT_PATH}" 2>"${STDERR_PATH}"
    RUN_STATUS=$?
    set -e
    assert_status 1 "${RUN_STATUS}" "doctor runtime without python3"
    assert_contains "python3 is required for authenticated install bootstrap context resolution." "${STDERR_PATH}" "doctor runtime without python3"
    assert_output_scrubbed "doctor runtime without python3"
    assert_no_lifecycle "${RUN_LOG}" "doctor runtime without python3"
}

run_doctor_help_without_python3
run_doctor_runtime_without_python3
run_doctor_case "healthy" "healthy" "healthy" "0" 0 "Bounded routed readiness prerequisites passed. Forwarding activation remains an R3 prerequisite."
run_doctor_case "healthy-custom-vm" "healthy" "healthy" "0" 0 "declared VM: doctor-named-vm" "doctor-named-vm"
run_doctor_case "world-fail" "healthy" "world_fail" "0" 1 "Doctor detected 1 routed-readiness issue(s)."
run_doctor_case "layout-mismatch" "healthy" "healthy" "0" 1 "Layout sentinel socket-parity-v1" "substrate" "socket-parity-v1"
run_doctor_case "unit-mismatch" "unit_mismatch" "healthy" "0" 1 "Guest service unit differs from the canonical rendered contract"
run_doctor_case "socket-missing-breakglass" "socket_missing" "healthy" "1" 1 "Agent socket not found (breakglass)."
assert_contains "Doctor detected 1 guest-direct breakglass issue(s)." "${STDERR_PATH}" "socket-missing-breakglass"
run_doctor_case "vm-missing" "vm_missing" "healthy" "0" 1 "VM 'substrate' does not exist; run \`substrate world enable\` first; if you need the current helper-backed declared-instance Stage-1 create/start path directly, \`scripts/mac/lima-warm.sh\` remains degraded-but-supported."
run_doctor_case "vm-stopped" "vm_stopped" "healthy" "0" 1 "VM 'substrate' is not running (status: Stopped); run \`substrate world enable\` first; if you need the current helper-backed declared-instance Stage-1 create/start path directly, \`scripts/mac/lima-warm.sh\` remains degraded-but-supported."

[[ "$(find "${AMBIENT_B}" -mindepth 1 -maxdepth 1 -exec basename {} \;)" == "sentinel" ]] \
    || fail "ambient B was mutated"
[[ "$(<"${AMBIENT_B}/sentinel")" == "ambient-b-sentinel" ]] \
    || fail "ambient B sentinel changed"

printf '[%s] PASS\n' "${SCRIPT_NAME}"
