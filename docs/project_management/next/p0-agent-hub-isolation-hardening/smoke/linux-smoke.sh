#!/usr/bin/env bash
set -euo pipefail

if [[ "$(uname -s)" != "Linux" ]]; then
  echo "SKIP: agent hub hardening linux smoke (not Linux)"
  exit 0
fi

if ! command -v substrate >/dev/null 2>&1; then
  echo "FAIL: substrate not found on PATH" >&2
  exit 1
fi

# This smoke script is intentionally minimal and only verifies that the harness can run.
# Deeper validation is described in manual_testing_playbook.md and must be exercised once the I0–I5 work lands.
echo "OK: agent hub hardening linux smoke (preflight)"
