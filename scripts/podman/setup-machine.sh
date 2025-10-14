#!/usr/bin/env bash
set -euo pipefail

echo "[podman] Ensuring Podman machine is initialized and started"
if ! podman machine list >/dev/null 2>&1; then
  echo "[podman] podman CLI not found or not initialized" >&2
  exit 1
fi

if ! podman machine inspect default >/dev/null 2>&1; then
  echo "[podman] Initializing default machine"
  podman machine init --cpus 4 --memory 8192 --disk-size 40 || true
fi

podman machine start

echo "[podman] Configuring kernel sysctls and modules in the VM"
podman machine ssh <<'EOF'
set -euo pipefail
echo "== VM Kernel Setup =="
echo 'kernel.unprivileged_userns_clone=1' | sudo tee /etc/sysctl.d/99-userns.conf >/dev/null
echo 'kernel.dmesg_restrict=0' | sudo tee /etc/sysctl.d/99-dmesg.conf >/dev/null
sudo sysctl --system || true

echo overlay | sudo tee /etc/modules-load.d/overlay.conf >/dev/null
echo -e "nf_tables\nnf_conntrack\nnfnetlink" | sudo tee /etc/modules-load.d/nft.conf >/dev/null

sudo modprobe overlay || true
sudo modprobe nf_tables || true
sudo modprobe nf_conntrack || true
sudo modprobe nfnetlink || true

echo "== VM Checks =="
uname -a
sysctl -n kernel.unprivileged_userns_clone || true
sysctl -n kernel.dmesg_restrict || true
grep overlay /proc/filesystems || true
test -f /sys/fs/cgroup/cgroup.controllers && echo "cgroup v2 present" || echo "cgroup v2 missing"
EOF

# Try to set the default connection to the machine (handles rootful/rootless variants)
echo "[podman] Setting default connection to the VM"
podman system connection list || true
podman system connection default podman-machine-default 2>/dev/null || true
podman system connection default podman-machine-default-root 2>/dev/null || true
podman system connection default podman-machine-default-rootful 2>/dev/null || true
podman system connection default podman-machine-default-rootless 2>/dev/null || true

echo "[podman] VM ready"
