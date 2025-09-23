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
ExecStart=/usr/local/bin/substrate-world-agent --socket /run/substrate.sock
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

systemctl daemon-reload
systemctl enable substrate-world-agent.service