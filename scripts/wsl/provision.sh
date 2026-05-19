#!/usr/bin/env bash
set -euo pipefail

cat >&2 <<'EOF'
[substrate/wsl-provision][ERROR] WSL world provisioning is intentionally fail-closed in this slice.
The WSL helper path is not aligned with the Linux/macOS placement contract for
SUBSTRATE_HOME placement, socket/group ownership, and runtime artifact access.

Do not use this script to mutate a WSL guest. Use one of these paths instead:
  - Linux host-native provisioning via scripts/linux/world-provision.sh
  - macOS Lima provisioning via scripts/mac/lima-warm.sh
  - CLI-only flows inside WSL with --no-world until WSL alignment lands
EOF

exit 4
