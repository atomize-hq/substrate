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

printf "\033[32mSubstrate uninstall running…\033[0m\n"
if "${UPSTREAM_UNINSTALL}" "$@" >"${TMP_LOG}" 2>&1; then
  printf "\033[32mSubstrate uninstall complete!\033[0m\n"
else
  printf "\033[31mSubstrate uninstall failed.\033[0m See %s for details.\n" "${TMP_LOG}" >&2
  cat "${TMP_LOG}" >&2
  exit 1
fi
