#!/usr/bin/env bash
set -euo pipefail

failures=0
LAYOUT_EXPECTED="socket-parity-v2-staged-workspace-v1"
SCRIPTS_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPTS_ROOT}/../.." && pwd)"
CANONICAL_UNIT_SOURCE_DIR="${REPO_ROOT}/scripts/mac/lima/units"
VM_NAME="${SUBSTRATE_LIMA_VM_NAME:-${LIMA_VM_NAME:-substrate}}"
RUN_BREAKGLASS_CHECKS="${SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS:-0}"

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

host_sha256() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

check_rendered_unit_parity() {
    local parity_tmp=""
    local expected_dir=""
    local actual_dir=""
    local vm_user=""
    local vm_home=""
    local guest_substrate_home=""
    local enable_netfilter="${SUBSTRATE_WORLD_NETFILTER_ENABLE:-0}"
    local expected_netfilter_env=""
    local expected_service_sha=""
    local expected_socket_sha=""
    local actual_service_sha=""
    local actual_socket_sha=""
    local mismatch=0

    if [[ ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" || ! -f "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" ]]; then
        warn "Missing canonical unit sources under ${CANONICAL_UNIT_SOURCE_DIR}."
        return 1
    fi

    if ! command -v envsubst >/dev/null 2>&1; then
        warn "envsubst is required to render the canonical guest units locally."
        return 1
    fi

    parity_tmp="$(mktemp -d)"
    expected_dir="${parity_tmp}/expected"
    actual_dir="${parity_tmp}/actual"
    mkdir -p "${expected_dir}" "${actual_dir}"

    vm_user="$(limactl shell "${VM_NAME}" id -un 2>/dev/null | tr -d '\r' || true)"
    if [[ -z "${vm_user}" ]]; then
        warn "Unable to determine the Lima guest user for rendered-unit parity."
        rm -rf "${parity_tmp}"
        return 1
    fi

    vm_home="$(limactl shell "${VM_NAME}" getent passwd "${vm_user}" 2>/dev/null | cut -d: -f6 | tr -d '\r' || true)"
    if [[ -z "${vm_home}" ]]; then
        vm_home="/home/${vm_user}"
    fi
    guest_substrate_home="${vm_home}/.substrate"

    case "${enable_netfilter}" in
        1|true|yes|TRUE|YES)
            expected_netfilter_env="Environment=WORLD_NETFILTER_ENABLE=1"
            ;;
    esac

    SUBSTRATE_GUEST_HOME="${guest_substrate_home}" WORLD_NETFILTER_ENV="${expected_netfilter_env}" \
        envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.service.tmpl" > "${expected_dir}/substrate-world-service.service"
    envsubst < "${CANONICAL_UNIT_SOURCE_DIR}/substrate-world-service.socket" > "${expected_dir}/substrate-world-service.socket"

    if ! limactl shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.service \
        | sed '/^# \//d' \
        | awk 'BEGIN { seen=0 } { if (!seen && $0 == "") next; seen=1; print }' > "${actual_dir}/substrate-world-service.service"; then
        warn "Unable to capture the loaded guest service unit via systemctl cat."
        rm -rf "${parity_tmp}"
        return 1
    fi

    if ! limactl shell "${VM_NAME}" sudo -n systemctl cat substrate-world-service.socket \
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

    if ! limactl list "${VM_NAME}" >/dev/null 2>&1; then
        warn "VM '${VM_NAME}' does not exist; run scripts/mac/lima-warm.sh to create it."
        return
    fi

    local status=""
    status="$(limactl list "${VM_NAME}" --json | jq -r '.status // "unknown"' 2>/dev/null || true)"
    if [[ "${status}" != "Running" ]]; then
        warn "VM '${VM_NAME}' is not running (status: ${status:-unknown}); run scripts/mac/lima-warm.sh."
        return
    fi

    printf "\033[32m[PASS]\033[0m VM '%s' exists and is running\n" "${VM_NAME}"
    diagnose "SSH connectivity (breakglass)" limactl shell "${VM_NAME}" uname -a

    if limactl shell "${VM_NAME}" sudo -n test -S /run/substrate.sock >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m Agent socket exists (breakglass)\n'

        if limactl shell "${VM_NAME}" sudo -n timeout 5 curl --fail --unix-socket /run/substrate.sock http://localhost/v1/capabilities >/dev/null 2>&1; then
            printf '\033[32m[PASS]\033[0m Agent responds to direct capabilities probe (breakglass)\n'
        else
            warn "Agent direct capabilities probe failed (breakglass)."
        fi

        local socket_meta=""
        socket_meta="$(limactl shell "${VM_NAME}" sudo -n stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)"
        if [[ "${socket_meta}" == "root:substrate 660" ]]; then
            printf '\033[32m[PASS]\033[0m Socket ownership root:substrate (0660) (breakglass)\n'
        else
            warn "Socket metadata ${socket_meta:-unknown} (expected root:substrate 660). Run scripts/mac/lima-warm.sh to repair."
        fi

        local vm_user=""
        vm_user="$(limactl shell "${VM_NAME}" id -un 2>/dev/null | tr -d '\r' || true)"
        if [[ -n "${vm_user}" ]] && limactl shell "${VM_NAME}" id -nG "${vm_user}" 2>/dev/null | tr ' ' '\n' | grep -qx substrate; then
            printf '\033[32m[PASS]\033[0m %s belongs to substrate group (breakglass)\n' "${vm_user}"
        else
            warn "Unable to confirm substrate group membership for ${vm_user:-guest}. Run scripts/mac/lima-warm.sh."
        fi

        local layout_version=""
        layout_version="$(limactl shell "${VM_NAME}" sudo -n cat /etc/substrate-lima-layout 2>/dev/null | tr -d '\r' || true)"
        if [[ "${layout_version}" == "${LAYOUT_EXPECTED}" ]]; then
            printf '\033[32m[PASS]\033[0m Socket parity layout detected (%s) (breakglass)\n' "${layout_version}"
        else
            warn "Layout sentinel ${layout_version:-missing} (expected ${LAYOUT_EXPECTED}). Run scripts/mac/lima-warm.sh to rebuild."
        fi
    else
        warn "Agent socket not found (breakglass)."
    fi

    if limactl shell "${VM_NAME}" systemctl is-active substrate-world-service >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m substrate-world-service service is active (breakglass)\n'
    else
        warn "substrate-world-service service is not active (breakglass)."
    fi

    if limactl shell "${VM_NAME}" which nft >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m nftables available (breakglass)\n'
    else
        warn "nftables unavailable in guest (breakglass)."
    fi

    echo ""
    echo "Disk Usage (breakglass):"
    limactl shell "${VM_NAME}" bash -lc 'df -h / | tail -1' 2>/dev/null || warn "Could not get guest disk usage."
}

SUBSTRATE_BIN="$(resolve_substrate_bin)"

echo "=== Substrate Lima Doctor ==="
echo ""
echo "Support posture:"
echo "  supported: substrate host doctor [--json]; substrate world doctor [--json]; substrate world gateway sync|status|restart; substrate world enable; substrate world deps current sync for dependency reconciliation"
echo "  degraded-but-supported: scripts/mac/lima-doctor.sh wraps routed readiness and only escalates to guest-direct diagnostics after failure or explicit opt-in; scripts/mac/lima-warm.sh retains create/warm/repair plus staged-workspace copy"
echo "  breakglass: raw limactl shell, plain SSH, direct guest systemctl, guest socket curl, guest journalctl, and host-side SUBSTRATE_WORLD_SOCKET override use"
echo "  note: substrate workspace sync is not the frozen normal macOS sync/copy path in this packet"
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
echo "Routed Readiness Proof (canonical for provisioned backends):"
note_routed_override_bypass
check_doctor_json "substrate host doctor --json" '.ok == true and .host.ok == true' run_routed_readiness_command "${SUBSTRATE_BIN}" host doctor --json
check_doctor_json "substrate world doctor --json" '.ok == true and .host.ok == true and .world.ok == true and .world.status == "ok"' run_routed_readiness_command "${SUBSTRATE_BIN}" world doctor --json
check_with_failure_detail "Canonical rendered guest units match the loaded service/socket contract" check_rendered_unit_parity

echo ""
if [[ "${RUN_BREAKGLASS_CHECKS}" == "1" || "${failures}" -ne 0 ]]; then
    run_breakglass_guest_checks
else
    echo "Guest-Direct Breakglass Diagnostics:"
    echo "  Skipped because routed readiness is healthy."
    echo "  Set SUBSTRATE_MAC_DOCTOR_INCLUDE_BREAKGLASS=1 to run guest-direct diagnostics explicitly."
fi

echo ""
if [ "${failures}" -ne 0 ]; then
    echo "Doctor detected ${failures} routed-readiness issue(s). See above output for details." >&2
    exit 1
else
    echo "All routed critical checks passed."
    exit 0
fi
