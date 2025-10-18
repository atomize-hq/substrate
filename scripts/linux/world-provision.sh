#!/usr/bin/env bash
# Provision the Substrate world-agent systemd service on a Linux host.
#
# This script builds the world-agent binary (unless --skip-build is set),
# installs it under /usr/local/bin, writes the systemd unit file, and
# enables/starts the service so /run/substrate.sock is owned by root.
#
# Usage: scripts/linux/world-provision.sh [--profile release|debug] [--skip-build]

set -euo pipefail

PROFILE=release
SKIP_BUILD=0

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
        -h|--help)
            cat <<'USAGE'
Usage: scripts/linux/world-provision.sh [options]

Options:
  --profile <name>   Cargo profile to build (default: release)
  --skip-build       Assume target/<profile>/world-agent already exists
  -h, --help         Show this help
USAGE
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
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
        cargo build -p world-agent --release --manifest-path "${REPO_ROOT}/Cargo.toml"
        BIN_PATH="${REPO_ROOT}/target/release/world-agent"
    else
        cargo build -p world-agent --profile "${PROFILE}" --manifest-path "${REPO_ROOT}/Cargo.toml"
    fi
else
    echo "==> Skipping build as requested"
fi

if [[ ! -x "${BIN_PATH}" ]]; then
    echo "world-agent binary not found at ${BIN_PATH}. Did the build succeed?" >&2
    exit 1
fi

SERVICE_PATH="/etc/systemd/system/substrate-world-agent.service"

echo "==> Installing world-agent to /usr/local/bin (sudo will prompt if needed)"
sudo install -Dm0755 "${BIN_PATH}" /usr/local/bin/substrate-world-agent

echo "==> Ensuring runtime directories exist"
sudo install -d -m0750 /run/substrate
sudo install -d -m0750 /var/lib/substrate

TMP_UNIT=$(mktemp)
cat <<'UNIT' > "${TMP_UNIT}"
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

echo "==> Writing systemd unit to ${SERVICE_PATH}"
sudo install -Dm0644 "${TMP_UNIT}" "${SERVICE_PATH}"
rm -f "${TMP_UNIT}"

echo "==> Reloading systemd and starting substrate-world-agent"
sudo systemctl daemon-reload
sudo systemctl enable --now substrate-world-agent.service

echo "==> substrate-world-agent status (last 10 log lines)"
sudo systemctl status substrate-world-agent.service --no-pager --lines=10 || true

echo "==> Provisioning complete"
echo "    Verify socket with: sudo ls -l /run/substrate.sock"
echo "    Probe capabilities: sudo curl --unix-socket /run/substrate.sock http://localhost/v1/capabilities"
