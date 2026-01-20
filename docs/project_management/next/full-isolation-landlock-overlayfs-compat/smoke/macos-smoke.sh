#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: full-isolation-landlock-overlayfs-compat smoke (not macOS)"
  exit 0
fi

echo "OK: macOS smoke is a no-op for this feature"
exit 0
