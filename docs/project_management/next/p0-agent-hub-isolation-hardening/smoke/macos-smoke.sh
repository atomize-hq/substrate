#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "SKIP: agent hub hardening macOS smoke (not macOS)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

echo "OK: agent hub hardening macOS smoke (preflight)"

