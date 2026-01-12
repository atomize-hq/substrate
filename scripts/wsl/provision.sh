#!/usr/bin/env bash
set -euo pipefail

export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y \
    nftables \
    iproute2 \
    libseccomp-dev \
    curl \
    jq \
    git \
    python3 \
    python3-pip \
    build-essential \
    dnsmasq \
    openssh-server \
    unzip \
    ca-certificates

if ! command -v cargo >/dev/null 2>&1; then
    echo "[INFO] Installing Rust toolchain via rustup (required to build world-agent)"
    curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.89.0
fi

# Ensure cargo/rustup are on PATH for subsequent provisioning steps.
if [[ -f /root/.cargo/env ]]; then
    # shellcheck disable=SC1091
    . /root/.cargo/env
fi

install -d -m 0700 /run/substrate
install -d -m 0755 /etc/substrate
install -d -m 0755 /var/log/substrate
install -d -m 0755 /var/lib/substrate

cat <<'UNIT' >/etc/systemd/system/substrate-world-agent.service
[Unit]
Description=Substrate World Agent
After=network.target

[Service]
Type=simple
Environment="SUBSTRATE_AGENT_TCP_PORT=61337"
Environment="SUBSTRATE_WORLD_SOCKET=/run/substrate.sock"
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
User=root
Group=root
RuntimeDirectory=substrate
RuntimeDirectoryMode=0700
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
UNIT

cat <<'UNIT' >/etc/systemd/system/substrate-world-agent.socket
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

systemctl daemon-reload
systemctl enable substrate-world-agent.service
systemctl enable --now substrate-world-agent.socket
