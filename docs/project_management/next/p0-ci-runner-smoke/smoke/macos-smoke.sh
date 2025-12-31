#!/usr/bin/env bash
set -euo pipefail

echo "[macos-smoke] substrate --version"
ver="$(substrate --version)"
if [[ -z "${ver}" ]]; then
  echo "ERROR: substrate --version produced empty output" >&2
  exit 1
fi
echo "OK: ${ver}"
