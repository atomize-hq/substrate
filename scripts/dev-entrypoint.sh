#!/usr/bin/env bash
set -euo pipefail

# Purpose: prepare the container for substrate privileged tests when run with --privileged
# - try to enable user namespaces (best-effort)
# - ensure cgroup v2 is mounted
# - ensure tracefs/debugfs are mounted (for extra debugging if needed)
# - try to relax dmesg restrictions for nft LOG checks
# - load relevant kernel modules if possible (best-effort)

echo "[entrypoint] Starting privileged test environment setup"

export HOME=${HOME:-/root}
export RUST_LOG=${RUST_LOG:-info}

# Best-effort sysctls (may require --privileged and host to allow)
for kv in \
  kernel.unprivileged_userns_clone=1 \
  kernel.dmesg_restrict=0 \
  net.ipv4.ip_unprivileged_port_start=0 \
; do
  if sysctl -w "$kv" 2>/dev/null; then
    echo "[entrypoint] sysctl set: $kv"
  else
    echo "[entrypoint] sysctl failed (benign): $kv"
  fi
done

# Ensure cgroup v2 is mounted at /sys/fs/cgroup
# Detect by presence of cgroup.controllers file
if [[ ! -f /sys/fs/cgroup/cgroup.controllers ]]; then
  echo "[entrypoint] Mounting cgroup v2 at /sys/fs/cgroup (best-effort)"
  # Some base images mount tmpfs here; attempt to remount cgroup2
  # If it fails, we continue; substrate should degrade gracefully.
  if mount -t cgroup2 none /sys/fs/cgroup 2>/dev/null; then
    echo "[entrypoint] cgroup v2 mounted"
  else
    echo "[entrypoint] cgroup v2 mount failed (benign)"
  fi
fi

# Mount tracefs and debugfs for better observability (best-effort)
if ! mountpoint -q /sys/kernel/tracing 2>/dev/null; then
  mkdir -p /sys/kernel/tracing || true
  mount -t tracefs tracefs /sys/kernel/tracing 2>/dev/null || true
fi
if ! mountpoint -q /sys/kernel/debug 2>/dev/null; then
  mkdir -p /sys/kernel/debug || true
  mount -t debugfs debugfs /sys/kernel/debug 2>/dev/null || true
fi

# Try to pre-load kernel modules we rely on (nftables path)
for mod in nfnetlink nf_tables nf_conntrack; do
  if modprobe "$mod" 2>/dev/null; then
    echo "[entrypoint] modprobe ok: $mod"
  else
    echo "[entrypoint] modprobe failed (benign): $mod"
  fi
done

# Print quick diagnostics
echo "[entrypoint] Diagnostics:"
echo -n "  kernel.unprivileged_userns_clone="; sysctl -n kernel.unprivileged_userns_clone 2>/dev/null || echo "n/a"
echo -n "  kernel.dmesg_restrict="; sysctl -n kernel.dmesg_restrict 2>/dev/null || echo "n/a"
echo -n "  cgroup v2 present="; [[ -f /sys/fs/cgroup/cgroup.controllers ]] && echo yes || echo no
echo -n "  nft version="; (nft --version 2>/dev/null || echo "n/a")
echo -n "  curl version="; (curl --version 2>/dev/null | head -n1 || echo "n/a")

echo "[entrypoint] Environment ready. Exec: $*"
exec "$@"

