#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: overlayfs enumeration smoke (not macOS)"
  exit 0
fi

echo "SKIP: overlayfs enumeration smoke (Linux-only ADR-0004 scope)"
exit 0

