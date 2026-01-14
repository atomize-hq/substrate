set -euo pipefail

# Example script install that places the runnable entrypoint under:
#   /var/lib/substrate/world-deps/bin

world_deps_root="/var/lib/substrate/world-deps"
world_deps_bin="${world_deps_root}/bin"
bun_root="${world_deps_root}/bun"

mkdir -p "${world_deps_bin}"
mkdir -p "${bun_root}"

export BUN_INSTALL="${bun_root}"

if [ -x "${bun_root}/bin/bun" ]; then
  "${bun_root}/bin/bun" upgrade
else
  curl -fsSL https://bun.sh/install | bash
fi

ln -sf "${bun_root}/bin/bun" "${world_deps_bin}/bun"

