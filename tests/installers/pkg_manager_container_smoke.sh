#!/usr/bin/env bash
set -euo pipefail

if command -v docker >/dev/null 2>&1; then
  RUNTIME="docker"
elif command -v podman >/dev/null 2>&1; then
  RUNTIME="podman"
else
  echo "[pkg-manager-container-smoke] ERROR: docker or podman is required" >&2
  exit 2
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

run_check() {
  local image="$1"
  local expected_id="$2"
  local expected_mgr="$3"

  echo "[pkg-manager-container-smoke] image=${image} expected_id=${expected_id} expected_mgr=${expected_mgr}" >&2

  "${RUNTIME}" run --rm \
    -v "${repo_root}:/work" \
    -w /work \
    "${image}" \
    bash -lc "
      set -euo pipefail
      if [[ ! -f /etc/os-release ]]; then
        echo 'missing /etc/os-release' >&2
        exit 3
      fi
      # shellcheck disable=SC1091
      source /etc/os-release
      if [[ \"\${ID:-}\" != \"${expected_id}\" ]]; then
        echo \"expected ID=${expected_id}, got ID=\${ID:-<unset>}\" >&2
        echo \"os-release: \$(cat /etc/os-release)\" >&2
        exit 4
      fi
      if ! command -v \"${expected_mgr}\" >/dev/null 2>&1; then
        echo \"expected ${expected_mgr} to exist in PATH\" >&2
        echo \"PATH=\$PATH\" >&2
        exit 5
      fi
      echo \"ok: ID=${expected_id}, mgr=${expected_mgr}\" >&2
    "
}

run_check "ubuntu:24.04" "ubuntu" "apt-get"
run_check "archlinux:latest" "arch" "pacman"

echo "[pkg-manager-container-smoke] OK" >&2
