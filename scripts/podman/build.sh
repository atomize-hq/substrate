#!/usr/bin/env bash
set -euo pipefail

# Optional: export BUILD_ARCH=amd64 to force x86_64 image
# Optional: pass extra build args, e.g. --build-arg CODEX_NPM_PKG=@openai/codex

set -x
EXTRA_ARGS=""
if [[ -n "${BUILD_ARCH:-}" ]]; then
  EXTRA_ARGS="--arch ${BUILD_ARCH} --os linux"
fi

echo "[podman] Building image with podman (no external compose)"
podman build ${EXTRA_ARGS} -t substrate-dev -f Dockerfile . "$@"
set +x
