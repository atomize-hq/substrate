#!/usr/bin/env bash
set -euo pipefail

echo "[podman] Validating VM kernel prerequisites"
podman machine ssh <<'EOF'
set -euo pipefail
echo "== VM Checks =="
uname -a
echo -n "unprivileged_userns_clone="; sysctl -n kernel.unprivileged_userns_clone || echo n/a
echo -n "dmesg_restrict="; sysctl -n kernel.dmesg_restrict || echo n/a
grep overlay /proc/filesystems || true
test -f /sys/fs/cgroup/cgroup.controllers && echo "cgroup v2 present" || echo "cgroup v2 missing"
EOF

echo "[podman] Validating container prerequisites"
podman run --rm -it \
  --privileged \
  --security-opt seccomp=unconfined \
  --security-opt label=disable \
  --cap-add=ALL \
  --device /dev/fuse \
  -v "$PWD":/src -v "$HOME/.codex":/root/.codex \
  -w /src \
  substrate-dev bash -lc 'bash scripts/check-container-prereqs.sh || true'

echo "[podman] Validation done"
