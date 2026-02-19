#!/bin/sh
set -eu

# Installs nvm into the Substrate-managed world-deps prefix instead of $HOME.
#
# Why:
# - In hardened worlds, $HOME (typically /root) may be read-only.
# - Tooling must be runnable without relying on shell rc/profile files.

NVM_VERSION="${NVM_VERSION:-v0.39.7}"
NVM_DIR="${NVM_DIR:-/var/lib/substrate/world-deps/nvm}"
export NVM_DIR

if ! command -v bash >/dev/null 2>&1; then
  echo "substrate: world deps install failed for nvm: bash is required in the world image" >&2
  exit 127
fi

mkdir -p "$NVM_DIR"

INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/${NVM_VERSION}/install.sh"

fetch() {
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1"
    return 0
  fi
  if command -v wget >/dev/null 2>&1; then
    wget -qO- "$1"
    return 0
  fi
  echo "substrate: world deps install failed for nvm: neither curl nor wget is available in the world image" >&2
  exit 127
}

# Prevent the upstream installer from mutating shell profile files.
export PROFILE=/dev/null

fetch "$INSTALL_URL" | bash

if [ ! -f "$NVM_DIR/nvm.sh" ]; then
  echo "substrate: world deps install failed for nvm: expected $NVM_DIR/nvm.sh to exist after install" >&2
  exit 1
fi

