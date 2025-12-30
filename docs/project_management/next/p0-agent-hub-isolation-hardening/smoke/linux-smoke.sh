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

out="$(substrate --version 2>/dev/null || true)"
if [[ -z "${out}" ]]; then
  echo "FAIL: substrate --version produced no output" >&2
  exit 1
fi
echo "OK: agent hub hardening linux smoke"
