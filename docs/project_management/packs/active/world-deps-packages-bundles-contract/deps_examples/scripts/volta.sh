#!/usr/bin/env bash
set -euo pipefail

# Example script install that installs Volta into the world-deps prefix and exposes:
#   /var/lib/substrate/world-deps/bin/volta
#
# Note: This is only an example; real recipes may need additional OS deps.

world_deps_root="/var/lib/substrate/world-deps"
world_deps_bin="${world_deps_root}/bin"
volta_home="${world_deps_root}/volta"

mkdir -p "${world_deps_bin}"
mkdir -p "${volta_home}"

export VOLTA_HOME="${volta_home}"
export PATH="${volta_home}/bin:${PATH}"

if [ -x "${volta_home}/bin/volta" ]; then
  "${volta_home}/bin/volta" self update || true
else
  curl -fsSL https://get.volta.sh | bash
fi

ln -sf "${volta_home}/bin/volta" "${world_deps_bin}/volta"
