#!/usr/bin/env bash
set -euo pipefail

pass() { echo -e "[PASS] $*"; }
warn() { echo -e "[WARN] $*"; }
fail() { echo -e "[FAIL] $*"; }

echo "=== Substrate + Codex Host Prereq Check ==="
echo "OS: $(uname -a)"

# 1) Kernel version
KVER=$(uname -r || true)
echo "Kernel: $KVER"

# 2) User namespaces
if sysctl -a 2>/dev/null | grep -q '^kernel.unprivileged_userns_clone ='; then
  V=$(sysctl -n kernel.unprivileged_userns_clone || echo n/a)
  if [[ "$V" == "1" ]]; then pass "unprivileged_userns_clone=1"; else warn "unprivileged_userns_clone=$V (set to 1 for overlayfs-in-userns)"; fi
else
  warn "sysctl kernel.unprivileged_userns_clone not readable (may be fine on macOS Docker Desktop)"
fi

# 3) OverlayFS support
if grep -q overlay /proc/filesystems 2>/dev/null || lsmod 2>/dev/null | grep -q '^overlay'; then
  pass "OverlayFS available"
else
  warn "OverlayFS not detected in filesystems/modules; world overlay isolation may degrade"
fi

# 4) cgroup v2 check
if [[ -f /sys/fs/cgroup/cgroup.controllers ]]; then
  pass "cgroup v2 present"
else
  warn "cgroup v2 not detected on host; container will attempt mount (best-effort)"
fi

# 5) Docker info (if available)
if command -v docker &>/dev/null; then
  if docker info >/dev/null 2>&1; then
    RUNTIME=$(docker info --format '{{.Runtimes.runc.Path}}' 2>/dev/null || echo runc)
    echo "Docker runtime: $RUNTIME"
    if docker info 2>/dev/null | grep -qi 'userns'; then
      echo "Docker userns features detected (good)"
    else
      echo "Docker userns features not visible (may be fine)"
    fi
  else
    warn "docker info unavailable (daemon not running?)"
  fi
else
  warn "docker not installed"
fi

# 6) /dev/fuse presence (for fuse-overlayfs fallback)
if [[ -e /dev/fuse ]]; then pass "/dev/fuse present"; else warn "/dev/fuse missing; add --device /dev/fuse for fuse-overlayfs fallback"; fi

echo "=== Guidance ==="
cat <<'EOT'
- If user namespaces are disabled (unprivileged_userns_clone=0), overlayfs-in-userns may fail with EINVAL.
  Options:
  - Linux host: sudo sysctl -w kernel.unprivileged_userns_clone=1; persist in /etc/sysctl.conf
  - Use Podman (better userns support): podman run --privileged --userns=keep-id ...
  - Use fuse-overlayfs fallback: ensure /dev/fuse is available and we pass --device /dev/fuse
- macOS Docker Desktop: kernel is inside a VM; userns/overlayfs support may be limited. World isolation will degrade gracefully.
- Ensure privileged runs: --privileged --security-opt seccomp=unconfined --security-opt apparmor=unconfined --cap-add=ALL
EOT

echo "Done."

