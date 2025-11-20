#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UPSTREAM_UNINSTALL="${SCRIPT_DIR}/uninstall-substrate.sh"

if [[ ! -x "${UPSTREAM_UNINSTALL}" ]]; then
  echo "[substrate-uninstall] missing uninstaller at ${UPSTREAM_UNINSTALL}" >&2
  exit 1
fi

TMP_LOG="$(mktemp -t substrate-uninstall-log.XXXXXX)"
cleanup() { rm -f "${TMP_LOG}"; }
trap cleanup EXIT

echo "[substrate-uninstall] Removing Substrate (logs: ${TMP_LOG})"
if "${UPSTREAM_UNINSTALL}" "$@" >"${TMP_LOG}" 2>&1; then
  echo "[substrate-uninstall] Done."
else
  echo "[substrate-uninstall] Failed. See ${TMP_LOG} for details." >&2
  cat "${TMP_LOG}" >&2
  exit 1
fi
