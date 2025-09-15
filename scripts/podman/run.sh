#!/usr/bin/env bash
set -euo pipefail

echo "[podman] Running via podman run (no external compose)"
podman run --rm -it \
  --privileged \
  --security-opt seccomp=unconfined \
  --security-opt label=disable \
  --cap-add=ALL \
  --device /dev/fuse \
  -v "$PWD":/src -v "$HOME/.codex":/root/.codex \
  -w /src \
  substrate-dev "$@"
