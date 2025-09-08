#!/usr/bin/env bash
set -euo pipefail

pass() { echo -e "[PASS] $*"; }
warn() { echo -e "[WARN] $*"; }
fail() { echo -e "[FAIL] $*"; }

echo "=== Substrate + Codex Container Prereq Check ==="
echo "Container kernel: $(uname -a)"

echo -n "nft version: "; (nft --version 2>/dev/null || echo n/a)
echo -n "curl version: "; (curl --version 2>/dev/null | head -n1 || echo n/a)

if [[ -f /sys/fs/cgroup/cgroup.controllers ]]; then pass "cgroup v2 mounted"; else warn "cgroup v2 not mounted"; fi

if sysctl -a 2>/dev/null | grep -q '^kernel.unprivileged_userns_clone ='; then
  V=$(sysctl -n kernel.unprivileged_userns_clone || echo n/a)
  echo "kernel.unprivileged_userns_clone=$V"
fi
if sysctl -a 2>/dev/null | grep -q '^kernel.dmesg_restrict ='; then
  V=$(sysctl -n kernel.dmesg_restrict || echo n/a)
  echo "kernel.dmesg_restrict=$V"
fi

if [[ -e /dev/fuse ]]; then pass "/dev/fuse present"; else warn "/dev/fuse missing"; fi
if command -v fuse-overlayfs &>/dev/null; then pass "fuse-overlayfs installed"; else warn "fuse-overlayfs not installed"; fi

echo "Testing nftables list (benign)"
if nft list tables >/dev/null 2>&1; then pass "nft list tables OK"; else warn "nft list tables failed (needs privileged + modules)"; fi

echo "Done."

