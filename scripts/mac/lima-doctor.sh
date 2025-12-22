#!/usr/bin/env bash
set -euo pipefail

failures=0
LAYOUT_EXPECTED="socket-parity-v1"

# Function to check a condition
check() {
    local name="$1"
    shift
    if "$@" >/dev/null 2>&1; then
        printf '\033[32m[PASS]\033[0m %s\n' "$name"
    else
        printf '\033[31m[FAIL]\033[0m %s\n' "$name"
        failures=$((failures+1))
    fi
}

# Function to check with visible output
check_verbose() {
    local name="$1"
    shift
    echo "Checking: $name"
    if "$@"; then
        printf '\033[32m[PASS]\033[0m %s\n' "$name"
    else
        printf '\033[31m[FAIL]\033[0m %s\n' "$name"
        failures=$((failures+1))
    fi
}

echo "=== Substrate Lima Doctor ==="
echo ""

# Host checks
echo "Host Environment:"
check "Lima installed" command -v limactl
check "jq installed" command -v jq
check "envsubst installed" command -v envsubst
check "Virtualization available" test "$(sysctl -n kern.hv_support 2>/dev/null)" -eq 1

# Check for vsock-proxy (may not be available in all Lima installations)
if command -v vsock-proxy >/dev/null 2>&1; then
    printf '\033[32m[PASS]\033[0m vsock-proxy available\n'
else
    printf '\033[33m[WARN]\033[0m vsock-proxy not found (will use SSH forwarding as fallback)\n'
fi

echo ""
echo "VM Status:"

# Check if VM exists
if limactl list substrate >/dev/null 2>&1; then
    status=$(limactl list substrate --json | jq -r '.status // "unknown"')
    if [ "$status" = "Running" ]; then
        printf '\033[32m[PASS]\033[0m VM exists and is running\n'

        # VM-specific checks
        echo ""
        echo "VM Health:"
        check "SSH connectivity" limactl shell substrate uname -a

        # Check agent socket
        if limactl shell substrate sudo -n test -S /run/substrate.sock 2>/dev/null; then
            printf '\033[32m[PASS]\033[0m Agent socket exists\n'

            # Check agent capabilities
            if limactl shell substrate sudo -n timeout 5 curl --fail --unix-socket /run/substrate.sock http://localhost/v1/capabilities >/dev/null 2>&1; then
                printf '\033[32m[PASS]\033[0m Agent responding to capabilities request\n'
            else
                printf '\033[31m[FAIL]\033[0m Agent not responding (service may not be running)\n'
                failures=$((failures+1))
            fi

            socket_meta=$(limactl shell substrate sudo -n stat -c '%U:%G %a' /run/substrate.sock 2>/dev/null || true)
            if [[ "${socket_meta}" == "root:substrate 660" ]]; then
                printf '\033[32m[PASS]\033[0m Socket ownership root:substrate (0660)\n'
            else
                printf '\033[33m[WARN]\033[0m Socket metadata %s (expected root:substrate 660). Run scripts/mac/lima-warm.sh to repair.\n' "${socket_meta:-unknown}"
            fi

            vm_user=$(limactl shell substrate id -un 2>/dev/null | tr -d '\r' || true)
            if [[ -n "${vm_user}" ]] && limactl shell substrate id -nG "${vm_user}" 2>/dev/null | tr ' ' '\n' | grep -qx substrate; then
                printf '\033[32m[PASS]\033[0m %s belongs to substrate group\n' "${vm_user}"
            else
                printf '\033[33m[WARN]\033[0m Unable to confirm substrate group membership for %s. Run scripts/mac/lima-warm.sh.\n' "${vm_user:-guest}"
            fi

            layout_version=$(limactl shell substrate sudo -n cat /etc/substrate-lima-layout 2>/dev/null | tr -d '\r' || true)
            if [[ "${layout_version}" == "${LAYOUT_EXPECTED}" ]]; then
                printf '\033[32m[PASS]\033[0m Socket parity layout detected (%s)\n' "${layout_version}"
            else
                printf '\033[33m[WARN]\033[0m Layout sentinel %s (expected %s). Run scripts/mac/lima-warm.sh to rebuild.\n' "${layout_version:-missing}" "${LAYOUT_EXPECTED}"
            fi
        else
            printf '\033[33m[WARN]\033[0m Agent socket not found (agent may not be installed)\n'
        fi

        # Check systemd service
        if limactl shell substrate systemctl is-active substrate-world-agent >/dev/null 2>&1; then
            printf '\033[32m[PASS]\033[0m substrate-world-agent service is active\n'
        else
            printf '\033[31m[FAIL]\033[0m substrate-world-agent service is not active\n'
            failures=$((failures+1))
        fi

        # Check nftables
        check "nftables available" limactl shell substrate which nft

        # Check disk usage
        echo ""
        echo "Disk Usage:"
        limactl shell substrate bash -lc 'df -h / | tail -1' 2>/dev/null || echo "Could not get disk usage"

    else
        printf '\033[33m[WARN]\033[0m VM exists but is not running (status: %s)\n' "$status"
        echo "  Run: scripts/mac/lima-warm.sh"
    fi
else
    printf '\033[33m[WARN]\033[0m VM 'substrate' does not exist\n'
    echo "  Run: scripts/mac/lima-warm.sh to create VM"
fi

echo ""
if [ $failures -ne 0 ]; then
    echo "Doctor detected $failures issue(s). See above output for details." >&2
    exit 1
else
    echo "All critical checks passed."
    exit 0
fi
