#!/usr/bin/env bash
set -euo pipefail

SCRIPT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_ROOT}/.." && pwd)"
DRIVER="${REPO_ROOT}/scripts/dev/substrate_shell_driver"

# Phase D validation script: shim status/deploy/resolution basics

BIN="${BIN:-${REPO_ROOT}/target/debug/substrate}"

echo "== Phase D: validate --shim-status, redeploy, PATH checks =="

if [[ ! -x "$BIN" ]]; then
  echo "error: substrate binary not found at $BIN" >&2
  exit 2
fi

run_substrate() {
  SUBSTRATE_BIN="${BIN}" "${DRIVER}" "$@"
}

echo "-- Baseline status --"
set +e
run_substrate --shim-status
RC=$?
set -e
echo "RC=$RC"

echo
echo "-- Simulate drift (remove one shim) --"
SHIMS_DIR="$HOME/.substrate/shims"
if [[ -d "$SHIMS_DIR" ]]; then
  rm -f "$SHIMS_DIR/npm" || true
else
  echo "shims dir not found; attempting deploy first"
  run_substrate --shim-deploy || true
fi

set +e
run_substrate --shim-status
RC=$?
set -e
echo "RC=$RC (expect 1 and \"Needs redeploy\")"

echo
echo "-- Force redeploy --"
run_substrate --shim-deploy || true

set +e
run_substrate --shim-status
RC=$?
set -e
echo "RC=$RC (expect 0)"

echo
echo "-- PATH misorder (prepend /usr/bin) --"
export PATH="/usr/bin:$PATH"
set +e
run_substrate --shim-status
RC=$?
set -e
echo "RC=$RC (expect 0 with PATH: WARN)"

echo
echo "Done."

