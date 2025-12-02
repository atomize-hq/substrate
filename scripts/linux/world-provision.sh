#!/usr/bin/env bash
# Provision the Substrate world-agent systemd service + socket on a Linux host.
#
# This script builds the world-agent binary (unless --skip-build is set),
# installs it under /usr/local/bin, writes both the .service and .socket unit
# files, and enables/starts the listener so /run/substrate.sock is owned by
# root. Pass --dry-run to print the actions without invoking sudo.
#
# Usage: scripts/linux/world-provision.sh [--profile <name>] [--skip-build] [--dry-run]

set -euo pipefail

PROFILE=release
SKIP_BUILD=0
DRY_RUN=0

show_cmd() {
    printf '[dry-run]'
    for arg in "$@"; do
        printf ' %q' "$arg"
    done
    printf '\n'
}

run_cmd() {
    if [[ ${DRY_RUN} -eq 1 ]]; then
        show_cmd "$@"
    else
        "$@"
    fi
}

install_unit() {
    local destination="$1"
    local content="$2"
    if [[ ${DRY_RUN} -eq 1 ]]; then
        printf '[dry-run] install unit %s\n' "${destination}"
        return
    fi
    local tmp
    tmp=$(mktemp)
    printf '%s\n' "${content}" >"${tmp}"
    sudo install -Dm0644 "${tmp}" "${destination}"
    rm -f "${tmp}"
}

usage() {
    cat <<'USAGE'
Usage: scripts/linux/world-provision.sh [options]

Options:
  --profile <name>   Cargo profile to build (default: release)
  --skip-build       Assume target/<profile>/world-agent already exists
  --dry-run          Print the provisioning steps without executing them
  -h, --help         Show this help
USAGE
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --skip-build)
            SKIP_BUILD=1
            shift
            ;;
        --dry-run)
            DRY_RUN=1
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage
            exit 1
            ;;
    esac
 done

if [[ ${EUID} -eq 0 ]]; then
    echo "Please run this script without sudo; it will invoke sudo when needed." >&2
    exit 1
fi

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "${SCRIPT_DIR}/../.." && pwd)
BIN_PATH="${REPO_ROOT}/target/${PROFILE}/world-agent"

if [[ ${SKIP_BUILD} -eq 0 ]]; then
    echo "==> Building world-agent (profile: ${PROFILE})"
    if [[ "${PROFILE}" == "release" ]]; then
        BIN_PATH="${REPO_ROOT}/target/release/world-agent"
        if [[ ${DRY_RUN} -eq 1 ]]; then
            show_cmd cargo build -p world-agent --release --manifest-path "${REPO_ROOT}/Cargo.toml"
        else
            cargo build -p world-agent --release --manifest-path "${REPO_ROOT}/Cargo.toml"
        fi
    else
        if [[ ${DRY_RUN} -eq 1 ]]; then
            show_cmd cargo build -p world-agent --profile "${PROFILE}" --manifest-path "${REPO_ROOT}/Cargo.toml"
        else
            cargo build -p world-agent --profile "${PROFILE}" --manifest-path "${REPO_ROOT}/Cargo.toml"
        fi
    fi
else
    echo "==> Skipping build as requested"
fi

if [[ ${DRY_RUN} -eq 0 && ! -x "${BIN_PATH}" ]]; then
    echo "world-agent binary not found at ${BIN_PATH}. Did the build succeed?" >&2
    exit 1
fi

SERVICE_PATH="/etc/systemd/system/substrate-world-agent.service"
SOCKET_PATH="/etc/systemd/system/substrate-world-agent.socket"

read -r -d '' SERVICE_UNIT_CONTENT <<'UNIT' || true
[Unit]
Description=Substrate World Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=SUBSTRATE_AGENT_TCP_PORT=61337
Environment=SUBSTRATE_WORLD_SOCKET=/run/substrate.sock
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
ReadWritePaths=/var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
UNIT

read -r -d '' SOCKET_UNIT_CONTENT <<'UNIT' || true
[Unit]
Description=Substrate World Agent Socket
PartOf=substrate-world-agent.service

[Socket]
ListenStream=/run/substrate.sock
SocketMode=0660
SocketUser=root
SocketGroup=root
DirectoryMode=0750
RemoveOnStop=yes
Service=substrate-world-agent.service

[Install]
WantedBy=sockets.target
UNIT

echo "==> Installing world-agent to /usr/local/bin (sudo will prompt if needed)"
run_cmd sudo install -Dm0755 "${BIN_PATH}" /usr/local/bin/substrate-world-agent

echo "==> Ensuring runtime directories exist"
run_cmd sudo install -d -m0750 /run/substrate
run_cmd sudo install -d -m0750 /var/lib/substrate

echo "==> Writing systemd units to ${SERVICE_PATH} and ${SOCKET_PATH}"
install_unit "${SERVICE_PATH}" "${SERVICE_UNIT_CONTENT}"
install_unit "${SOCKET_PATH}" "${SOCKET_UNIT_CONTENT}"

echo "==> Reloading systemd and enabling socket activation"
run_cmd sudo systemctl daemon-reload
run_cmd sudo systemctl enable substrate-world-agent.service
run_cmd sudo systemctl enable --now substrate-world-agent.socket
run_cmd sudo systemctl restart substrate-world-agent.service

echo "==> substrate-world-agent.socket status (last 10 log lines)"
run_cmd sudo systemctl status substrate-world-agent.socket --no-pager --lines=10 || true
echo "==> substrate-world-agent.service status (last 10 log lines)"
run_cmd sudo systemctl status substrate-world-agent.service --no-pager --lines=10 || true

echo "==> Provisioning complete"
echo "    Verify socket with: sudo ls -l /run/substrate.sock"
echo "    Probe capabilities: sudo curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities"
echo "    Doctor socket block: substrate world doctor --json | jq '.world_socket'"
echo "    Shim summary: substrate --shim-status | grep 'World socket'"
